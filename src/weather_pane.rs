use colorgrad::Color;
use fltk::enums::ColorDepth;
use fltk::frame::Frame;
use fltk::group::Group;
use fltk::image::{PngImage, RgbImage, SharedImage};
use fltk::prelude::{ButtonExt, InputExt, MenuExt, ValuatorExt, WidgetExt};
use image_crate::{imageops};
use image_crate::imageops::FilterType;
use image_old::{ImageBuffer, Rgb};
use ordered_float::OrderedFloat;
use rand::{Rng, thread_rng};
use three_d::{ColorMaterial, Gm, Mesh};
use crate::{FileData, ViewState, WeatherVisualization};
use crate::weather::{Climate, GenData, HumidDry};
use crate::weather_settings::WeatherSettings;

pub const DEFAULT_WEATHERSETTINGS: WeatherSettings = WeatherSettings {
seed: None,
koppen: None,
latitude: 0,
grid_size: 16,
};

pub fn vis_image(r#box: &mut Group, hour: u32, grid_vector: &mut Vec<GenData>, state: &ViewState) {
    let mut i: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(16, 16);
    let layer = state.layer;
    for component in grid_vector.clone().as_slice() {
        if component.index.1 == layer as i32 {
            let p = &mut grid_vector[component.index.0 as usize + 16 *(component.index.1 as usize + 6 * component.index.2 as usize)];
            let range = match hour {
                0 => 0..24,
                _ => (((24 * hour) - 24) as usize)..((24 * hour) as usize)
            };
            match state.mode {
                WeatherVisualization::Init => {},
                WeatherVisualization::Wind => {},
                WeatherVisualization::Temperature => {
                    if !p.temperature.is_empty() {
                        println!("{:?}", p.index);
                        let median = (p.temperature[range.clone()].iter().sum::<OrderedFloat<f64>>().0 as isize) / range.clone().len() as isize;
                        let color = match median {
                            -60..=-10 => [30, 92, 179],
                            -11..=-1 => [4, 161, 230],
                            0..=5 => [102, 204, 206],
                            6..=10 => [192, 229, 136],
                            11..=15 => [204, 230, 75],
                            16..=20 => [243, 240, 29],
                            21..=25 => [248, 157, 14],
                            26..=30 => [219, 30, 38],
                            31..=90 => [164, 38, 44],
                            _ => [255, 255, 255]
                        };
                        println!("COLOR: {:?}", color);

                        i.put_pixel(p.index.0 as u32, p.index.2 as u32, Rgb::from([color[0] as u8, color[1] as u8, color[2] as u8]));

                    }
                },
                WeatherVisualization::Pressure => {
                    if !p.pressure.is_empty() {
                        let median = (p.pressure[range.clone()].iter().sum::<OrderedFloat<f64>>().0 as usize) / range.clone().len();
                        let color = match median {
                            50..=950 => [40, 40, 255] ,
                            951..=990 => [102, 102, 255],
                            991..=1000 => [161, 161, 255],
                            1001..=1015 => [203, 203, 255],
                            1016..=1030 => [255, 138, 138],
                            1031..=1060 => [255, 103, 103],
                            1061..=2000 => [255, 41, 41],
                            _ => [255, 255, 255]
                        };
                        
                        i.put_pixel(p.index.0 as u32, p.index.2 as u32, Rgb::from([color[0] as u8, color[1] as u8, color[2] as u8]));

                    }
                },
                WeatherVisualization::Humidity => {
                    let median = (p.humidity[range.clone()].iter().sum::<OrderedFloat<f64>>().0 as usize) / range.clone().len();
                    let color = match median {
                        0..=10 => [255, 255, 217],
                        11..=20 => [237, 248, 177],
                        21..=30 => [199, 233, 180],
                        31..=40 => [127, 205, 187],
                        41..=50 => [65, 182, 196],
                        51..=60 => [29, 145, 192],
                        61..=70 => [34, 94, 168],
                        71..=80 => [37, 52, 148],
                        81..=90 => [8, 29, 88],
                        91..=100 => [3, 20, 70],
                        _ => [255, 255, 255]
                    };
                    i.put_pixel(p.index.0 as u32, p.index.2 as u32, Rgb::from([color[0] as u8, color[1] as u8, color[2] as u8]));
                }
        }
        
    }
}
    i.save("test_16.png").unwrap();
    r#box.set_image_scaled(None::<SharedImage>);
    let a = image_old::imageops::resize(&i, 1024, 1024, image_old::imageops::FilterType::Nearest);
    let s = RgbImage::new(a.as_raw().as_slice(), 1024, 1024, ColorDepth::Rgb8).unwrap();
    r#box.set_image_scaled(Some(SharedImage::from_image(s).unwrap()));
    r#box.redraw();
    a.save("test.png").unwrap();
}

