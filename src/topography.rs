use noise::{Billow, Curve, Fbm, MultiFractal, Perlin, Seedable, Simplex};
use fltk::prelude::{InputExt, MenuExt, ValuatorExt, WidgetExt};
use fltk::app::Sender;
use fltk::{enums, image, menu};
use noise::utils::{ImageRenderer, NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use image_crate::{ImageBuffer, Luma, Pixel};
use fltk::enums::ColorDepth;
use fltk::image::{RgbImage, SharedImage};
use map_range::MapRange;
use crate::{FileData, Message};
use crate::erosion::world::World;
use crate::topo_settings::{NoiseTypesUi, TopoSettings};
use crate::utils::{get_raw_u16, get_raw_u8};

pub const DIMENSIONS: usize = 512;
pub const PREV_DIMENSIONS: usize = 512;

pub const DEFAULT_TOPOSETTINGS: TopoSettings =  TopoSettings {
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

pub fn erode_terrain_preview(file: &mut FileData) {
    let b: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::from_raw(512, 512, file.clone().raw_map_512).unwrap();
    file.raw_map_512 = b.clone().into_raw();
    let (width, height) = b.dimensions();
    let heightmap = b.into_raw();
    let mut erosion_world = World::new(
        heightmap,
        width as usize,
        height as usize,
        file.topography.seed.unwrap() as i16,
    );

    eprintln!("Eroding preview.");
    eprintln!(
        "Total cycle count: {}",
        file.topography.erosion_cycles as i32
    );

    for cycle in 0..(file.topography.erosion_cycles as i32) {
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
    file.eroded_raw_512 = buffer.clone().into_raw();
    apply_color_eroded(file, 512);
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

pub fn apply_color_eroded(file: &mut FileData, size: u16) {
    
    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let data = match size {
        512 => file.eroded_raw_512.clone(),
        8192 => file.eroded_full.clone(),
        _ => vec![]
    };
    let eroded_image: ImageBuffer<Luma<u16>, Vec<u16>> = image_crate::ImageBuffer::from_raw(size as u32, size as u32, data).unwrap();
    let mut map = NoiseMap::new(size as usize, size as usize);

    for x in 0..size {
        for y in 0..size {
            let pixel = eroded_image
                .get_pixel(x as u32, y as u32)
                .channels()
                .first()
                .unwrap();
            let p_i = *pixel as f32;
            let output = p_i.map_range(0.0..32767.0, -1.0..1.0);
            map.set_value(x as usize, y as usize, output as f64);
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
    let i = get_raw_u8(&b);

    match size {
        512 => file.color_eroded_512 = i.to_vec(),
        8192 => file.eroded_full_color = i.to_vec(),
        _ => {}
    };

}

pub fn update_perlin_noise(data: &mut FileData, dimensions: usize) {
    
    let mut perlin: Fbm<Perlin> = Default::default();
    perlin = perlin
        .set_seed(data.topography.seed.unwrap())
        .set_octaves(data.topography.noise_octaves.unwrap() as usize)
        .set_frequency(data.topography.noise_frequency.unwrap())
        .set_lacunarity(data.topography.noise_lacunarity.unwrap());

    let curved = apply_variations_perlin(perlin, data.topography.mountain_pct, data.topography.sea_pct);

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

    let a = get_raw_u8(&renderer);
    let b = get_raw_u16(&map);

    data.raw_map_512 = b.to_vec();
    data.color_map_512 = a.to_vec();

}

pub fn update_simplex_noise(dimensions: usize, data: &mut FileData) {
    
    let mut simplex: Fbm<Simplex> = Default::default();
    simplex = simplex
        .set_seed(data.topography.seed.unwrap())
        .set_octaves(data.topography.noise_octaves.unwrap() as usize)
        .set_frequency(data.topography.noise_frequency.unwrap())
        .set_lacunarity(data.topography.noise_lacunarity.unwrap());

    let curved = apply_variations_simplex(simplex, data.topography.mountain_pct, data.topography.sea_pct);

    // .add_control_point(sealevel, 0.0)
    let map = PlaneMapBuilder::<Curve<f64, Fbm<Simplex>, 2>, 2>::new(curved)
        .set_size(dimensions, dimensions)
        .set_is_seamless(false)
        .set_x_bounds(-1.0, 1.0)
        .set_y_bounds(-1.0, 1.0)
        .build();

    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let renderer = ImageRenderer::new().set_gradient(gradient).render(&map);

    data.raw_map_512 = get_raw_u16(&map).to_vec();
    data.color_map_512 = get_raw_u8(&renderer).to_vec();

}

/// 0 => preview_box_topo (512 not eroded)
/// 1 => preview_erosion_topo (512 eroded)
/// 2 => hydro_preview (8192 eroded)
/// 3 => hydro_mask_preview (8192 water mask)
pub fn update_noise_img(w: &mut impl WidgetExt, data: &FileData, img_type: u8) {
    let map = match img_type {
        0 => { image::RgbImage::new(data.color_map_512.as_slice(), 512, 512, ColorDepth::Rgba8).unwrap() },
        1 => image::RgbImage::new(data.color_eroded_512.as_slice(), 512, 512, ColorDepth::Rgba8).unwrap(),
        _ => RgbImage::new(&[], 512, 512, ColorDepth::Rgba8).unwrap()
    };
    w.set_image_scaled(None::<SharedImage>);
    let img = SharedImage::from_image(map).unwrap();
    w.set_image_scaled(Some(img));
    w.redraw();
}

pub fn update_billow_noise(dimensions: usize, data: &mut FileData) {
    
    let mut perlin: Billow<Perlin> = Default::default();
    perlin = perlin
        .set_seed(data.topography.seed.unwrap())
        .set_octaves(data.topography.noise_octaves.unwrap() as usize)
        .set_frequency(data.topography.noise_frequency.unwrap())
        .set_lacunarity(data.topography.noise_lacunarity.unwrap());

    let curved = apply_variations_billow(perlin, data.topography.mountain_pct, data.topography.sea_pct);

    // .add_control_point(sealevel, 0.0)
    let map = PlaneMapBuilder::<Curve<f64, Billow<Perlin>, 2>, 2>::new(curved)
        .set_size(dimensions, dimensions)
        .set_is_seamless(false)
        .set_x_bounds(-1.0, 1.0)
        .set_y_bounds(-1.0, 1.0)
        .build();

    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let renderer = ImageRenderer::new().set_gradient(gradient).render(&map);

    let a = get_raw_u8(&renderer);
    let b = get_raw_u16(&map);

    data.raw_map_512 = b.to_vec();
    data.color_map_512 = a.to_vec();

}

pub fn noise_choice_do(w: &mut impl MenuExt, sender: &Sender<Message>) {
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

pub fn change_noise_type(noise_types_ui: NoiseTypesUi, data: &mut FileData) {
    data.topography.set_type(Some(noise_types_ui));
}

pub fn octaves_input_do(w: &mut impl InputExt, data: &mut FileData) {
    data.topography.set_octaves(Some(w.value().parse::<u32>().unwrap()));

    match data.topography.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(DIMENSIONS, data);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(data, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(DIMENSIONS, data);
        }
        _ => {}
    };
}

pub fn frequency_input_do(w: &mut impl InputExt, data: &mut FileData) {
    println!("Has input changed? {}", w.changed());
    data.topography.set_frequency(Some(w.value().parse::<f64>().unwrap()));

    match data.topography.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(DIMENSIONS, data);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(data, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(DIMENSIONS, data);
        }
        _ => {}
    };
}

pub fn lacunarity_input_do(w: &mut impl InputExt, data: &mut FileData) {
    data.topography.set_lacunarity(Some(w.value().parse().unwrap()));

    match data.topography.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(DIMENSIONS, data);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(data, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(DIMENSIONS, data);
        }
        _ => {}
    };
}

pub fn mtn_slider_do(w: &mut impl ValuatorExt, data: &mut FileData) {
    data.topography.set_mtn_pct(w.value());
    match data.topography.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(DIMENSIONS, data);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(data, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(DIMENSIONS, data);
        }
        _ => {}
    };
}

pub fn sea_slider_do(w: &mut impl ValuatorExt, data: &mut FileData) {
    data.topography.set_sea_pct(w.value());
    match data.topography.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(DIMENSIONS, data);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(data, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(DIMENSIONS, data);
        }
        _ => {}
    };
}

pub fn cycle_input_do(w: &mut impl ValuatorExt, data: &mut FileData) {
    data.topography.set_cycles(w.value() as u64);
}
