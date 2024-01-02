use std::fs;
use std::io::stdout;
use std::ops::Index;

use map_range::MapRange;
use crate::topo_settings::TopoSettings;
use fltk::app::{modal, Sender};
use fltk::group::Group;
use fltk::image::{Image, PngImage, SharedImage};
use fltk::{prelude::*, *};
use noise::{Billow, Cache, Curve, Fbm, MultiFractal, NoiseFn, OpenSimplex, Perlin, Seedable, Simplex};
use noise::utils::{ImageRenderer, NoiseImage, NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use rand::{Rng, thread_rng};
use std::path::Path;
use image_crate::{DynamicImage, GenericImage, ImageBuffer, Luma, Pixel, Rgb, Rgba};
use imageproc::integral_image::ArrayData;
use rand::rngs::ThreadRng;
use topo_settings::NoiseTypesUi;
use crate::erosion::world::World;
use crate::ui::UserInterface;

mod topo_settings;
mod ui;
mod erosion;

const DIMENSIONS: usize = 256;

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
    ErodeButton
}

fn erode_terrain_preview(w: &mut impl WidgetExt, topo_settings: &TopoSettings) {
        let img = image_crate::io::Reader::open("example_images/raw.png").unwrap().decode().unwrap().into_luma16();
        let (width, height) = img.dimensions();
        let heightmap = img.into_raw();
        let mut erosion_world = World::new(heightmap, width as usize, height as usize, topo_settings.seed.unwrap() as i16);

        for cycle in 0..topo_settings.erosion_cycles as i32 {
            erosion_world.erode(width as usize);
        }
        let eroded_preview: Vec<u16> = erosion_world.map.heightmap.iter().map(|x| (x.height * 255.0) as u16).collect();
        let buffer: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::from_raw(width, height, eroded_preview).unwrap();
        buffer.save("example_images/eroded_cache.png");


    }

fn update_preview_ero(w: &mut impl WidgetExt) {
    apply_color_eroded();
    w.set_image_scaled(None::<SharedImage>);
    let img = SharedImage::load("example_images/eroded_cache.png").unwrap();
    w.set_image_scaled(Some(img));
    w.redraw();
}

fn apply_variations_perlin(source: Fbm<Perlin>, mtn: f64, sea: f64) -> Curve<f64, Fbm<Perlin>, 2>
{
    let sealevel = sea / 100.0;
    let mountainlevel = mtn / 100.0;

    let curved: Curve<f64, Fbm<Perlin>, 2> = Curve::new(source)
        .add_control_point(1.0, mountainlevel)
        .add_control_point(0.999, mountainlevel)
        .add_control_point(-0.999, 0.0 - sealevel)
        .add_control_point(-1.0,  0.0 - sealevel);

    return curved;
}

fn apply_variations_simplex(source: Fbm<Simplex>, mtn: f64, sea: f64) -> Curve<f64, Fbm<Simplex>, 2>
{
    let sealevel = sea / 100.0;
    let mountainlevel = mtn / 100.0;

    let curved: Curve<f64, Fbm<Simplex>, 2> = Curve::new(source)
        .add_control_point(1.0, mountainlevel)
        .add_control_point(0.999, mountainlevel)
        .add_control_point(-0.999, 0.0 - sealevel)
        .add_control_point(-1.0,  0.0 - sealevel);

    return curved;
}

fn apply_variations_billow(source: Billow<Perlin>, mtn: f64, sea: f64) -> Curve<f64, Billow<Perlin>, 2>
{
    let sealevel = sea / 100.0;
    let mountainlevel = mtn / 100.0;

    let curved: Curve<f64, Billow<Perlin>, 2> = Curve::new(source)
        .add_control_point(1.0, mountainlevel)
        .add_control_point(0.999, mountainlevel)
        .add_control_point(-0.999, 0.0 - sealevel)
        .add_control_point(-1.0,  0.0 - sealevel);

    return curved;
}

fn apply_color_eroded() {
    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let eroded_image = image_crate::open("example_images/eroded_cache.png").unwrap().into_luma16();

    let mut map = NoiseMap::new(DIMENSIONS, DIMENSIONS);

    for x in 0..DIMENSIONS {
        for y in 0..DIMENSIONS {
            let pixel = eroded_image.get_pixel(x as u32, y as u32).channels().first().unwrap();
            let p_i = *pixel as f32;
            let output = p_i.map_range(0.0..32767.0, -1.0..-0.01);
            map.set_value(x, y, output as f64);
        }
    }
    let mut r = ImageRenderer::new().set_gradient(gradient).render(&map);

    if Path::new("example_images/eroded_cache.png").exists() {
        fs::remove_file("example_images/eroded_cache.png").unwrap();
    }

    r.write_to_file("eroded_cache.png");

}

