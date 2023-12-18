use std::fs;

use crate::topo_settings::TopoSettings;
use fltk::app::modal;
use fltk::group::Group;
use fltk::image::{Image, PngImage, SharedImage};
use fltk::{prelude::*, *};
use noise::{Billow, Cache, Fbm, MultiFractal, Perlin, Seedable, Simplex};
use noise::utils::{NoiseImage, NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use rand::Rng;
use std::path::Path;
use topo_settings::NoiseTypesUi;

mod menu_bar;
mod topo_settings;
mod topography_pane;
mod ui;

static mut NOISE_CHANGED: bool = false;

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

fn main() {
    let mut rng = rand::thread_rng();

    let mut topo_settings = TopoSettings {
        seed: Some(4294967295),
        noise_type: Some(NoiseTypesUi::Simplex),
        noise_octaves: Some(8),
        noise_frequency: Some(0.13),
        noise_lacunarity: Some(2.0),
        mountain_pct: 25,
        sea_pct: 5,
        min_height: -50,
        max_height: 1000,
        erosion_cycles: 0,
    };

    let app = app::App::default();
    let mut ui = ui::UserInterface::make_window();
    let mut win = ui.main_window.clone();

    ui.seed_input.set_callback(move |x| {
        if x.changed() {
            topo_settings.seed = Some(x.clone().value().parse().unwrap());

            match topo_settings.noise_type.clone() {
                Some(NoiseTypesUi::Simplex) => {
                    update_simplex_noise(&topo_settings);
                    unsafe {
                        NOISE_CHANGED = true;
                    }
                }
                Some(NoiseTypesUi::Perlin) => {
                    update_perlin_noise(&topo_settings);
                    unsafe {
                        NOISE_CHANGED = true;
                    }
                }
                Some(NoiseTypesUi::BillowPerlin) => {
                    update_billow_noise(&topo_settings);
                    unsafe {
                        NOISE_CHANGED = true;
                    }
                }
                _ => {}
            };
        }
    });

    ui.seed_random_button.set_callback(move |x1| {
        let seed: u32 = rng.gen_range(std::u32::MIN..std::u32::MAX);
        ui.seed_input.set_value(&*format!("{}", seed));
        topo_settings.seed = Some(seed);

        match topo_settings.noise_type.clone() {
            Some(NoiseTypesUi::Simplex) => {
                update_simplex_noise(&topo_settings);
                unsafe {
                    NOISE_CHANGED = true;
                }
            }
            Some(NoiseTypesUi::Perlin) => {
                update_perlin_noise(&topo_settings);
                unsafe {
                    NOISE_CHANGED = true;
                }
            }
            Some(NoiseTypesUi::BillowPerlin) => {
                update_billow_noise(&topo_settings);
                unsafe {
                    NOISE_CHANGED = true;
                }
            }
            _ => {}
        };
    });

    ui.noise_choice.add_choice("Perlin");
    ui.noise_choice.add_choice("Billowed Perlin");
    ui.noise_choice.add_choice("Simplex");


  ui.noise_choice.set_callback(move |b| {
      match b.value() {
          0 => {
              topo_settings.noise_type = Some(NoiseTypesUi::Perlin);
              update_perlin_noise(&topo_settings);
              unsafe {
                  NOISE_CHANGED = true;
              }
          }
          1 => {
              topo_settings.noise_type = Some(NoiseTypesUi::BillowPerlin);
              update_billow_noise(&topo_settings);
              unsafe {
                  NOISE_CHANGED = true;
              }
          }
          2 => {
              topo_settings.noise_type = Some(NoiseTypesUi::Simplex);
              update_simplex_noise(&topo_settings);
              unsafe {
                  NOISE_CHANGED = true;
              }
          }
          _ => {}
      }
  });;

    topo_settings.noise_type = match ui.noise_choice.value() {
        0 => Some(NoiseTypesUi::Perlin),
        1 => Some(NoiseTypesUi::BillowPerlin),
        2 => Some(NoiseTypesUi::Simplex),
        _ => None,
    };

    ui.noise_octaves_input.set_callback(move |x2| {
        if x2.changed() {
            topo_settings.noise_octaves = Some(x2.value().parse().unwrap());

            match topo_settings.noise_type {
                Some(NoiseTypesUi::Simplex) => {
                    update_simplex_noise(&topo_settings);
                    unsafe {
                        NOISE_CHANGED = true;
                    }
                }
                Some(NoiseTypesUi::Perlin) => {
                    update_perlin_noise(&topo_settings);
                    unsafe {
                        NOISE_CHANGED = true;
                    }
                }
                Some(NoiseTypesUi::BillowPerlin) => {
                    update_billow_noise(&topo_settings);
                    unsafe {
                        NOISE_CHANGED = true;
                    }
                }
                _ => {}
            };
        }
    });

    ui.noise_freq_input.set_callback(move |x3| {
        if x3.changed() {
            topo_settings.noise_frequency = Some(x3.value().parse().unwrap());

            match topo_settings.noise_type {
                Some(NoiseTypesUi::Simplex) => {
                    update_simplex_noise(&topo_settings);
                    unsafe {
                        NOISE_CHANGED = true;
                    }
                }
                Some(NoiseTypesUi::Perlin) => {
                    update_perlin_noise(&topo_settings);
                    unsafe {
                        NOISE_CHANGED = true;
                    }
                }
                Some(NoiseTypesUi::BillowPerlin) => {
                    update_billow_noise(&topo_settings);
                    unsafe {
                        NOISE_CHANGED = true;
                    }
                }
                _ => {}
            };
        }
    });

    ui.noise_lacunarity_input.set_callback(move |x4| {
        if x4.changed() {
            topo_settings.noise_lacunarity = Some(x4.value().parse().unwrap());

            match topo_settings.noise_type {
                Some(NoiseTypesUi::Simplex) => {
                    update_simplex_noise(&topo_settings);
                    unsafe {
                        NOISE_CHANGED = true;
                    }
                }
                Some(NoiseTypesUi::Perlin) => {
                    update_perlin_noise(&topo_settings);
                    unsafe {
                        NOISE_CHANGED = true;
                    }
                }
                Some(NoiseTypesUi::BillowPerlin) => {
                    update_billow_noise(&topo_settings);
                    unsafe {
                        NOISE_CHANGED = true;
                    }
                }
                _ => {}
            };
        }
    });

    ui.min_height_input.set_callback(move |x5| {
        if x5.changed() {
            topo_settings.min_height = x5.value() as i32;
        }
    });

    ui.max_height_input.set_callback(move |x6| {
        if x6.changed() {
            topo_settings.max_height = x6.value() as i32;
        }
    });

    ui.erosion_cycles_input.set_callback(move |x7| {
        if x7.changed() {
            topo_settings.erosion_cycles = x7.value() as u64;
        }
    });

    while app.wait() {
        unsafe {
            if NOISE_CHANGED == true {
              println!("{:?}", topo_settings);
                let img = SharedImage::load("example_images/cache.png").expect("Error loading file.");
                ui.preview_box_topo.set_image_scaled(Some(img));
              ui.preview_box_topo.redraw();
                NOISE_CHANGED = false;
            }
        }
    }
}
