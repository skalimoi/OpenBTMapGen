use image_crate::{ImageBuffer, Luma, Pixel, Rgba};
use noise::utils::{NoiseImage, NoiseMap};

pub fn write_map_to_file(map: &NoiseMap, filename: &str) {
    use std::{fs, path::Path};

    // Create the output directory for the images, if it doesn't already exist
    let target_dir = Path::new("example_images/");

    if !target_dir.exists() {
        fs::create_dir(target_dir).expect("failed to create example_images directory");
    }

    //concatenate the directory to the filename string
    let directory: String = "example_images/".to_owned();
    let file_path = directory + filename;

    // collect the values from f64 into u8 in a separate vec
    let (width, height) = map.size();
    let mut b: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::new(width as u32, height as u32);

    for x in 0..width {
        for y in 0..height {
            let c = map.get_value(x, y);
            let l = ((c * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u16;
            b.put_pixel(x as u32, y as u32, *Luma::<u16>::from_slice(&[l]));
        }
    }


    b.save(file_path);

    println!("\nFinished generating {}", filename);
}

pub fn write_to_file(map: &NoiseImage, filename: &str) {
    use std::{fs, path::Path};

    // Create the output directory for the images, if it doesn't already exist
    let target_dir = Path::new("example_images/");

    if !target_dir.exists() {
        fs::create_dir(target_dir).expect("failed to create example_images directory");
    }

    //concatenate the directory to the filename string
    let directory: String = "example_images/".to_owned();
    let file_path = directory + filename;

    // collect the values from the map vector into an array
    let (width, height) = map.size();
    let mut b: ImageBuffer<Rgba<u16>, Vec<u16>> = ImageBuffer::new(width as u32, height as u32);

    for x in 0..width {
        for y in 0..height {
            let c = map.get_value(x, y);
            let s = c.map(|x| x as u16);
            b.put_pixel(x as u32, y as u32, Rgba::<u16>::from(s));
        }
    }

    let _ = b.save(file_path);

    println!("\nFinished generating {}", filename);
}
