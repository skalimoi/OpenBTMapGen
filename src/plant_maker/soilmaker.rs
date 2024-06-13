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

use photon_rs::multiple::blend;
use photon_rs::native::open_image;
use soil_binder::{elevpercentile, geomorphons, georreference, trindex};
use crate::plant_maker::config::GreyscaleImage;
use crate::soil_def::SoilType;

const DIM: u16 = 8192;

#[derive(Clone)]
struct GrayMap {
    pub map: GreyscaleImage<u8>
}
pub enum SoilColor {
    Dirt([u8; 4]),
    Loam([u8; 4]),
    Silt([u8; 4]),
    Clay([u8; 4]),
    Stone([u8; 4]),
    Gravel([u8; 4]),
    Sand([u8; 4])
}

pub fn init_soilmaker(f: &mut Frame, base_soil: SoilType, blocklist: &HashMap<SoilType, bool>, heightmap16: &ImageBuffer<Luma<u16>, Vec<u16>>, min_val: i32, max_val: i32) -> Vec<u8> {
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
    

    // let mut i = image_crate::open("cache/tri.png").unwrap().to_luma8();
    // let highest = *i.as_raw().iter().max().unwrap();
    // imageproc::contrast::stretch_contrast_mut(&mut i, 6, 10);
    // i.save("cache/tri.png");
    // 
    // let mut i = image_crate::open("cache/ep.png").unwrap().to_luma8();
    // let highest = *i.as_raw().iter().max().unwrap();
    // imageproc::contrast::stretch_contrast_mut(&mut i, 0, highest);
    // i.save("cache/ep.png");
    
    let mut base = open_image("cache/gm.png").expect("File should open.");
    let tri = open_image("cache/tri.png").expect("File should open.");
    let elevp = open_image("cache/ep.png").expect("");
    
    blend(&mut base, &tri, "hard_light");
    blend(&mut base, &elevp, "overlay");

    let raw = base.get_raw_pixels();
    
    let mut old_to_convert: buffer_old<Rgb_old<u8>, Vec<u8>> = buffer_old::from_raw(8192, 8192, raw).unwrap();

    /////////////////////
    
    let mut color_vec: Vec<[u8; 3]> = vec![];
    for (soil, value) in blocklist.clone() {
        if !value {
            match soil {
                SoilType::Dirt => color_vec.push([5,5,5]),
                SoilType::Silt => color_vec.push([100,100,100]),
                SoilType::Stone => color_vec.push([16,16,16]),
                SoilType::Gravel => color_vec.push([32,32,32]),
                SoilType::Loam => color_vec.push([64,64,64]),
                SoilType::Clay => color_vec.push([128,128,128]),
                SoilType::Sand => color_vec.push([200,200,200])
            }
        }
    }
    let colormap = color_reduce::palette::BasePalette::new(
        color_vec
    );
    quantize(&mut old_to_convert, &colormap, QuantizeMethod::Luma, None);
    let i = image_old::imageops::resize(&old_to_convert, 1024, 1024, image_old::imageops::FilterType::Nearest);
    i.save("soil_test.png");
    f.set_image_scaled(None::<SharedImage>);
    let s = RgbImage::new(i.as_raw().as_slice(), 1024, 1024, ColorDepth::Rgb8).unwrap();
    f.set_image(SharedImage::from_image(s).ok());
    f.redraw();
    i.into_raw()
}
