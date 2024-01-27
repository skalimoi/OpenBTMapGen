use crate::topo_settings::{TopoSettings};
use fltk::app::Sender;
use image_crate::imageops::FilterType;
use map_range::MapRange;
use fltk::image::SharedImage;
use fltk::{prelude::*, *};
use image_crate::{ImageBuffer, Luma, Pixel, DynamicImage};
use noise::utils::{ImageRenderer, NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::{Billow, Curve, Fbm, MultiFractal, Perlin, Seedable, Simplex};
use rand::{thread_rng, Rng};

use std::sync::Arc;
use fltk::window::{GlutWindow};
use three_d::{AmbientLight, Camera, ClearState, ColorMaterial, Context, CpuMaterial, CpuMesh, CpuTexture, DirectionalLight, FromCpuMaterial, Gm, LightingModel, Mat4, Mesh, PhysicalMaterial, RenderTarget, Srgba, Terrain, Vec3, Viewport};
use std::default::Default;

use colorgrad::Color;


use crate::erosion::world::{Vec2, World};
use topo_settings::NoiseTypesUi;
use crate::utils::get_height;
use crate::weather::{Climate, GenData, HumidDry, koppen_afam, koppen_as, koppen_aw, koppen_bsh, koppen_bsk, koppen_bwh, koppen_bwk, koppen_cfa, koppen_cfb, koppen_cfc, koppen_cwa, koppen_cwb, koppen_cwc, koppen_dfa, koppen_dfb, koppen_dfc, koppen_dsc, koppen_et};
use crate::weather_settings::WeatherSettings;
use crate::WeatherVisualization::Init;

mod erosion;
mod topo_settings;
mod ui;
mod utils;
mod weather;
mod weather_settings;

const DIMENSIONS: usize = 512;
const PREV_DIMENSIONS: usize = 512;

#[derive(Copy, Clone)]
enum WeatherVisualization {
    Wind,
    Temperature,
    Pressure,
    Humidity,
    Init
}
#[derive(Copy, Clone)]
enum Message {
    PerlinChoice,
    BillowChoice,
    SimplexChoice,
    SeedInput,
    SeedRandom,
    OctaveInput,
    LacInput,
    FreqInput,
    MtnSlider,
    SeaSlider,
    CycleInput,
    ErodeButton,
    FullPreview,
    TurnViewRight,
    TurnViewLeft,
    GenWeather,
    WeatherSeedInput,
    WeatherClimInput,
    WeatherLatInput,
    WeatherGridSize,
    WeatherRandomSeed,
    ViewTemperature,
    ViewHumidity,
    ViewPressure,
    ViewWind,
    DaySlider
}

struct ViewState {
    mode: WeatherVisualization,
    hour: u32
}

fn set_hour(w: &mut impl ValuatorExt, state: &mut ViewState) {
    state.hour = w.value() as u32;
}

/// Outputs previous state for comparison
fn set_view_state(options: &mut ViewState, state: WeatherVisualization) -> WeatherVisualization {
    let previous_state = options.mode;
    options.mode = state;
    previous_state
}
fn update_grid_at_time(hour: u32, v_type: WeatherVisualization, grid_vector: &mut Vec<GenData>, cube_vector: &mut [Gm<Mesh, ColorMaterial>]) {
    use ordered_float::OrderedFloat;


    for component in grid_vector.as_slice() {
        let cube = &mut cube_vector[component.index.0 as usize + 16 *(component.index.1 as usize + 6 * component.index.2 as usize)];
        let range = match hour {
            0 => 0..24,
            _ => (((24 * hour) - 24) as usize)..((24 * hour) as usize)
        };
        match v_type {
            Init => {},
            WeatherVisualization::Wind => {},
            WeatherVisualization::Temperature => {
                let median = (component.temperature[range.clone()].iter().sum::<OrderedFloat<f64>>().0 as isize) / range.clone().len() as isize;
                let color = match median {
                    -60..=-10 => Color::from_rgba8(30, 92, 179, 10),
                    -11..=-1 => Color::from_rgba8(4, 161, 230, 10),
                    0..=5 => Color::from_rgba8(102, 204, 206, 10),
                    6..=10 => Color::from_rgba8(192, 229, 136, 10),
                    11..=15 => Color::from_rgba8(204, 230, 75, 10),
                    16..=20 => Color::from_rgba8(243, 240, 29, 10),
                    21..=25 => Color::from_rgba8(248, 157, 14, 10),
                    26..=30 => Color::from_rgba8(219, 30, 38, 10),
                    31..=90 => Color::from_rgba8(164, 38, 44, 10),
                    _ => Color::from_rgba8(255, 255, 255, 10)
                };
                cube.material.color = Srgba::new(color.to_linear_rgba_u8().0, color.to_linear_rgba_u8().1, color.to_linear_rgba_u8().2, color.to_linear_rgba_u8().3);
            },
            WeatherVisualization::Pressure => {
                let median = (component.pressure[range.clone()].iter().sum::<OrderedFloat<f64>>().0 as usize) / range.clone().len();
                println!("{}", median);
            },
            WeatherVisualization::Humidity => {
                let median = (component.humidity[range.clone()].iter().sum::<OrderedFloat<f64>>().0 as usize) / range.clone().len();
                println!("{}", median);
            }
        }
    }
}

fn hydro_preview_do(topo_settings: &TopoSettings) {
    match topo_settings.noise_type.unwrap() {
        NoiseTypesUi::Perlin => update_perlin_noise(topo_settings, PREV_DIMENSIONS),
        NoiseTypesUi::Simplex => update_simplex_noise(topo_settings, PREV_DIMENSIONS),
        NoiseTypesUi::BillowPerlin => update_billow_noise(topo_settings, PREV_DIMENSIONS),
    }
}

fn erode_terrain_preview(topo_settings: &TopoSettings) {
    let img = image_crate::io::Reader::open("example_images/raw.png")
        .unwrap()
        .decode()
        .unwrap()
        .into_luma16();
    let (width, height) = img.dimensions();
    let heightmap = img.into_raw();
    let mut erosion_world = World::new(
        heightmap,
        width as usize,
        height as usize,
        topo_settings.seed.unwrap() as i16,
    );

    eprintln!("Eroding preview.");
    eprintln!(
        "Total cycle count: {}",
        topo_settings.erosion_cycles as i32
    );

    for cycle in 0..(topo_settings.erosion_cycles as i32) {
        erosion_world.erode(width as usize, 1.0);
        if cycle == 0 {
            eprintln!("0")
        } else {
            eprint!("..{}", cycle)
        }
        // update_text_buffer(w, cycle);
    }
    let eroded_preview: Vec<u16> = erosion_world
        .map
        .heightmap
        .iter()
        .map(|x| (x.height * 255.0) as u16)
        .collect();
    let buffer: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_raw(width, height, eroded_preview).unwrap();
    buffer.save("example_images/eroded_cache.png").expect("Error saving eroded image.");
}

fn erode_heightmap_full(topo_settings: &TopoSettings) {
    

    let mut discharge_map = vec![0; 8192 * 8192];

    let img = image_crate::io::Reader::open("example_images/raw.png")
        .unwrap()
        .decode()
        .unwrap()
        .into_luma16();
    let heightmap = img.into_raw();
    let mut erosion_world_1 = World::new(
        heightmap,
        512,
        512,
        topo_settings.seed.unwrap() as i16,
    );
    
    //////////////////////
    //    SIZE: 512     //
    //////////////////////

    eprint!("Eroding full preview.");
    eprintln!("Size: 512");
    eprintln!(
        "Total cycle count: {}",
        topo_settings.erosion_cycles as i32
    );

    for cycle in 0..(topo_settings.erosion_cycles as i32) {
        erosion_world_1.erode(512, 1.0);
        if cycle == 0 {
            eprintln!("0")
        } else {
            eprint!("..{}", cycle)
        }
    }
    
    //////////////////////
    //   SIZE: 1024     //
    //////////////////////
    
    let eroded_preview_to_be_1024: Vec<u16> = erosion_world_1
        .map
        .heightmap
        .iter()
        .map(|x| (x.height * 255.0) as u16)
        .collect();
        
    let buffer: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_raw(512, 512, eroded_preview_to_be_1024).unwrap();
    
    let resampled_vec_1024 = image_crate::imageops::resize(&buffer, 1024, 1024, FilterType::Lanczos3).into_raw();
    
    let mut erosion_world_2 = World::new(
        resampled_vec_1024,
        1024,
        1024,
        topo_settings.seed.unwrap() as i16,
    );
    
    eprintln!("Size: 1024");
    eprintln!(
        "Total cycle count: {}",
        topo_settings.erosion_cycles as i32
    );
    
    for cycle in 0..(topo_settings.erosion_cycles as i32) {
        erosion_world_2.erode(1024, 1.0);
        if cycle == 0 {
            eprintln!("0")
        } else {
            eprint!("..{}", cycle)
        }
    }
    
    //////////////////////
    //   SIZE: 2048     //
    //////////////////////
    
    let eroded_preview_to_be_2048: Vec<u16> = erosion_world_2
        .map
        .heightmap
        .iter()
        .map(|x| (x.height * 255.0) as u16)
        .collect();
    
    let buffer: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_raw(1024, 1024, eroded_preview_to_be_2048).unwrap();
    
    let resampled_vec_2048 = image_crate::imageops::resize(&buffer, 2048, 2048, FilterType::Lanczos3).into_raw();
    
    let mut erosion_world_3 = World::new(
        resampled_vec_2048,
        2048,
        2048,
        topo_settings.seed.unwrap() as i16,
    );
    
    eprintln!("Size: 2048");
    eprintln!(
        "Total cycle count: {}",
        topo_settings.erosion_cycles as i32
    );
    
    for cycle in 0..(topo_settings.erosion_cycles as i32) {
        erosion_world_3.erode(2048, 1.0);
        if cycle == 0 {
            eprintln!("0")
        } else {
            eprint!("..{}", cycle)
        }
        // update_text_buffer(w, cycle);
    }
    
    //////////////////////
    //   SIZE: 4096     //
    //////////////////////
    
    let eroded_preview_to_be_4096: Vec<u16> = erosion_world_3
        .map
        .heightmap
        .iter()
        .map(|x| (x.height * 255.0) as u16)
        .collect();
    
    let buffer: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_raw(2048, 2048, eroded_preview_to_be_4096).unwrap();
    
    let resampled_vec_4096 = image_crate::imageops::resize(&buffer, 4096, 4096, FilterType::Lanczos3).into_raw();
    
    let mut erosion_world_4 = World::new(
        resampled_vec_4096,
        4096,
        4096,
        topo_settings.seed.unwrap() as i16,
    );
    
    eprintln!("Size: 4096");
    eprintln!(
        "Total cycle count: {}",
        topo_settings.erosion_cycles as i32
    );
    
    for cycle in 0..(topo_settings.erosion_cycles as i32) {
        erosion_world_4.erode(4096, 1.0);
        if cycle == 0 {
            eprintln!("0")
        } else {
            eprint!("..{}", cycle)
        }
        // update_text_buffer(w, cycle);
    }
    
    //////////////////////
    //   SIZE: 8192     //
    //////////////////////
    
    let eroded_preview_to_be_8192: Vec<u16> = erosion_world_4
        .map
        .heightmap
        .iter()
        .map(|x| (x.height * 255.0) as u16)
        .collect();
    
    let buffer: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_raw(4096, 4096, eroded_preview_to_be_8192).unwrap();
    
    let resampled_vec_8192 = image_crate::imageops::resize(&buffer, 8192, 8192, FilterType::Lanczos3).into_raw();
    
    let mut erosion_world_5 = World::new(
        resampled_vec_8192,
        8192,
        8192,
        topo_settings.seed.unwrap() as i16,
    );
    
    eprintln!("Size: 8192");
    eprintln!(
        "Total cycle count: {}",
        topo_settings.erosion_cycles as i32
    );
    
    for cycle in 0..(topo_settings.erosion_cycles as i32) {
        erosion_world_5.erode(8192, 1.0);
        if cycle == 0 {
            eprintln!("0")
        } else {
            eprint!("..{}", cycle)
        }
        // update_text_buffer(w, cycle);
    }
    
    /////////////////
    
    for i in 0..discharge_map.len() {
        let pos = Vec2::new(i as f64 % 8192.0, (i / 8192) as f64);
        discharge_map[i] = ((erosion_world_5.map.discharge(pos) + 1.0) * 0.5 * 255.0) as u8;
    }
    let eroded_preview: Vec<u16> = erosion_world_5
        .map
        .heightmap
        .iter()
        .map(|x| (x.height * 255.0) as u16)
        .collect();
    let buffer: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_raw(8192, 8192, eroded_preview).unwrap();
    buffer.save("example_images/eroded_cache_full.png").unwrap();

    let discharge_buffer: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_raw(8192, 8192, discharge_map.clone()).unwrap();
    discharge_buffer
        .save("example_images/hydro_mask_full.png".to_string().as_str())
        .unwrap();

    let proc_water =
        image_crate::io::Reader::open("example_images/hydro_mask_full.png".to_string().as_str())
            .unwrap()
            .decode()
            .unwrap();
    let mut gray = proc_water.to_luma8();
    imageproc::contrast::stretch_contrast_mut(&mut gray, 130, 200);
    gray.save("example_images/hydro_mask_full.png".to_string().as_str())
        .unwrap();

    apply_color_hydro_full();
}

fn update_preview_ero(w: &mut impl WidgetExt) {
    apply_color_eroded();
    w.set_image_scaled(None::<SharedImage>);
    let img = SharedImage::load("example_images/eroded_cache_prev.png").unwrap();
    w.set_image_scaled(Some(img));
    w.redraw();
}

fn update_hydro_prev(w: &mut impl WidgetExt, left: bool) {
    apply_color_eroded();
    w.set_image_scaled(None::<SharedImage>);

    match left {
        true => {
            let img = SharedImage::load("example_images/colored_full.png").unwrap();
            w.set_image_scaled(Some(img));
            w.redraw();
        }
        false => {
            let img = SharedImage::load("example_images/hydro_mask_full.png").unwrap();
            w.set_image_scaled(Some(img));
            w.redraw();
        }
    };
}

fn apply_variations_perlin(source: Fbm<Perlin>, mtn: f64, sea: f64) -> Curve<f64, Fbm<Perlin>, 2> {
    let sealevel = sea / 100.0;
    let mountainlevel = mtn / 100.0;

    let curved: Curve<f64, Fbm<Perlin>, 2> = Curve::new(source)
        .add_control_point(1.0, mountainlevel)
        .add_control_point(0.999, mountainlevel)
        .add_control_point(-0.999, 0.0 - sealevel)
        .add_control_point(-1.0, 0.0 - sealevel);

    curved
}

fn apply_variations_simplex(
    source: Fbm<Simplex>,
    mtn: f64,
    sea: f64,
) -> Curve<f64, Fbm<Simplex>, 2> {
    let sealevel = sea / 100.0;
    let mountainlevel = mtn / 100.0;

    let curved: Curve<f64, Fbm<Simplex>, 2> = Curve::new(source)
        .add_control_point(1.0, mountainlevel)
        .add_control_point(0.999, mountainlevel)
        .add_control_point(-0.999, 0.0 - sealevel)
        .add_control_point(-1.0, 0.0 - sealevel);

    curved
}

fn apply_variations_billow(
    source: Billow<Perlin>,
    mtn: f64,
    sea: f64,
) -> Curve<f64, Billow<Perlin>, 2> {
    let sealevel = sea / 100.0;
    let mountainlevel = mtn / 100.0;

    let curved: Curve<f64, Billow<Perlin>, 2> = Curve::new(source)
        .add_control_point(1.0, mountainlevel)
        .add_control_point(0.999, mountainlevel)
        .add_control_point(-0.999, 0.0 - sealevel)
        .add_control_point(-1.0, 0.0 - sealevel);

    curved
}

fn apply_color_eroded() {
    
    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let eroded_image = image_crate::open("example_images/eroded_cache.png")
        .unwrap()
        .into_luma16();

    let mut map = NoiseMap::new(DIMENSIONS, DIMENSIONS);

    for x in 0..DIMENSIONS {
        for y in 0..DIMENSIONS {
            let pixel = eroded_image
                .get_pixel(x as u32, y as u32)
                .channels()
                .first()
                .unwrap();
            let p_i = *pixel as f32;
            let output = p_i.map_range(0.0..32767.0, -1.0..1.0);
            map.set_value(x, y, output as f64);
        }
    }


    let mut r = ImageRenderer::new()
        .set_gradient(gradient)
        .set_light_elevation(Default::default())
        .set_light_color(Default::default())
        .set_light_azimuth(Default::default())
        .set_light_brightness(Default::default())
        .set_light_contrast(Default::default())
        .set_light_intensity(Default::default());
    r.disable_light();
    let b = r.render(&map);
    utils::write_to_file(&b, "eroded_cache_prev.png");
}

fn apply_color_hydro_full() {
    
    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let eroded_image = image_crate::open("example_images/eroded_cache_full.png")
        .unwrap()
        .into_luma16();

    let mut map = NoiseMap::new(8192, 8192);

    for x in 0..8192 {
        for y in 0..8192 {
            let pixel = eroded_image
                .get_pixel(x as u32, y as u32)
                .channels()
                .first()
                .unwrap();
            let p_i = *pixel as f32;
            let output = p_i.map_range(0.0..32767.0, -1.0..1.0);
            map.set_value(x, y, output as f64);
        }
    }
    let r = ImageRenderer::new().set_gradient(gradient).render(&map);

    utils::write_to_file(&r, "colored_full.png");
}

fn update_perlin_noise(settings: &TopoSettings, dimensions: usize) {
    
    let mut perlin: Fbm<Perlin> = Default::default();
    perlin = perlin
        .set_seed(settings.seed.unwrap())
        .set_octaves(settings.noise_octaves.unwrap() as usize)
        .set_frequency(settings.noise_frequency.unwrap())
        .set_lacunarity(settings.noise_lacunarity.unwrap());

    let curved = apply_variations_perlin(perlin, settings.mountain_pct, settings.sea_pct);

    // .add_control_point(sealevel, 0.0)

    let map = PlaneMapBuilder::<Curve<f64, Fbm<Perlin>, 2>, 2>::new(curved)
        .set_size(dimensions, dimensions)
        .set_is_seamless(false)
        .set_x_bounds(-1.0, 1.0)
        .set_y_bounds(-1.0, 1.0)
        .build();

    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let renderer = ImageRenderer::new().set_gradient(gradient).render(&map);

    // for x in 0..DIMENSIONS {
    //     for y in 0..DIMENSIONS {
    //         println!("{}", map.get_value(x, y));
    //     }
    // }

    utils::write_map_to_file(&map, "raw.png");
    utils::write_to_file(&renderer, "cache.png");
}

fn update_simplex_noise(settings: &TopoSettings, dimensions: usize) {
    
    let mut simplex: Fbm<Simplex> = Default::default();
    simplex = simplex
        .set_seed(settings.seed.unwrap())
        .set_octaves(settings.noise_octaves.unwrap() as usize)
        .set_frequency(settings.noise_frequency.unwrap())
        .set_lacunarity(settings.noise_lacunarity.unwrap());

    let curved = apply_variations_simplex(simplex, settings.mountain_pct, settings.sea_pct);

    // .add_control_point(sealevel, 0.0)
    let map = PlaneMapBuilder::<Curve<f64, Fbm<Simplex>, 2>, 2>::new(curved)
        .set_size(dimensions, dimensions)
        .set_is_seamless(false)
        .set_x_bounds(-1.0, 1.0)
        .set_y_bounds(-1.0, 1.0)
        .build();

    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let renderer = ImageRenderer::new().set_gradient(gradient).render(&map);

    utils::write_map_to_file(&map, "raw.png");
    utils::write_to_file(&renderer, "cache.png");
}

fn update_noise_img(w: &mut impl WidgetExt) {
    w.set_image_scaled(None::<SharedImage>);
    let img = SharedImage::load("example_images/cache.png").unwrap();
    w.set_image_scaled(Some(img));
    w.redraw();
}

fn update_billow_noise(settings: &TopoSettings, dimensions: usize) {
    
    let mut perlin: Billow<Perlin> = Default::default();
    perlin = perlin
        .set_seed(settings.seed.unwrap())
        .set_octaves(settings.noise_octaves.unwrap() as usize)
        .set_frequency(settings.noise_frequency.unwrap())
        .set_lacunarity(settings.noise_lacunarity.unwrap());

    let curved = apply_variations_billow(perlin, settings.mountain_pct, settings.sea_pct);

    // .add_control_point(sealevel, 0.0)
    let map = PlaneMapBuilder::<Curve<f64, Billow<Perlin>, 2>, 2>::new(curved)
        .set_size(dimensions, dimensions)
        .set_is_seamless(false)
        .set_x_bounds(-1.0, 1.0)
        .set_y_bounds(-1.0, 1.0)
        .build();

    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let renderer = ImageRenderer::new().set_gradient(gradient).render(&map);

    utils::write_map_to_file(&map, "raw.png");
    utils::write_to_file(&renderer, "cache.png");
}

fn weather_seed_do(w: &mut impl InputExt, weather_settings: &mut WeatherSettings) {
    if w.changed() {
        weather_settings.set_seed(w.value().parse().unwrap());
    }
}

fn weather_lat_do(w: &mut impl InputExt, weather_settings: &mut WeatherSettings) {
    if w.changed() {
        weather_settings.set_latitude(w.value().parse().unwrap());
    }
}

fn weather_grid_size_do(w: &mut impl InputExt, weather_settings: &mut WeatherSettings) {
    if w.changed() {
        weather_settings.set_grid_size(w.value().parse().unwrap());
    }
}

fn weather_climate_do(w: &mut impl MenuExt, weather_settings: &mut WeatherSettings, climates: &[Climate; 18]) {

        let choice_name = w.choice().unwrap();
        let mut climate_choice: Climate = Climate {
            name: "Blank".to_string(),
            general_type: 'x',
            second_type: 'x',
            third_type: 'x',
            spring: Default::default(),
            winter: (HumidDry::Humid, Default::default()),
            fall: Default::default(),
            summer: (HumidDry::Humid, Default::default()),
            diurnal_range: Default::default(),
        };
        for climate in climates {
            if choice_name.as_str() == climate.name.as_str() {
                climate_choice = climate.clone();
                break
            } else {
                continue
            }
        }
        weather_settings.set_climate(climate_choice);

}

fn seed_input_do(w: &mut impl InputExt, topo_settings: &mut TopoSettings) {
    if w.changed() {
        topo_settings.set_seed(Some(w.value().parse().unwrap()));

        match topo_settings.noise_type {
            Some(NoiseTypesUi::Simplex) => {
                update_simplex_noise(topo_settings, DIMENSIONS);
            }
            Some(NoiseTypesUi::Perlin) => {
                update_perlin_noise(topo_settings, DIMENSIONS);
            }
            Some(NoiseTypesUi::BillowPerlin) => {
                update_billow_noise(topo_settings, DIMENSIONS);
            }
            _ => {}
        };
    }
}

fn weather_seed_random_do(
    _w: &mut impl ButtonExt,
    seed_box: &mut impl InputExt,
    weather_settings: &mut WeatherSettings,
) {
    let mut rng = thread_rng();
    let seed: u32 = rng.gen_range(u32::MIN..u32::MAX);
    seed_box.set_value(&format!("{}", seed));
    weather_settings.set_seed(seed);
}

fn seed_random_do(
    _w: &mut impl ButtonExt,
    seed_box: &mut impl InputExt,
    topo_settings: &mut TopoSettings,
) {
    let mut rng = thread_rng();
    let seed: u32 = rng.gen_range(u32::MIN..u32::MAX);
    seed_box.set_value(&format!("{}", seed));
    topo_settings.set_seed(Some(seed));

    match topo_settings.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(topo_settings, DIMENSIONS);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(topo_settings, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(topo_settings, DIMENSIONS);
        }
        _ => {}
    };
}

