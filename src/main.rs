use std::fs;

use crate::topo_settings::TopoSettings;
use fltk::app::{modal, Sender};
use fltk::group::Group;
use fltk::image::{Image, PngImage, SharedImage};
use fltk::{prelude::*, *};
use noise::{Billow, Cache, Fbm, MultiFractal, Perlin, Seedable, Simplex};
use noise::utils::{NoiseImage, NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use rand::{Rng, thread_rng};
use std::path::Path;
use rand::rngs::ThreadRng;
use topo_settings::NoiseTypesUi;
use crate::ui::UserInterface;

mod menu_bar;
mod topo_settings;
mod topography_pane;
mod ui;

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
    NoiseChanged
}


fn update_perlin_noise(settings: &TopoSettings) {
    let mut perlin: Fbm<Perlin> = Default::default();
    perlin = perlin
        .set_seed(settings.seed.unwrap() as u32)
        .set_octaves(settings.noise_octaves.unwrap() as usize)
        .set_frequency(settings.noise_frequency.unwrap())
        .set_lacunarity(settings.noise_lacunarity.unwrap());

    if Path::new("example_images/cache.png").exists() {
        fs::remove_file("example_images/cache.png").unwrap();
    }

  let map = PlaneMapBuilder::<Fbm<Perlin>, 2>::new(perlin.clone()).set_size(230, 230).set_is_seamless(false).set_x_bounds(-1.0, 1.0).set_y_bounds(-1.0, 1.0).build();

  map.write_to_file("cache.png");
}

fn update_simplex_noise(settings: &TopoSettings) {
    let mut simplex: Fbm<Simplex> = Default::default();
    simplex = simplex
  .set_seed(settings.seed.unwrap() as u32)
  .set_octaves(settings.noise_octaves.unwrap() as usize)
  .set_frequency(settings.noise_frequency.unwrap())
  .set_lacunarity(settings.noise_lacunarity.unwrap());

  if Path::new("example_images/cache.png").exists() {
      fs::remove_file("example_images/cache.png").unwrap();
  }
  let map = PlaneMapBuilder::<Fbm<Simplex>, 2>::new(simplex.clone()).set_size(230, 230).set_is_seamless(false).set_x_bounds(-1.0, 1.0).set_y_bounds(-1.0, 1.0).build();

  map.write_to_file("cache.png");
}

fn update_billow_noise(settings: &TopoSettings) {
    let mut perlin: Billow<Perlin> = Default::default();
    perlin = perlin
  .set_seed(settings.seed.unwrap() as u32)
  .set_octaves(settings.noise_octaves.unwrap() as usize)
  .set_frequency(settings.noise_frequency.unwrap())
  .set_lacunarity(settings.noise_lacunarity.unwrap());

  if Path::new("example_images/cache.png").exists() {
      fs::remove_file("example_images/cache.png").unwrap();
  }
  let map = PlaneMapBuilder::<Billow<Perlin>, 2>::new(perlin.clone()).set_size(230, 230).set_is_seamless(false).set_x_bounds(-1.0, 1.0).set_y_bounds(-1.0, 1.0).build();

  map.write_to_file("cache.png");
}