pub fn set_hour(w: &mut impl ValuatorExt, state: &mut ViewState) {
    state.hour = w.value() as u32;
}

/// Outputs previous state for comparison
pub fn set_view_state(options: &mut ViewState, state: WeatherVisualization) -> WeatherVisualization {
    let previous_state = options.mode;
    options.mode = state;
    previous_state
}

pub fn update_grid_at_time(hour: u32, grid_vector: &mut Vec<GenData>, cube_vector: &mut [Gm<Mesh, ColorMaterial>], state: &ViewState) {
    use ordered_float::OrderedFloat;
    use colorgrad::Color;
    use three_d_asset::Srgba;
    use crate::WeatherVisualization::Init;
    for component in grid_vector.as_slice() {
        let cube = &mut cube_vector[component.index.0 as usize + 16 *(component.index.1 as usize + 6 * component.index.2 as usize)];
        let range = match hour {
            0 => 0..24,
            _ => (((24 * hour) - 24) as usize)..((24 * hour) as usize)
        };
        match state.mode {
            Init => {},
            WeatherVisualization::Wind => {},
            WeatherVisualization::Temperature => {
                if !component.pressure.is_empty() {
                    let median = (component.temperature[range.clone()].iter().sum::<OrderedFloat<f64>>().0 as isize) / range.clone().len() as isize;
                    let color = match median {
                        -60..=-10 => Color::from_rgba8(30, 92, 179, 10),
                        -11..=-1 => Color::from_rgba8(4, 161, 230, 10),
                        0..=5 => Color::from_rgba8(102, 204, 206, 10),
                        6..=10 => Color::from_rgba8(192, 229, 136, 10),
                        11..=15 => Color::from_rgba8(204, 230, 75, 10),
                        16..=20 => Color::from_rgba8(243, 240, 29, 10),
                        21..=25 => Color::from_rgba8(248, 157, 14, 10),
                        26..=30 => Color::from_rgba8(219, 30, 38, 10),
                        31..=90 => Color::from_rgba8(164, 38, 44, 10),
                        _ => Color::from_rgba8(255, 255, 255, 10)
                    };
                    let color_rgba = color.to_linear_rgba_u8();
                    let mut opacity = 10;

                    match state.layer {
                        0 => {opacity = 10},
                        1 => {if component.index.1 != 0 {opacity = 0}},
                        2 => {if component.index.1 != 1 {opacity = 0}},
                        3 => {if component.index.1 != 2 {opacity = 0}},
                        4 => {if component.index.1 != 3 {opacity = 0}},
                        5 => {if component.index.1 != 4 {opacity = 0}},
                        6 => {if component.index.1 != 5 {opacity = 0}},
                        _ => { opacity = 10 }
                    }
                    if color.r == 1.0 && color.g == 1.0 && color.b == 1.0 {
                        opacity = 0;
                    }
                    cube.material.color = Srgba::new(color_rgba.0, color_rgba.1, color_rgba.2, opacity);
                }
            },
            WeatherVisualization::Pressure => {
                if !component.pressure.is_empty() {
                    let median = (component.pressure[range.clone()].iter().sum::<OrderedFloat<f64>>().0 as usize) / range.clone().len();
                    let color = match median {
                        50..=950 => Color::from_rgba8(40, 40, 255, 10) ,
                        951..=990 => Color::from_rgba8(102, 102, 255, 10),
                        991..=1000 => Color::from_rgba8(161, 161, 255, 10),
                        1001..=1015 => Color::from_rgba8(203, 203, 255, 10),
                        1016..=1030 => Color::from_rgba8(255, 138, 138, 10),
                        1031..=1060 => Color::from_rgba8(255, 103, 103, 10),
                        1061..=2000 => Color::from_rgba8(255, 41, 41, 10),
                        _ => Color::from_rgba8(255, 255, 255, 10)
                    };
                    let color_rgba = color.to_linear_rgba_u8();
                    let mut opacity = 10;
                    match state.layer {
                        0 => {opacity = 10},
                        1 => {if component.index.1 != 0 {opacity = 0}},
                        2 => {if component.index.1 != 1 {opacity = 0}},
                        3 => {if component.index.1 != 2 {opacity = 0}},
                        4 => {if component.index.1 != 3 {opacity = 0}},
                        5 => {if component.index.1 != 4 {opacity = 0}},
                        6 => {if component.index.1 != 5 {opacity = 0}},
                        _ => { opacity = 10 }
                    }
                    cube.material.color = Srgba::new(color_rgba.0, color_rgba.1, color_rgba.2, opacity);
                }
            },
            WeatherVisualization::Humidity => {
                let median = (component.humidity[range.clone()].iter().sum::<OrderedFloat<f64>>().0 as usize) / range.clone().len();
                let color = match median {
                    0..=10 => Color::from_rgba8(255, 255, 217, 10),
                    11..=20 => Color::from_rgba8(237, 248, 177, 10),
                    21..=30 => Color::from_rgba8(199, 233, 180, 10),
                    31..=40 => Color::from_rgba8(127, 205, 187, 10),
                    41..=50 => Color::from_rgba8(65, 182, 196, 10),
                    51..=60 => Color::from_rgba8(29, 145, 192, 10),
                    61..=70 => Color::from_rgba8(34, 94, 168, 10),
                    71..=80 => Color::from_rgba8(37, 52, 148, 10),
                    81..=90 => Color::from_rgba8(8, 29, 88, 10),
                    91..=100 => Color::from_rgba8(3, 20, 70, 10),
                    _ => Color::from_rgba8(255, 255, 255, 10)
                };
                let color_rgba = color.to_linear_rgba_u8();
                let mut opacity = 10;

                match state.layer {
                    0 => {opacity = 10},
                    1 => {if component.index.1 != 0 {opacity = 0}},
                    2 => {if component.index.1 != 1 {opacity = 0}},
                    3 => {if component.index.1 != 2 {opacity = 0}},
                    4 => {if component.index.1 != 3 {opacity = 0}},
                    5 => {if component.index.1 != 4 {opacity = 0}},
                    6 => {if component.index.1 != 5 {opacity = 0}},
                    _ => { opacity = 10 }
                }
                if color.r == 1.0 && color.g == 1.0 && color.b == 1.0 {
                    opacity = 0;
                }
                cube.material.color = Srgba::new(color_rgba.0, color_rgba.1, color_rgba.2, opacity);
            }
        }
    }
}