fn update_perlin_noise(settings: &TopoSettings) {
    let mut perlin: Fbm<Perlin> = Default::default();
    perlin = perlin
        .set_seed(settings.seed.unwrap())
        .set_octaves(settings.noise_octaves.unwrap() as usize)
        .set_frequency(settings.noise_frequency.unwrap())
        .set_lacunarity(settings.noise_lacunarity.unwrap());

    let curved= apply_variations_perlin(perlin, settings.mountain_pct, settings.sea_pct);

    // .add_control_point(sealevel, 0.0)

    if Path::new("example_images/cache.png").exists() {
        fs::remove_file("example_images/cache.png").unwrap();
    }
  let map = PlaneMapBuilder::<Curve<f64, Fbm<Perlin>, 2>, 2>::new(curved).set_size(DIMENSIONS, DIMENSIONS).set_is_seamless(false).set_x_bounds(-1.0, 1.0).set_y_bounds(-1.0, 1.0).build();

    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let renderer = ImageRenderer::new().set_gradient(gradient).render(&map);

    // for x in 0..DIMENSIONS {
    //     for y in 0..DIMENSIONS {
    //         println!("{}", map.get_value(x, y));
    //     }
    // }

    map.write_to_file("raw.png");
    renderer.write_to_file("cache.png");
}

fn update_simplex_noise(settings: &TopoSettings) {
    let mut simplex: Fbm<Simplex> = Default::default();
    simplex = simplex
  .set_seed(settings.seed.unwrap() as u32)
  .set_octaves(settings.noise_octaves.unwrap() as usize)
  .set_frequency(settings.noise_frequency.unwrap())
  .set_lacunarity(settings.noise_lacunarity.unwrap());

    let curved= apply_variations_simplex(simplex, settings.mountain_pct, settings.sea_pct);

    // .add_control_point(sealevel, 0.0)

    if Path::new("example_images/cache.png").exists() {
        fs::remove_file("example_images/cache.png").unwrap();
    }
    let map = PlaneMapBuilder::<Curve<f64, Fbm<Simplex>, 2>, 2>::new(curved).set_size(DIMENSIONS, DIMENSIONS).set_is_seamless(false).set_x_bounds(-1.0, 1.0).set_y_bounds(-1.0, 1.0).build();

    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let renderer = ImageRenderer::new().set_gradient(gradient).render(&map);

    map.write_to_file("raw.png");
    renderer.write_to_file("cache.png");
}

fn update_noise_img(w: &mut impl WidgetExt) {
    w.set_image_scaled(None::<SharedImage>);
    let img = SharedImage::load("example_images/cache.png").unwrap();
    w.set_image_scaled(Some(img));
    w.redraw();
}

fn update_billow_noise(settings: &TopoSettings) {
    let mut perlin: Billow<Perlin> = Default::default();
    perlin = perlin
  .set_seed(settings.seed.unwrap())
  .set_octaves(settings.noise_octaves.unwrap() as usize)
  .set_frequency(settings.noise_frequency.unwrap())
  .set_lacunarity(settings.noise_lacunarity.unwrap());

    let curved= apply_variations_billow(perlin, settings.mountain_pct, settings.sea_pct);

    // .add_control_point(sealevel, 0.0)

    if Path::new("example_images/cache.png").exists() {
        fs::remove_file("example_images/cache.png").unwrap();
    }
    let map = PlaneMapBuilder::<Curve<f64, Billow<Perlin>, 2>, 2>::new(curved).set_size(DIMENSIONS, DIMENSIONS).set_is_seamless(false).set_x_bounds(-1.0, 1.0).set_y_bounds(-1.0, 1.0).build();

    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let renderer = ImageRenderer::new().set_gradient(gradient).render(&map);

    map.write_to_file("raw.png");
    renderer.write_to_file("cache.png");
}

fn seed_input_do(w: &mut impl InputExt, topo_settings: &mut TopoSettings) {
    if w.changed() {
        topo_settings.set_seed(Some(w.value().parse().unwrap()));

        match topo_settings.noise_type {
            Some(NoiseTypesUi::Simplex) => {
                update_simplex_noise(topo_settings);
            }
            Some(NoiseTypesUi::Perlin) => {
                update_perlin_noise(topo_settings);
            }
            Some(NoiseTypesUi::BillowPerlin) => {
                update_billow_noise(topo_settings);
            }
            _ => {}
        };

    }
}

fn seed_random_do(w: &mut impl ButtonExt, seed_box: &mut impl InputExt, topo_settings: &mut TopoSettings) {
    let mut rng = thread_rng();
    let seed: u32 = rng.gen_range(u32::MIN..u32::MAX);
    seed_box.set_value(&format!("{}", seed));
    topo_settings.set_seed(Some(seed));

    match  topo_settings.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise( topo_settings);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise( topo_settings);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(topo_settings);
        }
        _ => {}
    };
}

fn aux_choice_do(topo_settings: &mut TopoSettings) {
    match topo_settings.noise_type.unwrap() {
        NoiseTypesUi::Perlin => {
            topo_settings.set_type(Some(NoiseTypesUi::Perlin));
            update_perlin_noise(topo_settings);
        },
        NoiseTypesUi::Simplex => {
            topo_settings.set_type(Some(NoiseTypesUi::Simplex));
            update_simplex_noise(topo_settings);
        },
        NoiseTypesUi::BillowPerlin => {
            topo_settings.set_type(Some(NoiseTypesUi::BillowPerlin));
            update_billow_noise(topo_settings);
        },
    }
}

