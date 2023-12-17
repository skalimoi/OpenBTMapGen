use std::fs;
use std::path::Path;
use fltk::{prelude::*, *};
use fltk::app::modal;
use fltk::group::Group;
use fltk::image::{Image, PngImage, SharedImage};
use noise::{Billow, Cache, Fbm, MultiFractal, Perlin, Seedable, Simplex};
use noise::utils::{NoiseImage, NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use rand::Rng;
use crate::topo_settings::TopoSettings;
use topo_settings::NoiseTypesUi;

mod ui;
mod menu_bar;
mod topography_pane;
mod topo_settings;

fn main() {

    let mut simplex: Fbm<Simplex> = Default::default();
    let mut perlin: Fbm<Perlin> = Default::default();
    let mut billow: Billow<Perlin> = Default::default();

    let mut map: NoiseMap = Default::default();

    let mut rng = rand::thread_rng();

    let mut topo_settings = TopoSettings {
        seed: Some(10000000000000000000),
        noise_type: Some(NoiseTypesUi::Simplex),
        noise_octaves: Some(8),
        noise_frequency: Some(1),
        noise_lacunarity: Some(1),
        mountain_pct: 25,
        sea_pct: 5,
        min_height: -50,
        max_height: 1000,
        erosion_cycles: 0,
    };

    let tab_active: Group;

    let app = app::App::default();
    let mut ui = ui::UserInterface::make_window();
    let mut win = ui.main_window.clone();

    ui.seed_input.set_callback(move |x| {
        if x.changed() {
            topo_settings.seed = Some(x.value().parse::<u64>().unwrap());

            match topo_settings.noise_type.clone() {
                Some(NoiseTypesUi::Simplex) =>
                    {
                        &mut simplex.clone().set_seed(topo_settings.seed.unwrap() as u32);
                    },
                Some(NoiseTypesUi::Perlin) => {
                    &mut perlin.clone().set_seed(topo_settings.seed.unwrap() as u32);
                },
                Some(NoiseTypesUi::BillowPerlin) => { &mut billow.clone().set_seed(topo_settings.seed.unwrap() as u32); },
                _ => {}
            };


        }
    });

    ui.seed_random_button.set_callback(move |x1| {
        let seed: u64 = rng.gen_range(10000000000000000000..18446744073709551615);
        ui.seed_input.set_value(&*format!("{}", seed));
        topo_settings.seed = Some(ui.seed_input.value().parse().unwrap());

        match topo_settings.noise_type.clone() {
            Some(NoiseTypesUi::Simplex) =>
                {
                    &mut simplex.clone().set_seed(topo_settings.seed.unwrap() as u32);
                },
            Some(NoiseTypesUi::Perlin) => {
                &mut perlin.clone().set_seed(topo_settings.seed.unwrap() as u32);
            },
            Some(NoiseTypesUi::BillowPerlin) => { &mut billow.clone().set_seed(topo_settings.seed.unwrap() as u32); },
            _ => {}
        };
    });

    ui.noise_choice.add_choice("Perlin");
    ui.noise_choice.add_choice("Billowed Perlin");
    ui.noise_choice.add_choice("Simplex");
    ui.noise_choice.set_value(0);

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
                   simplex.clone().set_octaves(topo_settings.noise_octaves.unwrap() as usize);
                   map = PlaneMapBuilder::<Fbm<Simplex>, 2>::new(simplex.clone()).set_size(230, 230).set_is_seamless(false).set_x_bounds(-1.0, 1.0).set_y_bounds(-1.0, 1.0).build();
               },
               Some(NoiseTypesUi::Perlin) => {
                 perlin.clone().set_octaves(topo_settings.noise_octaves.unwrap() as usize);
                   map = PlaneMapBuilder::<Fbm<Perlin>, 2>::new(perlin.clone()).set_size(230, 230).set_is_seamless(false).set_x_bounds(-1.0, 1.0).set_y_bounds(-1.0, 1.0).build();
               },
               Some(NoiseTypesUi::BillowPerlin) => {
                   billow.clone().set_octaves(topo_settings.noise_octaves.unwrap() as usize);
                   map = PlaneMapBuilder::<Billow<Perlin>, 2>::new(billow.clone()).set_size(230, 230).set_is_seamless(false).set_x_bounds(-1.0, 1.0).set_y_bounds(-1.0, 1.0).build();
               },
               _ => {}

           };

           if Path::new("example_images/cache.png").exists() {
               fs::remove_file("example_images/cache.png").expect("Error deleting file.");
           }

           map.write_to_file("cache.png");

           use std::{fs, path::Path};

           let img = SharedImage::load(Path::new("example_images/cache.png")).expect("Error loading image.");

           ui.preview_box_topo.set_image_scaled(Some(img));

           ui.preview_box_topo.redraw();

       }
    });

    ui.noise_freq_input.set_callback(move |x3| {
        if x3.changed() {
            topo_settings.noise_frequency = Some(x3.value().parse().unwrap());
        }
    });

    ui.noise_lacunarity_input.set_callback(move |x4| {
        if x4.changed() {
            topo_settings.noise_lacunarity = Some(x4.value().parse().unwrap());
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
        if ui.topography_pane.active() {

        }
    }
}