fn aux_choice_do(topo_settings: &mut TopoSettings) {
    match topo_settings.noise_type.unwrap() {
        NoiseTypesUi::Perlin => {
            topo_settings.set_type(Some(NoiseTypesUi::Perlin));
            update_perlin_noise(topo_settings, DIMENSIONS);
        }
        NoiseTypesUi::Simplex => {
            topo_settings.set_type(Some(NoiseTypesUi::Simplex));
            update_simplex_noise(topo_settings, DIMENSIONS);
        }
        NoiseTypesUi::BillowPerlin => {
            topo_settings.set_type(Some(NoiseTypesUi::BillowPerlin));
            update_billow_noise(topo_settings, DIMENSIONS);
        }
    }
}

fn noise_choice_do(w: &mut impl MenuExt, sender: &Sender<Message>) {
    w.add_emit(
        "Simplex",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        *sender,
        Message::SimplexChoice,
    );
    w.add_emit(
        "Perlin",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        *sender,
        Message::PerlinChoice,
    );
    w.add_emit(
        "Billowed Perlin",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        *sender,
        Message::BillowChoice,
    );
}

fn change_noise_type(noise_types_ui: NoiseTypesUi, topo_settings: &mut TopoSettings) {
    topo_settings.set_type(Some(noise_types_ui));
}

