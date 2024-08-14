use std::collections::HashMap;
use std::fs;
use std::path::Path;
use color_reduce;
use color_reduce::{quantize, QuantizeMethod};
use fltk::enums::ColorDepth;
use fltk::frame::Frame;
use fltk::image::{RgbImage, SharedImage};
use fltk::prelude::WidgetExt;
use image_crate::{DynamicImage, ImageBuffer, Luma};
use image_old::{ImageBuffer as buffer_old, Rgb as Rgb_old};
use photon_rs::conv::gaussian_blur;

use photon_rs::multiple::blend;
use photon_rs::native::{open_image, save_image};
use photon_rs::noise::add_noise_rand;
use soil_binder::{elevpercentile, geomorphons, georreference, trindex};
use crate::plant_maker::config::Soil;


// pub fn load_and_show_veg(w: &mut CheckBrowser) {
//     let mut data = String::new();
//     File::open("vegetation_types.yaml")
//         .unwrap()
//         .read_to_string(&mut data)
//         .unwrap();
//     let vegetations: HashMap<String, Vegetation> = serde_yaml::from_str(&data).unwrap();
//     for vegetation in vegetations.iter() {
//         w.add(vegetation.0.clone().as_str(), false);
//     }
// }


pub fn init_soilmaker(blend1: &str, blend2: &str, f: &mut Frame, soils: HashMap<String, Soil>, heightmap16: &ImageBuffer<Luma<u16>, Vec<u16>>, min_val: i32, max_val: i32) -> Vec<u8> {
    
    if Path::new("cache/gm.png").exists() && Path::new("cache/tri.png").exists() && Path::new("cache/ep.png").exists() {
        let mut base = open_image("cache/gm.png").expect("File should open.");
        let tri = open_image("cache/tri.png").expect("File should open.");
        let elevp = open_image("cache/ep.png").expect("");

        blend(&mut base, &elevp, blend1);
        blend(&mut base, &tri, blend2);
        gaussian_blur(&mut base, 1);
        // add_noise_rand(&mut base);
        save_image(base, "cache/fusion.png");

        let raw = image_crate::open("cache/fusion.png").unwrap().into_rgb8().into_raw();

        let mut old_to_convert: buffer_old<Rgb_old<u8>, Vec<u8>> = buffer_old::from_raw(8192, 8192, raw).unwrap();

        /////////////////////

        let mut color_vec: Vec<[u8; 3]> = vec![];
        for (soil, value) in soils {
            color_vec.push([value.id, value.id, value.id]);
        }
        let colormap = color_reduce::palette::BasePalette::new(
            color_vec
        );
        quantize(&mut old_to_convert, &colormap, QuantizeMethod::CIE2000, None);
        let p = image_old::imageops::resize(&old_to_convert, 1024, 1024, image_old::imageops::FilterType::Nearest);
        f.set_image_scaled(None::<SharedImage>);
        let s = RgbImage::new(p.as_raw().as_slice(), 1024, 1024, ColorDepth::Rgb8).unwrap();
        f.set_image_scaled(SharedImage::from_image(s).ok());
        f.redraw();
        old_to_convert.into_raw()
    } else {
        let dynamic = DynamicImage::ImageLuma16(heightmap16.clone());

        if !Path::new("cache").exists() {
            fs::create_dir("cache").expect("Cannot create cache dir!");
        } else {
            fs::remove_dir_all("cache");
            fs::create_dir("cache").expect("Cannot create cache dir!");
        }

        dynamic.save("cache/map.png");

        georreference(min_val, max_val);

        geomorphons();

        elevpercentile();

        trindex();

        //  blend operation //

        let mut i = image_crate::open("cache/gm.png").unwrap().to_luma8();
        let highest = *i.as_raw().iter().max().unwrap();
        imageproc::contrast::stretch_contrast_mut(&mut i, 0, highest);
        i.save("cache/gm.png");
        
        let mut i = image_crate::open("cache/tri.png").unwrap().to_luma8();
        imageproc::contrast::stretch_contrast_mut(&mut i, 0, 10);
        i.save("cache/tri.png");
        // 
        // let mut i = image_crate::open("cache/ep.png").unwrap().to_luma8();
        // let highest = *i.as_raw().iter().max().unwrap();
        // imageproc::contrast::stretch_contrast_mut(&mut i, 0, highest);
        // i.save("cache/ep.png");

        let mut base = open_image("cache/gm.png").expect("File should open.");
        let tri = open_image("cache/tri.png").expect("File should open.");
        let elevp = open_image("cache/ep.png").expect("");

        blend(&mut base, &elevp, blend1);
        blend(&mut base, &tri, blend2);
        gaussian_blur(&mut base, 3);


        save_image(base, "cache/fusion.png");

        let raw = image_crate::open("cache/fusion.png").unwrap().into_rgb8().into_raw();

        let mut old_to_convert: buffer_old<Rgb_old<u8>, Vec<u8>> = buffer_old::from_raw(8192, 8192, raw).unwrap();

        /////////////////////

        let mut color_vec: Vec<[u8; 3]> = vec![];
        for (soil, value) in soils {
            color_vec.push([value.id, value.id, value.id]);
        }
        let colormap = color_reduce::palette::BasePalette::new(
            color_vec
        );
        quantize(&mut old_to_convert, &colormap, QuantizeMethod::CIE2000, None);
        let p = image_old::imageops::resize(&old_to_convert, 1024, 1024, image_old::imageops::FilterType::Nearest);
        f.set_image_scaled(None::<SharedImage>);
        let s = RgbImage::new(p.as_raw().as_slice(), 1024, 1024, ColorDepth::Rgb8).unwrap();
        f.set_image_scaled(SharedImage::from_image(s).ok());
        f.redraw();
        old_to_convert.into_raw()
    }
    
}
