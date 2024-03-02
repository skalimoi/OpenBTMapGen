use image_crate::{ImageBuffer, Luma, Pixel};
use image_crate::imageops::FilterType;
use noise::utils::{ImageRenderer, NoiseMap};
use fltk::prelude::WidgetExt;
use fltk::image::SharedImage;
use fltk::enums::ColorDepth;
use map_range::MapRange;
use crate::{FileData, topography};
use crate::erosion::world::{Vec2, World};
use crate::topo_settings::NoiseTypesUi;
use crate::topography::PREV_DIMENSIONS;
use crate::utils::get_raw_u8;

pub fn hydro_preview_do(data: &mut FileData) {
    match data.topography.noise_type.unwrap() {
        NoiseTypesUi::Perlin => topography::update_perlin_noise(data, PREV_DIMENSIONS),
        NoiseTypesUi::Simplex => topography::update_simplex_noise(PREV_DIMENSIONS, data),
        NoiseTypesUi::BillowPerlin => topography::update_billow_noise(PREV_DIMENSIONS, data),
    }
}

pub fn erode_heightmap_full(file: &mut FileData) {

    let mut discharge_map = vec![0; 8192 * 8192];

    let img: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::from_raw(512, 512, file.clone().raw_map_512).unwrap();

    let heightmap = img.into_raw();
    
    let mut erosion_world_1 = World::new(
        heightmap,
        512,
        512,
        file.topography.seed.unwrap() as i16,
    );
    
    //////////////////////
    //    SIZE: 512     //
    //////////////////////

    eprint!("Eroding full preview.");
    eprintln!("Size: 512");
    eprintln!(
        "Total cycle count: {}",
        file.topography.erosion_cycles as i32
    );

    for cycle in 0..(file.topography.erosion_cycles as i32) {
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
    
    let b: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::from_raw(1024, 1024, resampled_vec_1024).unwrap();

    //CHECK IF IT WORKS TODO 
    
    let i = imageproc::noise::gaussian_noise(&b, 1.5, 0.5, 284732);
    
    let mut erosion_world_2 = World::new(
        i.to_vec(),
        1024,
        1024,
        file.topography.seed.unwrap() as i16,
    );
    
    eprintln!("Size: 1024");
    eprintln!(
        "Total cycle count: {}",
        file.topography.erosion_cycles as i32
    );
    
    for cycle in 0..(file.topography.erosion_cycles as i32) {
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

    //CHECK IF IT WORKS TODO 

    let b: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::from_raw(2048, 2048, resampled_vec_2048).unwrap();

    let i = imageproc::noise::gaussian_noise(&b, 1.5, 0.5, 284732);
    
    let mut erosion_world_3 = World::new(
        i.to_vec(),
        2048,
        2048,
        file.topography.seed.unwrap() as i16,
    );
    
    eprintln!("Size: 2048");
    eprintln!(
        "Total cycle count: {}",
        file.topography.erosion_cycles as i32
    );
    
    for cycle in 0..(file.topography.erosion_cycles as i32) {
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
        file.topography.seed.unwrap() as i16,
    );
    
    eprintln!("Size: 4096");
    eprintln!(
        "Total cycle count: {}",
        file.topography.erosion_cycles as i32
    );
    
    for cycle in 0..(file.topography.erosion_cycles as i32) {
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
        file.topography.seed.unwrap() as i16,
    );
    
    eprintln!("Size: 8192");
    eprintln!(
        "Total cycle count: {}",
        file.topography.erosion_cycles as i32
    );
    
    for cycle in 0..(file.topography.erosion_cycles as i32) {
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

    file.eroded_full = eroded_preview.clone();

    let mut discharge_buffer: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_raw(8192, 8192, discharge_map.clone()).unwrap();

    imageproc::contrast::stretch_contrast_mut(&mut discharge_buffer, 130, 200);

    file.discharge = discharge_buffer.clone().into_raw();

    // here was apply_color_eroded(file, 8192);
}

pub fn update_hydro_prev(w: &mut impl WidgetExt, left: bool, file: &mut FileData) {
    topography::apply_color_eroded(file, 8192);
    w.set_image_scaled(None::<SharedImage>);

    match left {
        true => {
            let img = fltk::image::RgbImage::new(file.eroded_full_color.as_slice(), 8192, 8192, ColorDepth::Rgba8).unwrap();
            w.set_image_scaled(Some(img));
            w.redraw();
        }
        false => {
            let img = fltk::image::RgbImage::new(file.discharge.as_slice(), 8192, 8192, ColorDepth::L8).unwrap();
            w.set_image_scaled(Some(img));
            w.redraw();
        }
    };
}

fn apply_color_hydro_full(file: &mut FileData) {
    
    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let b: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::from_raw(8192, 8192, file.clone().eroded_full).unwrap();

    let mut map = NoiseMap::new(8192, 8192);

    for x in 0..8192 {
        for y in 0..8192 {
            let pixel = b
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

    let i = get_raw_u8(&r);

    file.eroded_full_color = i.to_vec();
}