fn octaves_input_do(w: &mut impl InputExt, topo_settings: &mut TopoSettings) {
    // DEBUG
    println!("Oct value: {}", w.value());
    //
    topo_settings.set_octaves(Some(w.value().parse::<u32>().unwrap()));

    match topo_settings.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(topo_settings, DIMENSIONS);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(topo_settings, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(topo_settings, DIMENSIONS);
        }
        _ => {}
    };
}

fn frequency_input_do(w: &mut impl InputExt, topo_settings: &mut TopoSettings) {
    println!("Has input changed? {}", w.changed());
    topo_settings.set_frequency(Some(w.value().parse::<f64>().unwrap()));

    match topo_settings.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(topo_settings, DIMENSIONS);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(topo_settings, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(topo_settings, DIMENSIONS);
        }
        _ => {}
    };
}

fn lacunarity_input_do(w: &mut impl InputExt, topo_settings: &mut TopoSettings) {
    topo_settings.set_lacunarity(Some(w.value().parse().unwrap()));

    match topo_settings.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(topo_settings, DIMENSIONS);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(topo_settings, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(topo_settings, DIMENSIONS);
        }
        _ => {}
    };
}

fn mtn_slider_do(w: &mut impl ValuatorExt, topo_settings: &mut TopoSettings) {
    topo_settings.set_mtn_pct(w.value());
    match topo_settings.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(topo_settings, DIMENSIONS);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(topo_settings, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(topo_settings, DIMENSIONS);
        }
        _ => {}
    };
}
fn sea_slider_do(w: &mut impl ValuatorExt, topo_settings: &mut TopoSettings) {
    topo_settings.set_sea_pct(w.value());
    match topo_settings.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(topo_settings, DIMENSIONS);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(topo_settings, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(topo_settings, DIMENSIONS);
        }
        _ => {}
    };
}

