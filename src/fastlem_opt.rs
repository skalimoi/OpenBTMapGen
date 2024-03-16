use fastlem::core::parameters::TopographicalParameters;
use fastlem::core::traits::Model;
use fastlem::core::units::Elevation;
use fastlem::lem::generator::TerrainGenerator;
use fastlem::models::surface::builder::TerrainModel2DBulider;
use fastlem::models::surface::sites::Site2D;
use fltk::enums::ColorDepth;
use fltk::image;
use fltk::image::SharedImage;
use fltk::prelude::{ImageExt, WidgetExt};
use image_crate::{ImageBuffer, Luma};
use noise::{NoiseFn, Perlin};
use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;
use crate::FileData;
use crate::topo_settings::TopoSettings;
use crate::topography::DEFAULT_TOPOSETTINGS;

pub fn generate_terrain(w: &mut impl WidgetExt, data: &mut FileData) {
    let bound_min = Site2D {x: data.topography.min_bound.0, y: data.topography.min_bound.1} ;
    let bound_max = Site2D {x: data.topography.max_bound.0, y: data.topography.max_bound.1} ;
    let seed = data.topography.seed.unwrap();

    let perlin = Perlin::new(seed);

    let num_per_square = data.topography.lod;

    let num = ((bound_max.x - bound_min.x) * (bound_max.y - bound_min.y) * num_per_square) as usize;
    let bound_range = Site2D {
        x: bound_max.x - bound_min.x,
        y: bound_max.y - bound_min.y,
    };

    let model = TerrainModel2DBulider::from_random_sites(num, bound_min, bound_max)
        .relaxate_sites(1)
        .unwrap()
        .add_edge_sites(None, None)
        .unwrap()
        .build()
        .unwrap();

    let sites = model.sites().to_vec();

    // fault
    let max_fault_radius = 35.0;
    let get_fault = |site: &Site2D| -> (f64, f64) {
        let scale = 100.0;

        let modulus = octaved_perlin(&perlin, site.x / scale, site.y / scale, 3, 0.5, 2.0).abs()
            * 2.0
            * max_fault_radius;
        let direction_x = octaved_perlin(
            &perlin,
            (site.x + bound_range.x) / scale,
            (site.y + bound_range.y) / scale,
            4,
            0.6,
            2.2,
        ) * 2.0;
        let direction_y = octaved_perlin(
            &perlin,
            (site.x - bound_range.x) / scale,
            (site.y - bound_range.y) / scale,
            4,
            0.6,
            2.2,
        ) * 2.0;
        (direction_x * modulus, direction_y * modulus)
    };

    // reef
    let max_reef_radius = 5.0;
    let get_reef = |site: &Site2D| -> (f64, f64) {
        let scale_scale = 800.0;
        let min_scale = 7.5;
        let max_scale = 20.0;
        let scale = octaved_perlin(
            &perlin,
            site.x / scale_scale,
            site.y / scale_scale,
            2,
            0.5,
            2.0,
        )
            .abs()
            * 2.0
            * (max_scale - min_scale)
            + min_scale;
        let modulus_rt =
            octaved_perlin(&perlin, site.x / scale, site.y / scale, 3, 0.5, 2.0).abs() * 2.0;
        let modulus = modulus_rt.powf(3.5) * max_reef_radius;
        let direction_x = octaved_perlin(
            &perlin,
            (site.x + bound_range.x) / scale,
            (site.y + bound_range.y) / scale,
            4,
            0.6,
            2.2,
        );
        let direction_y = octaved_perlin(
            &perlin,
            (site.x - bound_range.x) / scale,
            (site.y - bound_range.y) / scale,
            4,
            0.6,
            2.2,
        );

        (direction_x * modulus, direction_y * modulus)
    };

    let apply_fault = |site: &Site2D| -> Site2D {
        let fault = get_fault(site);
        let fault_x = site.x + fault.0;
        let fault_y = site.y + fault.1;
        Site2D {
            x: fault_x,
            y: fault_y,
        }
    };

    let apply_reef = |site: &Site2D| -> Site2D {
        let reef = get_reef(site);
        let reef_x = site.x + reef.0;
        let reef_y = site.y + reef.1;
        Site2D {
            x: reef_x,
            y: reef_y,
        }
    };

    let base_is_outlet = {
        sites
            .iter()
            .map(|site| {
                let site = apply_reef(&apply_fault(site));
                let persistence_scale = 50.;

                let plate_scale = 50.;
                let noise_persistence = octaved_perlin(
                    &perlin,
                    site.x / persistence_scale,
                    site.y / persistence_scale,
                    2,
                    0.5,
                    2.0,
                )
                    .abs()
                    * 0.7
                    + 0.3;
                let noise_plate = octaved_perlin(
                    &perlin,
                    site.x / plate_scale,
                    site.y / plate_scale,
                    8,
                    noise_persistence,
                    2.4,
                ) * 0.5
                    + 0.5;
                let continent_scale = 200.;
                let noise_continent = octaved_perlin(
                    &perlin,
                    site.x / continent_scale,
                    site.y / continent_scale,
                    3,
                    0.5,
                    1.8,
                ) * 0.7
                    + 0.5;
                let ocean_bias = 0.035;
                noise_plate > noise_continent - data.topography.sea_pct //noise_plate > noise_continent - ocean_bias
            })
            .collect::<Vec<bool>>()
    };

    let start_index = (num + 1..sites.len()).collect::<Vec<_>>();
    let graph = model.graph();

    let is_outlet = determine_outlets(&sites, base_is_outlet, start_index, graph).unwrap();

    let parameters = {
        sites
            .iter()
            .enumerate()
            .map(|(i, site)| {
                let site = apply_reef(&apply_fault(site));
                let erodibility_scale = data.topography.erod_scale;
                let noise_erodibility = (octaved_perlin(
                    &perlin,
                    site.x / erodibility_scale,
                    site.y / erodibility_scale,
                    5,
                    0.7,
                    2.2,
                )
                    .abs()
                    * 4.0
                    + 0.1) * data.topography.mountain_pct;
                TopographicalParameters::default()
                    .set_erodibility(noise_erodibility)
                    .set_is_outlet(is_outlet[i])
            })
            .collect::<Vec<TopographicalParameters>>()
    };

    let terrain = TerrainGenerator::default()
        .set_model(model)
        .set_parameters(parameters)
        .generate()
        .unwrap();

    let colormap: Vec<([u8; 3], f64)> = vec![
        ([70, 150, 200], 0.0),
        ([240, 240, 210], 0.5),
        ([190, 200, 120], 1.0),
        ([25, 100, 25], 18.0),
        ([15, 60, 15], 30.0),
    ];
    // get color from altitude
    let get_color = |altitude: f64| -> [u8; 3] {
        let color_index = {
            let mut i = 0;
            while i < colormap.len() {
                if altitude < colormap[i].1 {
                    break;
                }
                i += 1;
            }
            i
        };

        if color_index == 0 {
            colormap[0].0
        } else if color_index == colormap.len() {
            colormap[colormap.len() - 1].0
        } else {
            let color_a = colormap[color_index - 1];
            let color_b = colormap[color_index];

            let prop_a = color_a.1;
            let prop_b = color_b.1;

            let prop = (altitude - prop_a) / (prop_b - prop_a);

            let color = [
                (color_a.0[0] as f64 + (color_b.0[0] as f64 - color_a.0[0] as f64) * prop) as u8,
                (color_a.0[1] as f64 + (color_b.0[1] as f64 - color_a.0[1] as f64) * prop) as u8,
                (color_a.0[2] as f64 + (color_b.0[2] as f64 - color_a.0[2] as f64) * prop) as u8,
            ];

            color
        }
    };

    let img_width = 512;
    let img_height = 512;

    let mut color_buf = image_crate::RgbImage::new(img_width, img_height);
    let mut raw_buf: ImageBuffer<Luma<u16>, Vec<u16>> = image_crate::ImageBuffer::new(img_width, img_height);

    let max_elevation = terrain
        .elevations()
        .iter()
        .fold(std::f64::MIN, |acc, &n| n.max(acc));
    data.topography.max_alt = max_elevation;
    for imgx in 0..img_width {
        for imgy in 0..img_height {
            let x = bound_max.x * (imgx as f64 / img_width as f64);
            let y = bound_max.y * (imgy as f64 / img_height as f64);
            let site = Site2D { x, y };
            let altitude = terrain.get_elevation(&site);
            if let Some(altitude) = altitude {
                let color = get_color(altitude);
                let h = ((altitude / max_elevation) * 32768.0) as u16;
                raw_buf.put_pixel(imgx as u32, imgy as u32, image_crate::Luma([h]));
                color_buf.put_pixel(imgx as u32, imgy as u32, image_crate::Rgb(color));
            }
        }
    }
            data.raw_map_512 = raw_buf.clone().into_raw();
            data.color_map_512 = color_buf.clone().into_raw();
            w.set_image_scaled(None::<SharedImage>);
            let i = image::RgbImage::new(data.color_map_512.as_slice(), 512, 512, ColorDepth::Rgb8).unwrap();
            let img = SharedImage::from_image(i).unwrap();
            w.set_image_scaled(Some(img));
            w.redraw();
        }



