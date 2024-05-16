use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
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
use image_newest::{ImageBuffer as buffer_old, Rgb as Rgb_old, Rgba as Rgba_old};
use imageproc::distance_transform::Norm;

use noise::{Fbm, MultiFractal, Perlin};
use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use rand::{Rng, thread_rng};
use crate::soil::config::{Biom, GreyscaleImage, Map, SimConfig, Soil, SunConfig, Vegetation};
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
    let mut dynamic_hydro = DynamicImage::ImageLuma8(hydro_map.clone());
    println!("Doing base soil.");
    let b = init_base(match base_soil {
        SoilType::Dirt => [5,5,5,255],
        SoilType::Silt => [100,100,100, 255],
        SoilType::Stone => [16,16,16, 255],
        SoilType::Gravel => [32,32,32, 255],
        SoilType::Loam => [64,64,64, 255],
        SoilType::Clay => [128,128,128, 255],
        SoilType::Sand => [200,200,200, 255]
    }, DIM as u32);
    println!("Doing height.");
    let h = generate_height(&mut dynamic);
    let d_1 = overlay_with_weights(&b, &h, 1.0);
    println!("Doing clay.");
    let cl = generate_random_clay(&mut dynamic);
    let d_2 = overlay_with_weights(&d_1, &cl, 1.0);
    println!("Doing slope.");
    let s = generate_slope_soil(&heightmap16);
    let d_3 = overlay_with_weights(&d_2, &s, 1.0);
    println!("Doing low.");
    let l = generate_low_soils(&mut dynamic, &mut dynamic_hydro);
    let d_4 = overlay_with_weights(&d_3, &l, 1.0);
    println!("Doing coast.");
    let c = generate_coast_sediment(&mut dynamic);
    let d_5 = overlay_with_weights(&d_4, &c, 1.0);
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
    let i = image_newest::imageops::resize(&old_to_convert, 1024, 1024, image_newest::imageops::FilterType::Nearest);
    i.save("soil_test.png");
    f.set_image_scaled(None::<SharedImage>);
    let s = RgbImage::new(i.as_raw().as_slice(), 1024, 1024, ColorDepth::Rgb8).unwrap();
    f.set_image(SharedImage::from_image(s).ok());
    f.redraw();
    i.into_raw()
}

fn init_base(soil: [u8; 4], size: u32) -> DynamicImage {
    let mut i: DynamicImage = DynamicImage::new_luma8(size, size);
    for x in 0..size {
        for y in 0..size {
            i.put_pixel(x, y, Rgba::from(soil));
        }
    }
    i
}

fn generate_height(i: &mut DynamicImage) -> DynamicImage {
    let colormap = color_reduce::palette::BasePalette::new(
        vec![
            [0,0,0],
            [16,16,16], // stone
            [32,32,32], // gravel
        ]
    );
    i.brighten(150); // TODO test if appropiate value
    let conversion_to_old_dyn_raw = i.clone().into_rgb8().into_raw();
    let mut c: buffer_old<Rgb_old<u8>, Vec<u8>> = buffer_old::from_raw(DIM as u32, DIM as u32, conversion_to_old_dyn_raw).unwrap();
    quantize(&mut c, &colormap, QuantizeMethod::Luma, None);
    let b: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(c.dimensions().0, c.dimensions().1, c.into_raw()).unwrap();
    ImageRgb8(b)
}

fn generate_random_clay(i: &mut DynamicImage) -> DynamicImage {
    let mut c = DynamicImage::new_luma_a8(i.dimensions().0, i.dimensions().1);
    let mut rng = thread_rng();
    let noise: Fbm<Perlin> = Fbm::new(rng.gen::<u32>())
        .set_octaves(8)
        .set_frequency(2.0)
        .set_lacunarity(2.0);
    let n: NoiseMap = PlaneMapBuilder::<Fbm<Perlin>, 2>::new(noise)
        .set_size(DIM as usize, DIM as usize)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build();
    for x in 0..i.dimensions().0 {
        for y in 0..i.dimensions().1 {
            let v = n.get_value(x as usize, y as usize);
            if v > 0.5 {
                c.put_pixel(x, y, Rgba([128,128,128, 255]));
            }
            else {
                c.put_pixel(x, y, Rgba([0,0,0, 0]));
            }
        }
    }
    
    c
}

fn generate_slope_soil(i: &ImageBuffer<Luma<u16>, Vec<u16>>) -> DynamicImage {
    let c = sobel(i);
    c.adjust_contrast(50.0).brighten(50);
    let mut d = DynamicImage::new_luma_a8(i.dimensions().0, i.dimensions().0);
    // TODO: change gravel for nothing
    for x in 0..i.dimensions().0 {
        for y in 0..i.dimensions().1 {
            let angle = c.get_pixel(x, y).0[0];
            if angle > 0 && angle <= 1 {
                d.put_pixel(x, y, Rgba([16,16,16,255]));
            } else if angle > 1 {
                d.put_pixel(x, y, Rgba([32,32,32,255]));
            } else {
                d.put_pixel(x, y, Rgba([0,0,0,0]));
            }
        }
    }
    
    d
}   