fn cycle_input_do(w: &mut impl ValuatorExt, topo_settings: &mut TopoSettings) {
    topo_settings.set_cycles(w.value() as u64);
}


fn main() {
    //TODO check for adequate default values since results are not optimal

    let mut view_state = ViewState {
        mode: Init,
        hour: 0
    };

    let climates: [Climate; 18] = [koppen_cfa(), koppen_cfb(), koppen_cfc(), koppen_dfb(), koppen_dfc(), koppen_dfa(), koppen_cwc(), koppen_cwb(), koppen_cwa(), koppen_et(), koppen_afam(), koppen_as(), koppen_aw(), koppen_dsc(), koppen_bsh(), koppen_bsk(), koppen_bwh(), koppen_bwk()];

    let mut grid: Vec<GenData> = vec![];

    let mut topo_settings: TopoSettings = TopoSettings {
        seed: Some(42949),
        noise_type: Some(NoiseTypesUi::BillowPerlin),
        noise_octaves: Some(20),
        noise_frequency: Some(3.0),
        noise_lacunarity: Some(4.0),
        mountain_pct: 25.0,
        sea_pct: 5.0,
        min_height: -50,
        max_height: 1000,
        erosion_cycles: 0,
    };

    let mut weather_settings: WeatherSettings = WeatherSettings {
        seed: None,
        koppen: None,
        latitude: 0,
        grid_size: 16,
    };

    // Initialize grid
    for x in 0..weather_settings.grid_size {
        for y in 0..6 {
            for z in 0..weather_settings.grid_size {
                let _h = 0;
                let component = GenData {
                    index: (x as i32, y, z as i32),
                    temperature: vec![],
                    altitude: 0.0,
                    pressure: vec![],
                    humidity: vec![],
                    wind: vec![],
                    td: vec![],
                };
                grid.push(component);
            }
        }
    }

    let (s, r) = app::channel::<Message>();

    let app = app::App::default();
    let mut ui = ui::UserInterface::make_window();
    let mut win = ui.main_window.clone();
    let (x, y, w, h) = (ui.weather_preview.x(), ui.weather_preview.y(), ui.weather_preview.w(), ui.weather_preview.h());
    // create gl window
    let mut gl_widget = GlutWindow::new(x, y, w, h, None);
    // let mut gl_win = GlutWindow::new(x, y, w, h, None);
    gl_widget.set_mode(enums::Mode::Opengl3);
    ui.weather_preview.add(&gl_widget);
    win.end();
    gl_widget.end();
    win.show();
    gl_widget.show();

    ui.turn_right_vis.set_image(Some(SharedImage::load("icons/turn_right.png").unwrap()));
    ui.turn_left_vis.set_image(Some(SharedImage::load("icons/turn_left.png").unwrap()));
    ui.wind_mode.set_image(Some(SharedImage::load("icons/wind.png").unwrap()));
    ui.temperature_mode.set_image(Some(SharedImage::load("icons/temperature.png").unwrap()));
    ui.humidity_mode.set_image(Some(SharedImage::load("icons/humidity.png").unwrap()));
    ui.pressure_mode.set_image(Some(SharedImage::load("icons/pressure.png").unwrap()));

    /////////////

    let viewport = Viewport {
        x: (0-x) + 30, // don't know why tf it must be like this in order for the viewport to be aligned with the widget
        y: 0-y,
        width: w as u32,
        height: h as u32,
    };


    let gl = unsafe {
        three_d::context::Context::from_loader_function(|s| gl_widget.get_proc_address(s) as *const _)
    };

    // and this is three_d context
    let context = Context::from_gl_context(Arc::new(gl)).unwrap();

    let mut camera = Camera::new_orthographic(viewport,
                                              Vec3::new(0.0, 256.0, 0.0),
                                              Vec3::new(255.0, 128.0, 255.0),
                                              Vec3::new(0.0, 1.0, 0.0),
                                              768.0,
                                              0.1,
                                              10000.0
    );

    let m: CpuTexture = CpuTexture::default();

    let cpu_mat = CpuMaterial {
        name: "".to_string(),
        albedo: Default::default(),
        albedo_texture: Some(m),
        metallic: 0.0,
        roughness: 0.0,
        occlusion_metallic_roughness_texture: None,
        metallic_roughness_texture: None,
        occlusion_strength: 0.0,
        occlusion_texture: None,
        normal_scale: 0.0,
        normal_texture: None,
        emissive: Default::default(),
        emissive_texture: Default::default(),
        alpha_cutout: None,
        lighting_model: LightingModel::Phong,
        index_of_refraction: 0.0,
        transmission: 0.0,
        transmission_texture: None,
    };

    // three_d_asset::io::load(&["example_images/eroded_cache.png"]).unwrap().deserialize("").unwrap();

    let terrain_material = PhysicalMaterial::new_opaque(&context, &cpu_mat);

    let heightmap_opt_dyn = DynamicImage::new_luma16(512, 512);

    let heightmap_opt = heightmap_opt_dyn.to_luma16();

    let ambient = AmbientLight::new(&context, 0.4, Srgba::WHITE);
    let directional = DirectionalLight::new(&context, 0.4, Srgba::WHITE, &Vec3::new(0.0, -1.0, 100.0));
    let mut terrain = Terrain::new(
        &context,
        terrain_material.clone(),
        Arc::new(
            move |x, y| {
                *heightmap_opt.get_pixel(x as u32, y as u32).channels().first().unwrap() as f32 * 0.01
            }
        ),
        512.0,
        1.0,
        three_d::prelude::Vec2::new(255.0, 255.0)
    );

    let mut mesh_v: Vec<Gm<Mesh, ColorMaterial>> = vec![];

    //// RANDOMIZER ////

    let mut rng = thread_rng();

    ////           ////


    for x in 0..weather_settings.grid_size {
        for y in 0..6 {
            for z in 0..weather_settings.grid_size {
                let color: (u8, u8, u8) = (rng.gen_range(0..256) as u8, rng.gen_range(0..256) as u8, rng.gen_range(0..256) as u8);
                let mut cube = Gm::new(
                    Mesh::new(&context, &CpuMesh::cube()),
                    ColorMaterial::from_cpu_material(&context,
                        &CpuMaterial {
                            albedo: Srgba {
                                r: color.0,
                                g: color.1,
                                b: color.2,
                                a: 10,
                            },
                            ..Default::default()
                        },
                    )
                );
                cube.set_transformation(Mat4::from_translation(Vec3::new(32.0 * x as f32, 32.0 * (y + 6) as f32, 32.0 * z as f32)) * Mat4::from_scale(32.0));
                mesh_v.push(cube);
            }
        }
    }

    let mut frame = 0;

    /////////////

    for climate in &climates {
        ui.weather_type.add_choice(climate.name.as_str());
    }

    ui.wind_mode.emit(s, Message::ViewWind);

    ui.humidity_mode.emit(s, Message::ViewHumidity);

    ui.temperature_mode.emit(s, Message::ViewTemperature);

    ui.pressure_mode.emit(s, Message::ViewPressure);

    ui.day_vis_slider.emit(s, Message::DaySlider);

    ui.seed_random_button.emit(s, Message::SeedRandom);

    ui.seed_input.emit(s, Message::SeedInput);

    noise_choice_do(&mut ui.noise_choice, &s);

    ui.noise_octaves_input.emit(s, Message::OctaveInput);

    ui.noise_freq_input.emit(s, Message::FreqInput);

    ui.noise_lacunarity_input.emit(s, Message::LacInput);

    ui.high_elev_slider.emit(s, Message::MtnSlider);

    ui.sea_elev_slider.emit(s, Message::SeaSlider);

    ui.erosion_cycles_input.emit(s, Message::CycleInput);

    ui.erode_terrain_button.emit(s, Message::ErodeButton);

    ui.generate_hydro_prev.emit(s, Message::FullPreview);

    ui.turn_right_vis.emit(s, Message::TurnViewRight);

    ui.turn_left_vis.emit(s, Message::TurnViewLeft);

    ui.weather_seed_input.emit(s, Message::WeatherSeedInput);

    ui.weather_noise_random_seed.emit(s, Message::WeatherRandomSeed);

    ui.weather_type.emit(s, Message::WeatherClimInput);

    ui.latitude_input.emit(s, Message::WeatherLatInput);

    ui.grid_size_input.emit(s, Message::WeatherGridSize);

    ui.generate_weather_button.emit(s, Message::GenWeather);

    let target = *camera.target();

    let camera_y = camera.position().y;

    while app.wait() {
        if let Some(msg) = r.recv() {
            match msg {
                Message::SimplexChoice => {
                    change_noise_type(NoiseTypesUi::Simplex, &mut topo_settings);
                    aux_choice_do(&mut topo_settings);
                    update_noise_img(&mut ui.preview_box_topo);
                    println!("{:?}", &topo_settings);
                }
                Message::PerlinChoice => {
                    change_noise_type(NoiseTypesUi::Perlin, &mut topo_settings);
                    aux_choice_do(&mut topo_settings);
                    update_noise_img(&mut ui.preview_box_topo);
                    println!("{:?}", &topo_settings);
                }
                Message::BillowChoice => {
                    change_noise_type(NoiseTypesUi::BillowPerlin, &mut topo_settings);
                    aux_choice_do(&mut topo_settings);
                    update_noise_img(&mut ui.preview_box_topo);
                    println!("{:?}", &topo_settings);
                }
                Message::SeedRandom => {
                    seed_random_do(
                        &mut ui.seed_random_button,
                        &mut ui.seed_input,
                        &mut topo_settings,
                    );
                    update_noise_img(&mut ui.preview_box_topo);
                    println!("{:?}", &topo_settings);
                }
                Message::SeedInput => {
                    seed_input_do(&mut ui.seed_input, &mut topo_settings);
                    update_noise_img(&mut ui.preview_box_topo);
                    println!("{:?}", &topo_settings);
                }
                Message::OctaveInput => {
                    octaves_input_do(&mut ui.noise_octaves_input, &mut topo_settings);
                    update_noise_img(&mut ui.preview_box_topo);
                    println!("{:?}", &topo_settings);
                }
                Message::FreqInput => {
                    frequency_input_do(&mut ui.noise_freq_input, &mut topo_settings);
                    update_noise_img(&mut ui.preview_box_topo);
                    println!("{:?}", &topo_settings);
                }
                Message::LacInput => {
                    lacunarity_input_do(&mut ui.noise_lacunarity_input, &mut topo_settings);
                    update_noise_img(&mut ui.preview_box_topo);
                    println!("{:?}", &topo_settings);
                }
                Message::MtnSlider => {
                    mtn_slider_do(&mut ui.high_elev_slider, &mut topo_settings);
                    update_noise_img(&mut ui.preview_box_topo);
                    println!("{:?}", &topo_settings);
                }
                Message::SeaSlider => {
                    sea_slider_do(&mut ui.sea_elev_slider, &mut topo_settings);
                    update_noise_img(&mut ui.preview_box_topo);
                    println!("{:?}", &topo_settings);
                }
                Message::CycleInput => {
                    cycle_input_do(&mut ui.erosion_cycles_input, &mut topo_settings)
                }
                Message::ErodeButton => {
                    erode_terrain_preview(&mut topo_settings);
                    update_preview_ero(&mut ui.preview_erosion_topo);
                }
                Message::FullPreview => {
                    hydro_preview_do(&topo_settings);
                    erode_heightmap_full(&mut topo_settings);
                    update_hydro_prev(&mut ui.hydro_preview, true);
                    update_hydro_prev(&mut ui.hydro_mask_preview, false);
                }
                Message::TurnViewRight => {
                    camera.rotate_around_with_fixed_up(&target, 300.0, 0.0);
                    let camera_act_pos = Vec3::new(camera.position().x, camera_y, camera.position().z);
                    camera.set_view(camera_act_pos, target, Vec3::new(0.0, 1.0, 0.0));
                    // println!("pos: {:?}, target: {:?}, up: {:?}", camera.position(), camera.target(), camera.up());
                }
                Message::TurnViewLeft => {
                    camera.rotate_around_with_fixed_up(&target, -300.0, 0.0);
                    let camera_act_pos = Vec3::new(camera.position().x, camera_y, camera.position().z);
                    camera.set_view(camera_act_pos, target, Vec3::new(0.0, 1.0, 0.0));
                }
                Message::WeatherSeedInput => {
                    weather_seed_do(&mut ui.weather_seed_input, &mut weather_settings);
                }
                Message::WeatherLatInput => {
                    weather_lat_do(&mut ui.latitude_input, &mut weather_settings);
                }
                Message::WeatherGridSize => {
                    weather_grid_size_do(&mut ui.grid_size_input, &mut weather_settings);
                }
                Message::WeatherClimInput => {
                    weather_climate_do(&mut ui.weather_type, &mut weather_settings, &climates);
                    println!("Weather Info: {:?}", weather_settings);
                }
                Message::WeatherRandomSeed => {
                    weather_seed_random_do(&mut ui.weather_noise_random_seed, &mut ui.weather_seed_input, &mut weather_settings);
                }
                Message::GenWeather => {

                    let noise: Fbm<Perlin> = Fbm::new(weather_settings.seed.unwrap());
                    let map = image_crate::open("example_images/eroded_cache.png").unwrap().into_luma16();
                    let map2 = map.clone();
                    let terrain_map = Terrain::new(
                        &context,
                        terrain_material.clone(),
                        Arc::new(
                            move |x, y| {
                                *map2.get_pixel(x as u32, y as u32).channels().first().unwrap() as f32 * 0.01
                            }
                        ),
                        512.0,
                        1.0,
                        three_d::prelude::Vec2::new(255.0, 255.0)
                    );
                    terrain = terrain_map;
                    let component_size = 512.0 / weather_settings.grid_size as f64;

                    for component in grid.as_mut_slice() {
                                let area = heightmap_opt_dyn.crop_imm(component_size as u32 * component.index.0 as u32, component_size as u32 * component.index.2 as u32, component_size as u32, component_size as u32).to_luma16();
                                let h = match component.index.1 {
                                    0 => get_height(&area, topo_settings.max_height as f64),
                                    _ => 4000 * component.index.1 as u16,
                                };
                                component.altitude = h as f64;
                                println!("ALTITUDE: {}", h);
                                let gen = GenData::gen_year_data(weather_settings.latitude as i32, component.altitude, component.index, noise.clone(), weather_settings.koppen.clone().unwrap());

                                component.humidity = gen.humidity;
                                component.pressure = gen.pressure;
                                component.td = gen.td;
                                component.temperature = gen.temperature;
                                component.wind = gen.wind;
                                println!("GENDATA ALTITUDE: {}", component.altitude);

                    }


                    println!("Finished.");
                },
                Message::ViewHumidity => {
                    set_view_state(&mut view_state, WeatherVisualization::Humidity);
                    update_grid_at_time(view_state.hour, view_state.mode, &mut grid, &mut mesh_v);
                },
                Message::ViewPressure => {
                    set_view_state(&mut view_state, WeatherVisualization::Pressure);
                    update_grid_at_time(view_state.hour, view_state.mode, &mut grid, &mut mesh_v);
                },
                Message::ViewTemperature => {
                    set_view_state(&mut view_state, WeatherVisualization::Temperature);
                    update_grid_at_time(view_state.hour, view_state.mode, &mut grid, &mut mesh_v);
                },
                Message::ViewWind => {
                    set_view_state(&mut view_state, WeatherVisualization::Wind);
                    update_grid_at_time(view_state.hour, view_state.mode, &mut grid, &mut mesh_v);
                },
                Message::DaySlider => {
                    set_hour(&mut ui.day_vis_slider, &mut view_state);
                    update_grid_at_time(view_state.hour, view_state.mode, &mut grid, &mut mesh_v);
                }
            }
        }
        {
            gl_widget.make_current();
            context.set_viewport(viewport);
            let rt = RenderTarget::screen(&context, viewport.width, viewport.height);
            rt
                // Clear color and depth of the render target
                .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
                // Render the triangle with the per vertex colors defined at construction
                .render(&camera, &terrain, &[&directional, &ambient]);
            for x in 0..weather_settings.grid_size as usize {
                for y in 0..6 {
                    for z in 0..weather_settings.grid_size as usize {
                        rt.render(&camera, &mesh_v[x + (weather_settings.grid_size as usize) *(y + 6 * z)], &[&directional, &ambient]);
                    }
                }
            }

            frame += 1;
            // app::sleep(0.10);
            gl_widget.redraw();
        }
    }
}


