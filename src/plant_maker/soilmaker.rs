use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use imageproc::drawing::Canvas;
use color_reduce;
use color_reduce::{quantize, BasePalette, QuantizeMethod};
use fltk::enums::ColorDepth;
use fltk::frame::Frame;
use fltk::image::{PngImage, RgbImage, SharedImage};
use fltk::prelude::WidgetExt;
use image_crate::{ColorType, DynamicImage, GenericImage, ImageBuffer, Luma, Rgb, Rgba};
use image_crate::DynamicImage::{ImageLuma16, ImageLuma8, ImageRgb8, ImageRgba8};
use image_crate::imageops::FilterType;
use image_old::{ImageBuffer as buffer_old, Rgb as Rgb_old, Rgba as Rgba_old};
use imageproc::distance_transform::Norm;

use noise::{Fbm, MultiFractal, Perlin};
use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use rand::{Rng, thread_rng};
use soil_binder::georreference;
use crate::plant_maker::config::{Biom, GreyscaleImage, Map, SimConfig, Soil, SunConfig, Vegetation};
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

pub fn init_soilmaker(f: &mut Frame, base_soil: SoilType, blocklist: &HashMap<SoilType, bool>, heightmap16: &ImageBuffer<Luma<u16>, Vec<u16>>, hydro_map: &ImageBuffer<Luma<u8>, Vec<u8>>) -> Vec<u8> {
    let mut dynamic = DynamicImage::ImageLuma16(heightmap16.clone());

    // TODO delete dir before shutdown
    if !Path::new("cache").exists() {
        fs::create_dir("cache").expect("Cannot create cache dir!");
    }
    
    dynamic.save("cache/map.png");
    
    georreference();
    
    let mut old_to_convert: buffer_old<Rgb_old<u8>, Vec<u8>> = buffer_old::from_raw(DIM as u32, DIM as u32, d_5.into_rgb8().into_raw()).unwrap();
    let mut color_vec: Vec<[u8; 3]> = vec![];
    for (soil, value) in blocklist.clone() {
        if value == false {
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