fn noise_choice_do(w: &mut impl MenuExt, sender: &Sender<Message>) {
    w.add_emit(
        "Simplex",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        *sender,
        Message::SimplexChoice
    );
    w.add_emit(
        "Perlin",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        *sender,
        Message::PerlinChoice
    );
    w.add_emit(
        "Billowed Perlin",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        *sender,
        Message::BillowChoice
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
                update_simplex_noise(topo_settings);
            }
            Some(NoiseTypesUi::Perlin) => {
                update_perlin_noise(topo_settings);

            }
            Some(NoiseTypesUi::BillowPerlin) => {
                update_billow_noise(topo_settings);
            }
            _ => {}
        };
    }


fn frequency_input_do(w: &mut impl InputExt, topo_settings: &mut TopoSettings) {
        println!("Has input changed? {}", w.changed());
        topo_settings.set_frequency(Some(w.value().parse::<f64>().unwrap()));

        match topo_settings.noise_type {
            Some(NoiseTypesUi::Simplex) => {
                update_simplex_noise(topo_settings);
            }
            Some(NoiseTypesUi::Perlin) => {
                update_perlin_noise(topo_settings);

            }
            Some(NoiseTypesUi::BillowPerlin) => {
                update_billow_noise(topo_settings);
            }
            _ => {}
        };
    }

fn lacunarity_input_do(w: &mut impl InputExt, topo_settings: &mut TopoSettings) {
        topo_settings.set_lacunarity(Some(w.value().parse().unwrap()));

        match topo_settings.noise_type {
            Some(NoiseTypesUi::Simplex) => {
                update_simplex_noise(topo_settings);
            }
            Some(NoiseTypesUi::Perlin) => {
                update_perlin_noise(topo_settings);

            }
            Some(NoiseTypesUi::BillowPerlin) => {
                update_billow_noise(topo_settings);
            }
            _ => {}
        };
}

fn mtn_slider_do(w: &mut impl ValuatorExt, topo_settings: &mut TopoSettings) {
    topo_settings.set_mtn_pct(w.value());
    match topo_settings.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(topo_settings);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(topo_settings);

        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(topo_settings);
        }
        _ => {}
    };
}
fn sea_slider_do(w: &mut impl ValuatorExt, topo_settings: &mut TopoSettings) {
    topo_settings.set_sea_pct(w.value());
    match topo_settings.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(topo_settings);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(topo_settings);

        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(topo_settings);
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
    let mut win = ui.main_window.clone();

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

    while app.wait() {

        if let Some(msg) = r.recv() {
            match msg {
                Message::SimplexChoice => { change_noise_type(NoiseTypesUi::Simplex, &mut topo_settings); aux_choice_do(&mut topo_settings); update_noise_img(&mut ui.preview_box_topo); println!("{:?}", &topo_settings);},
                Message::PerlinChoice => {change_noise_type(NoiseTypesUi::Perlin, &mut topo_settings); aux_choice_do(&mut topo_settings); update_noise_img(&mut ui.preview_box_topo); println!("{:?}", &topo_settings);}
                Message::BillowChoice => {change_noise_type(NoiseTypesUi::BillowPerlin, &mut topo_settings); aux_choice_do(&mut topo_settings); update_noise_img(&mut ui.preview_box_topo); println!("{:?}", &topo_settings);}
                Message::SeedRandom => { seed_random_do(&mut ui.seed_random_button, &mut ui.seed_input, &mut topo_settings); update_noise_img(&mut ui.preview_box_topo); println!("{:?}", &topo_settings);},
                Message::SeedInput => { seed_input_do(&mut ui.seed_input, &mut topo_settings); update_noise_img(&mut ui.preview_box_topo); println!("{:?}", &topo_settings);},
                Message::OctaveInput => { octaves_input_do(&mut ui.noise_octaves_input, &mut topo_settings); update_noise_img(&mut ui.preview_box_topo); println!("{:?}", &topo_settings);},
                Message::FreqInput => { frequency_input_do(&mut ui.noise_freq_input, &mut topo_settings); update_noise_img(&mut ui.preview_box_topo); println!("{:?}", &topo_settings);},
                Message::LacInput => { lacunarity_input_do(&mut ui.noise_lacunarity_input, &mut topo_settings); update_noise_img(&mut ui.preview_box_topo); println!("{:?}", &topo_settings);},
                Message::MtnSlider => { mtn_slider_do(&mut ui.high_elev_slider, &mut topo_settings); update_noise_img(&mut ui.preview_box_topo); println!("{:?}", &topo_settings); },
                Message::SeaSlider => {sea_slider_do(&mut ui.sea_elev_slider, &mut topo_settings); update_noise_img(&mut ui.preview_box_topo); println!("{:?}", &topo_settings);},
                Message::CycleInput => { cycle_input_do(&mut ui.erosion_cycles_input, &mut topo_settings) },
                Message::ErodeButton => { erode_terrain_preview(&mut ui.topo_ero_preview, &mut topo_settings); update_preview_ero(&mut ui.preview_erosion_topo); }
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