fn octaved_perlin(
    perlin: &Perlin,
    x: f64,
    y: f64,
    octaves: usize,
    persistence: f64,
    lacunarity: f64,
) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += perlin.get([x * frequency, y * frequency, 0.0]) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    value / max_value
}

fn determine_outlets(
    sites: &Vec<Site2D>,
    base_is_outlet: Vec<bool>,
    start_index: Vec<usize>,
    graph: &EdgeAttributedUndirectedGraph<f64>,
) -> Option<Vec<bool>> {
    let mut queue = start_index
        .into_iter()
        .filter(|i| base_is_outlet[*i])
        .collect::<Vec<_>>();
    if queue.is_empty() {
        return None;
    }
    let mut outlets = vec![false; sites.len()];
    while let Some(i) = queue.pop() {
        if outlets[i] {
            continue;
        }
        outlets[i] = true;
        graph.neighbors_of(i).iter().for_each(|(j, _)| {
            if !outlets[*j] && base_is_outlet[*j] {
                queue.push(*j);
            }
        });
    }

    let is_outlet = outlets.iter().map(|&b| b).collect::<Vec<_>>();
    Some(is_outlet)
}

fn lerp_color(c1: [u8; 3], c2: [u8; 3], prop: f64) -> [u8; 3] {
    [
        (c1[0] as f64 + (c2[0] as f64 - c1[0] as f64) * prop) as u8,
        (c1[1] as f64 + (c2[1] as f64 - c1[1] as f64) * prop) as u8,
        (c1[2] as f64 + (c2[2] as f64 - c1[2] as f64) * prop) as u8,
    ]
}

fn apply_brightness(color: [u8; 3], brightness: f64) -> [u8; 3] {
    [
        (color[0] as f64 * brightness) as u8,
        (color[1] as f64 * brightness) as u8,
        (color[2] as f64 * brightness) as u8,
    ]
}

fn get_surounding_index(props: &Vec<f64>, target: f64) -> (usize, usize) {
    if target <= props[0] {
        (0, 0)
    } else if target >= props[props.len() - 1] {
        (props.len() - 1, props.len() - 1)
    } else {
        let mut index = 1;
        for i in 1..props.len() {
            if target < props[i] {
                index = i;
                break;
            }
        }
        (index - 1, index)
    }
}

fn get_interporated_color(colors: &Vec<[u8; 3]>, elevations: &Vec<f64>, elevation: f64) -> [u8; 3] {
    let (index_a, index_b) = get_surounding_index(elevations, elevation);
    let color_a = colors[index_a];
    let color_b = colors[index_b];
    let prop = (elevation - elevations[index_a]) / (elevations[index_b] - elevations[index_a]);
    lerp_color(color_a, color_b, prop.min(1.0).max(0.0))
}