fn seed_input_do(w: &mut impl InputExt, topo_settings: &mut TopoSettings) {
    if w.changed() {
        topo_settings.set_seed(Some(w.value().parse().unwrap()));

        match topo_settings.noise_type {
            Some(NoiseTypesUi::Simplex) => {
                update_simplex_noise(topo_settings);
                topo_settings.set_signal(true);
            }
            Some(NoiseTypesUi::Perlin) => {
                update_perlin_noise(topo_settings);
                topo_settings.set_signal(true);
            }
            Some(NoiseTypesUi::BillowPerlin) => {
                update_billow_noise(topo_settings);
                topo_settings.set_signal(true);
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
            topo_settings.set_signal(true);
            println!("{:?}",  topo_settings.noise_changed);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise( topo_settings);
            topo_settings.set_signal(true);
            println!("{:?}", topo_settings.noise_changed);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(& topo_settings);
            topo_settings.set_signal(true);
            println!("{:?}",  topo_settings.noise_changed);
        }
        _ => {}
    };
}

fn aux_choice_do(topo_settings: &mut TopoSettings) {
    match topo_settings.noise_type.unwrap() {
        NoiseTypesUi::Perlin => {
            topo_settings.set_type(Some(NoiseTypesUi::Perlin));
            update_perlin_noise(topo_settings);
            topo_settings.set_signal(true);
        },
        NoiseTypesUi::Simplex => {
            topo_settings.set_type(Some(NoiseTypesUi::Simplex));
            update_simplex_noise(topo_settings);
            topo_settings.set_signal(true);
        },
        NoiseTypesUi::BillowPerlin => {
            topo_settings.set_type(Some(NoiseTypesUi::BillowPerlin));
            update_billow_noise(topo_settings);
            topo_settings.set_signal(true);
        },
    }
}

fn noise_choice_do(w: &mut impl MenuExt, sender: &Sender<Message>) {
    w.add_emit(
        "Simplex",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        sender.clone(),
        Message::SimplexChoice
    );
    w.add_emit(
        "Perlin",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        sender.clone(),
        Message::PerlinChoice
    );
    w.add_emit(
        "Billowed Perlin",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        sender.clone(),
        Message::BillowChoice
    );
}

fn change_noise_type(noise_types_ui: NoiseTypesUi, topo_settings: &mut TopoSettings) {
    topo_settings.set_type(Some(noise_types_ui));
}

fn octaves_input_do(w: &mut impl InputExt, topo_settings: &mut TopoSettings) {
    if w.changed() {
        topo_settings.set_octaves(Some(w.value().parse().unwrap()));

        match topo_settings.noise_type {
            Some(NoiseTypesUi::Simplex) => {
                update_simplex_noise(topo_settings);
                topo_settings.set_signal(true);
            }
            Some(NoiseTypesUi::Perlin) => {
                update_perlin_noise(topo_settings);
                topo_settings.set_signal(true);

            }
            Some(NoiseTypesUi::BillowPerlin) => {
                update_billow_noise(topo_settings);
                topo_settings.set_signal(true);
            }
            _ => {}
        };
    }
}

fn frequency_input_do(w: &mut impl InputExt, topo_settings: &mut TopoSettings) {
    if w.changed() {
        topo_settings.set_frequency(Some(w.value().parse().unwrap()));

        match topo_settings.noise_type {
            Some(NoiseTypesUi::Simplex) => {
                update_simplex_noise(topo_settings);
                topo_settings.set_signal(true);
            }
            Some(NoiseTypesUi::Perlin) => {
                update_perlin_noise(topo_settings);
                topo_settings.set_signal(true);

            }
            Some(NoiseTypesUi::BillowPerlin) => {
                update_billow_noise(topo_settings);
                topo_settings.set_signal(true);
            }
            _ => {}
        };
    }
}

fn lacunarity_input_do(w: &mut impl InputExt, topo_settings: &mut TopoSettings) {
    if w.changed() {
        topo_settings.set_lacunarity(Some(w.value().parse().unwrap()));

        match topo_settings.noise_type {
            Some(NoiseTypesUi::Simplex) => {
                update_simplex_noise(topo_settings);
                topo_settings.set_signal(true);
            }
            Some(NoiseTypesUi::Perlin) => {
                update_perlin_noise(topo_settings);
                topo_settings.set_signal(true);

            }
            Some(NoiseTypesUi::BillowPerlin) => {
                update_billow_noise(topo_settings);
                topo_settings.set_signal(true);
            }
            _ => {}
        };
    }
}


fn main() {

    let mut topo_settings: TopoSettings = TopoSettings {
        seed: Some(4294967295),
        noise_type: Some(NoiseTypesUi::Simplex),
        noise_octaves: Some(8),
        noise_frequency: Some(0.13),
        noise_lacunarity: Some(1.0),
        mountain_pct: 25,
        sea_pct: 5,
        min_height: -50,
        max_height: 1000,
        erosion_cycles: 0,
        noise_changed: false
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

    while app.wait() {

        if let Some(msg) = r.recv() {
            match msg {
                Message::SimplexChoice => { change_noise_type(NoiseTypesUi::Simplex, &mut topo_settings); aux_choice_do(&mut topo_settings); },
                Message::PerlinChoice => {change_noise_type(NoiseTypesUi::Perlin, &mut topo_settings); aux_choice_do(&mut topo_settings);}
                Message::BillowChoice => {change_noise_type(NoiseTypesUi::BillowPerlin, &mut topo_settings); aux_choice_do(&mut topo_settings);}
                Message::SeedRandom => seed_random_do(&mut ui.seed_random_button, &mut ui.seed_input, &mut topo_settings),
                Message::SeedInput => seed_input_do(&mut ui.seed_input, &mut topo_settings),
                Message::OctaveInput => octaves_input_do(&mut ui.noise_octaves_input, &mut topo_settings),
                Message::FreqInput => frequency_input_do(&mut ui.noise_freq_input, &mut topo_settings),
                Message::LacInput => lacunarity_input_do(&mut ui.noise_lacunarity_input, &mut topo_settings),
                _ => {}
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