pub fn weather_seed_do(w: &mut impl InputExt, data: &mut FileData) {
    if w.changed() {
        data.weather.set_seed(w.value().parse().unwrap());
    }
}

pub fn weather_lat_do(w: &mut impl InputExt, data: &mut FileData) {
    if w.changed() {
        data.weather.set_latitude(w.value().parse().unwrap());
    }
}

pub fn weather_grid_size_do(w: &mut impl InputExt, data: &mut FileData) {
    if w.changed() {
        data.weather.set_grid_size(w.value().parse().unwrap());
    }
}

pub fn weather_climate_do(w: &mut impl MenuExt, data: &mut FileData, climates: &[Climate; 18]) {

        let choice_name = w.choice().unwrap();
        let mut climate_choice: Climate = Climate {
            name: "Blank".to_string(),
            general_type: 'x',
            second_type: 'x',
            third_type: 'x',
            spring: (HumidDry::Humid, Default::default()),
            winter: (HumidDry::Humid, Default::default()),
            fall: (HumidDry::Humid, Default::default()),
            summer: (HumidDry::Humid, Default::default()),
            diurnal_range: Default::default(),
        };
        for climate in climates {
            if choice_name.as_str() == climate.name.as_str() {
                climate_choice = climate.clone();
                break
            } else {
                continue
            }
        }
    data.weather.set_climate(climate_choice);

}

pub fn weather_seed_random_do(
    _w: &mut impl ButtonExt,
    seed_box: &mut impl InputExt,
    data: &mut FileData,
) {
    let mut rng = thread_rng();
    let seed: u32 = rng.gen_range(u32::MIN..u32::MAX);
    seed_box.set_value(&format!("{}", seed));
    data.weather.seed = Some(seed);
}
