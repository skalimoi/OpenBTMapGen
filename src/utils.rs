use image_crate::{DynamicImage, GenericImageView, ImageBuffer, Luma, Pixel, Rgba};
use map_range::CheckedMapRange;
use noise::utils::{NoiseImage, NoiseMap};


pub fn get_height(image: &DynamicImage, max: f64, min_value_total: u16, max_value_total: u16) -> i32 {
    let map = image.to_luma16();
    let mut points: Vec<i32> = vec![];
    for pixel in map.pixels() {
            let value = pixel.channels().first().unwrap();
            let a = (*value as i32).checked_map_range((min_value_total as i32)..(max_value_total as i32), 1..(max as i32)).unwrap();
            points.push(a);
    }
    let median = points.iter().sum::<i32>() / points.len() as i32;
    median

}

pub fn get_raw_u16(map: &NoiseMap) -> Vec<u16> {
    let (width, height) = map.size();
    let mut b: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::new(width as u32, height as u32);

    for x in 0..width {
        for y in 0..height {
            let c = map.get_value(x, y);
            let l = ((c * 0.5 + 0.5).clamp(0.0, 1.0) * 32767.0) as u16;
            b.put_pixel(x as u32, y as u32, Luma([l]));
        }
    }
    let n = b.clone();
    n.as_raw().to_vec()
}

pub fn get_raw_u8(map: &NoiseImage) -> Vec<u8> {
    let (width, height) = map.size();
    let mut b: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width as u32, height as u32);

    for x in 0..width {
        for y in 0..height {
            let c = map.get_value(x, y);
            let s = c.map(|x| x as u8);
            b.put_pixel(x as u32, y as u32, Rgba(s));
        }
    }
    b.as_raw().to_vec()
}
