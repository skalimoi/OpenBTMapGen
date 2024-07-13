use noise::Seedable;
use fltk::prelude::{InputExt, MenuExt, ValuatorExt, WidgetExt};
use fltk::image;
use noise::utils::{Color, ImageRenderer, NoiseMap};
use image_crate::{ImageBuffer, Luma, Pixel, Rgb};
use fltk::enums::ColorDepth;
use fltk::image::{RgbImage, SharedImage};
use map_range::MapRange;
use crate::FileData;
use crate::erosion::world::World;
use crate::topo_settings::TopoSettings;
use crate::utils::get_raw_u8;

pub const DIMENSIONS: usize = 512;
pub const PREV_DIMENSIONS: usize = 512;

pub const DEFAULT_TOPOSETTINGS: TopoSettings =  TopoSettings {
seed: Some(42949),
max_alt: 0.0,
min_bound: (0.0, 0.0),
max_bound: (100.0, 100.0),
lod: 4.0,
erod_scale: 0.0,
mountain_pct: 25.0,
sea_pct: 5.0,
min_height: -50,
max_height: 1000,
erosion_cycles: 0,
};

pub fn min_bounds_do(w: &mut impl ValuatorExt, data: &mut FileData) {
        println!("asdfa: {}", w.value());
        data.topography.set_min_bounds(w.value());
}

pub fn max_bounds_do(w: &mut impl ValuatorExt, data: &mut FileData) {
        data.topography.set_max_bounds(w.value());
}

pub fn lod_do(w: &mut impl ValuatorExt, data: &mut FileData) {
        data.topography.set_lod(w.value());
}

pub fn erod_scale_do(w: &mut impl ValuatorExt, data: &mut FileData) {
        data.topography.set_erod_scale(w.value());
}

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
pub fn color_eroded_image(file: &mut FileData) {
    file.color_eroded_512.clear();
    let colormap: Vec<([u8; 3], f64)> = vec![
        ([70, 150, 200], 0.0),
        ([240, 240, 210], 0.5),
        ([190, 200, 120], 1.0),
        ([25, 100, 25], 18.0),
        ([15, 60, 15], 30.0),
    ];
    let get_color = |altitude: f64| -> [u8; 3] {
        let color_index = {
            let mut i = 0;
            while i < colormap.len() {
                if altitude < colormap[i].1 {
                    break;
                }
                i += 1;
            }
            i
        };

        if color_index == 0 {
            colormap[0].0
        } else if color_index == colormap.len() {
            colormap[colormap.len() - 1].0
        } else {
            let color_a = colormap[color_index - 1];
            let color_b = colormap[color_index];

            let prop_a = color_a.1;
            let prop_b = color_b.1;

            let prop = (altitude - prop_a) / (prop_b - prop_a);

            let color = [
                (color_a.0[0] as f64 + (color_b.0[0] as f64 - color_a.0[0] as f64) * prop) as u8,
                (color_a.0[1] as f64 + (color_b.0[1] as f64 - color_a.0[1] as f64) * prop) as u8,
                (color_a.0[2] as f64 + (color_b.0[2] as f64 - color_a.0[2] as f64) * prop) as u8,
            ];

            color
        }
    };
    let mut i: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(512, 512);
    let r: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::from_raw(512, 512, file.raw_map_512.clone()).unwrap();
    
    for x in 0..512 {
        for y in 0..512 {
            let p = r.get_pixel(x, y);
            let raw_altitude = (p[0] / 32767) as f64 * file.topography.max_alt;
            let color = get_color(raw_altitude);
            i.put_pixel(x, y, Rgb::from(color));
        }
    }
    file.color_eroded_512 = i.into_raw();
}
pub fn apply_color_eroded(file: &mut FileData, size: u16) {
    let colormap: Vec<([u8; 3], f64)> = vec![
        ([70, 150, 200], 0.0),
        ([240, 240, 210], 0.5),
        ([190, 200, 120], 1.0),
        ([25, 100, 25], 18.0),
        ([15, 60, 15], 30.0),
    ];
    let p1 = 0.5.map_range(0.0..30.0, -1.0..1.0);
    let p2 = 1.0.map_range(0.0..30.0, -1.0..1.0);
    let p3 = 18.0.map_range(0.0..30.0, -1.0..1.0);
    let p4 = 30.0.map_range(0.0..30.0, -1.0..1.0);
    let gradient = noise::utils::ColorGradient::new()
        .add_gradient_point(p1, Color::from([240, 240, 210, 255]))
        .add_gradient_point(p2, Color::from([190, 200, 120, 255]))
        .add_gradient_point(p3, Color::from([25, 100, 25, 255]))
        .add_gradient_point(p4, Color::from([15, 60, 15, 255]));
    
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

pub fn apply_color(file: &mut FileData) {
    let colormap: Vec<([u8; 3], f64)> = vec![
        ([70, 150, 200], 0.0),
        ([240, 240, 210], 0.5),
        ([190, 200, 120], 1.0),
        ([25, 100, 25], 18.0),
        ([15, 60, 15], 30.0),
    ];
    let p1 = 0.5.map_range(0.0..30.0, -1.0..1.0);
    let p2 = 1.0.map_range(0.0..30.0, -1.0..1.0);
    let p3 = 18.0.map_range(0.0..30.0, -1.0..1.0);
    let p4 = 30.0.map_range(0.0..30.0, -1.0..1.0);
    let gradient = noise::utils::ColorGradient::new()
        .add_gradient_point(p1, Color::from([240, 240, 210, 255]))
        .add_gradient_point(p2, Color::from([190, 200, 120, 255]))
        .add_gradient_point(p3, Color::from([25, 100, 25, 255]))
        .add_gradient_point(p4, Color::from([15, 60, 15, 255]));

    let data = file.raw_map_512.clone();
    let eroded_image: ImageBuffer<Luma<u16>, Vec<u16>> = image_crate::ImageBuffer::from_raw(512, 512, data).unwrap();
    let mut map = NoiseMap::new(512, 512);

    for x in 0..512 {
        for y in 0..512 {
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

    file.color_map_512 = i.to_vec();

}


/// 0 => preview_box_topo (512 not eroded)
/// 1 => preview_erosion_topo (512 eroded)
/// 2 => hydro_preview (8192 eroded)
/// 3 => hydro_mask_preview (8192 water mask)
pub fn update_noise_img(w: &mut impl WidgetExt, data: &FileData, img_type: u8, depth: ColorDepth) {
    let map = match img_type {
        0 => { image::RgbImage::new(data.color_map_512.as_slice(), 512, 512, depth).unwrap() },
        1 => image::RgbImage::new(data.color_eroded_512.as_slice(), 512, 512,depth).unwrap(),
        _ => RgbImage::new(&[], 512, 512, ColorDepth::Rgba8).unwrap()
    };
    w.set_image_scaled(None::<SharedImage>);
    let img = SharedImage::from_image(map).unwrap();
    w.set_image_scaled(Some(img));
    w.redraw();
}



pub fn cycle_input_do(w: &mut impl ValuatorExt, data: &mut FileData) {
    data.topography.set_cycles(w.value() as u64);
}