fn generate_low_soils(_i: &mut DynamicImage, h_map: &DynamicImage) -> DynamicImage {
    let index = BasePalette::new(vec![
       [0,0,0],
        [128,128,128]
    ]);
    let palette = BasePalette::new(vec![
       [0,0,0],
        [64,64,64],
        [100,100,100]
    ]);
    let converted_to_old_buffer: buffer_old<Rgb_old<u8>, Vec<u8>> = buffer_old::from_raw(DIM as u32, DIM as u32, h_map.clone().into_rgb8().into_raw()).unwrap();
    // quantize(&mut converted_to_old_buffer, &index, QuantizeMethod::CIE2000, Some(255));
    let raw_for_conversion = converted_to_old_buffer.into_raw();
    let buffer_for_conversion: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(DIM as u32, DIM as u32, raw_for_conversion).unwrap();
    let expanded = imageproc::morphology::open(&DynamicImage::ImageRgb8(buffer_for_conversion.clone()).into_luma8(), Norm::L1, 1);
    let mut clay = imageproc::morphology::dilate(&expanded, Norm::L1, 10);
    let expanded_1 = imageproc::morphology::dilate(&expanded, Norm::L1, 3);
    let expanded = imageproc::morphology::close(&expanded_1, Norm::L1, 8);
    let mut converted_to_old_buffer: buffer_old<Rgb_old<u8>, Vec<u8>> = buffer_old::from_raw(DIM as u32, DIM as u32, ImageLuma8(clay).into_rgb8().into_raw()).unwrap();
    quantize(&mut converted_to_old_buffer, &index, QuantizeMethod::Luma, None);
    let mut base = DynamicImage::ImageRgb8(ImageBuffer::from_raw(DIM as u32, DIM as u32, converted_to_old_buffer.into_raw()).unwrap());
    for x in 0..DIM {
        for y in 0..DIM {
            if expanded.get_pixel(x as u32, y as u32).0[0] != 0 {
                base.put_pixel(x as u32, y as u32, Rgba([100,100,100,255]));
            }
            if expanded_1.get_pixel(x as u32, y as u32).0[0] != 0 {
                base.put_pixel(x as u32, y as u32, Rgba([64,64,64,255]));
            }
        }
    }
    base
}

fn generate_coast_sediment(i: &DynamicImage) -> DynamicImage {
    let c = i.clone();
    let mut c_sel = DynamicImage::new(DIM as u32, DIM as u32, ColorType::L8);
    for x in 0..DIM as u32 {
        for y in 0..DIM as u32 {
            if c.get_pixel(x, y).0[0] == 0 || c.get_pixel(x, y).0[0] == 1 || c.get_pixel(x, y).0[0] == 2 || c.get_pixel(x, y).0[0] == 3 {
                c_sel.put_pixel(x, y, Rgba([200,200,200,255]));
            }
        }
    }
    let coast = imageproc::morphology::dilate(&c_sel.into_luma8(), Norm::L1, 3);
    ImageLuma8(coast)
}

fn overlay_with_weights(bottom: &DynamicImage, top: &DynamicImage, weight: f64) -> DynamicImage {
    let bottom_c = bottom.clone();
    let mut top_c = top.clone().into_rgba8();
    let top_dim = top_c.dimensions();

    for x in 0..top_dim.0 {
        for y in 0..top_dim.1 {
            let mut opacity: u8 = (255.0 * weight) as u8;
            let pixel = top_c.get_pixel(x, y);
            if pixel.0[0] == 0 {
                opacity = 0;
            }
            let applied: Rgba<u8> = Rgba([pixel.0[0], pixel.0[1], pixel.0[2], opacity]);
            top_c.put_pixel(x, y, applied);
        }
    }

    let c = bottom_c.clone().into_rgba8();
    let raw = c.clone().into_raw();
    let mut new: buffer_old<Rgb_old<u8>, Vec<u8>> = buffer_old::from_raw(c.clone().dimensions().clone().0.clone(), c.dimensions().0, raw).unwrap();
    let mut new_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(c.dimensions().0, c.dimensions().1, new.into_raw()).unwrap();

    for x in 0..DIM as u32 {
        for y in 0..DIM as u32 {
            if new_buffer.get_pixel(x, y).0[0] == 0 {
                new_buffer.put_pixel(x, y, Rgba([0,0,0,0]));
            }
        }
    }
    image_crate::imageops::overlay(&mut new_buffer, &top_c, 0, 0);
    let mut o: buffer_old<Rgb_old<u8>, Vec<u8>> = buffer_old::from_raw(c.clone().dimensions().clone().0.clone(), c.dimensions().0, new_buffer.into_raw()).unwrap();
    // quantize(&mut o, &colormap, QuantizeMethod::CIE2000, None);
    let mut z: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(c.dimensions().0, c.dimensions().1, o.into_raw()).unwrap();

    ImageRgba8(z)
}

fn sobel(input: &ImageBuffer<Luma<u16>, Vec<u16>>) -> DynamicImage {
    let mut buff = ImageBuffer::new(DIM as u32, DIM as u32);
    let width = DIM -2;
    let height = DIM -2;
    for i in 0..width as u32 {
        for j in 0..height as u32 {
            /* Unwrap those loops! */
            let val0 = input.get_pixel(i, j).0[0] as i32;
            let val1 = input.get_pixel(i + 1 , j).0[0] as i32;
            let val2 = input.get_pixel(i + 2, j).0[0] as i32;
            let val3 = input.get_pixel(i, j + 1).0[0] as i32;
            let val5 = input.get_pixel(i + 2, j + 1).0[0] as i32;
            let val6 = input.get_pixel(i, j + 2).0[0] as i32;
            let val7 = input.get_pixel(i + 1, j + 2).0[0] as i32;
            let val8 = input.get_pixel(i + 2, j + 2).0[0] as i32;
            /* Apply Sobel kernels */
            let gx = (-1 * val0) + (-2 * val3) + (-1 * val6) + val2 + (2 * val5) + val8;
            let gy = (-1 * val0) + (-2 * val1) + (-1 * val2) + val6 + (2 * val7) + val8;
            let mut mag = ((gx as f64).powi(2) + (gy as f64).powi(2)).sqrt();

            if mag > 32767.0 {
                mag = 32767.0;
            }

            buff.put_pixel(i, j, Luma([mag as u16]));
        }
    };
    ImageLuma16(buff)
}
