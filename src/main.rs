use crate::topo_settings::TopoSettings;
use fltk::app::Sender;
use image_crate::imageops::FilterType;
use map_range::MapRange;
use std::ops::Add;

use fltk::image::SharedImage;
use fltk::{prelude::*, *};
use image_crate::{ImageBuffer, Luma, Pixel, DynamicImage};
use noise::utils::{ImageRenderer, NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::{Billow, Curve, Fbm, MultiFractal, Perlin, Seedable, Simplex};
use rand::{thread_rng, Rng};
use std::path::Path;

use crate::erosion::world::{Vec2, World};
use topo_settings::NoiseTypesUi;

mod erosion;
mod topo_settings;
mod ui;
mod utils;

const DIMENSIONS: usize = 512;
const PREV_DIMENSIONS: usize = 512;

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
}

fn hydro_preview_do(topo_settings: &TopoSettings) {
    match topo_settings.noise_type.unwrap() {
        NoiseTypesUi::Perlin => update_perlin_noise(topo_settings, PREV_DIMENSIONS),
        NoiseTypesUi::Simplex => update_simplex_noise(topo_settings, PREV_DIMENSIONS),
        NoiseTypesUi::BillowPerlin => update_billow_noise(topo_settings, PREV_DIMENSIONS),
    }
}

fn erode_terrain_preview(w: &mut impl InputExt, topo_settings: &TopoSettings) {
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
        (topo_settings.erosion_cycles as i32)
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
    buffer.save("example_images/eroded_cache.png");
}

fn erode_heightmap_full(w: &mut impl InputExt, topo_settings: &TopoSettings) {
    use std::fs;

    let mut discharge_map = vec![0; (8192 * 8192)];

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
        (topo_settings.erosion_cycles as i32)
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
        (topo_settings.erosion_cycles as i32)
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
        (topo_settings.erosion_cycles as i32)
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
        (topo_settings.erosion_cycles as i32)
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
        (topo_settings.erosion_cycles as i32)
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
    let img = SharedImage::load("example_images/eroded_cache.png").unwrap();
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

    return curved;
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

    return curved;
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

    return curved;
}

fn apply_color_eroded() {
    use std::fs;
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
    utils::write_to_file(&b, "eroded_cache.png");
}

fn update_text_buffer(w: &mut impl InputExt, cycle: i32) {
    let mut prev = w.value();
    let buf = prev.add(&*format!("..{}", cycle));
    w.set_value(buf.as_str());
}

fn apply_color_hydro_full() {
    use std::fs;
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
    use std::fs;
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
    use std::fs;
    let mut simplex: Fbm<Simplex> = Default::default();
    simplex = simplex
        .set_seed(settings.seed.unwrap() as u32)
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
    use std::fs;
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

    let (s, r) = app::channel::<Message>();

    let app = app::App::default();
    let mut ui = ui::UserInterface::make_window();
    let _win = ui.main_window.clone();

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
                    erode_terrain_preview(&mut ui.console_output_ero, &mut topo_settings);
                    update_preview_ero(&mut ui.preview_erosion_topo);
                }
                Message::FullPreview => {
                    hydro_preview_do(&topo_settings);
                    erode_heightmap_full(&mut ui.console_output_ero, &mut topo_settings);
                    update_hydro_prev(&mut ui.hydro_preview, true);
                    update_hydro_prev(&mut ui.hydro_mask_preview, false);
                }
            }
        }
    }
}

//
// unsafe {
// if TOPO_SETTINGS.noise_changed {
// println!("{:?}", TOPO_SETTINGS);
// ui.preview_box_topo.set_image_scaled(Some(img));
// ui.preview_box_topo.redraw();
// TOPO_SETTINGS.set_signal(false);
// }
// }

// ui.min_height_input.set_callback(move |x5| {
//     if x5.changed() {
//         topo_settings.min_height = x5.value() as i32;
//     }
// });
//
// ui.max_height_input.set_callback(move |x6| {
//     if x6.changed() {
//         topo_settings.max_height = x6.value() as i32;
//     }
// });
//
// ui.erosion_cycles_input.set_callback(move |x7| {
//
//     if x7.changed() {
//         topo_settings.erosion_cycles = x7.value() as u64;
//     }
// });
