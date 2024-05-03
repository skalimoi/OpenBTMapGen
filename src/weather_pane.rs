use fltk::prelude::{ButtonExt, InputExt, MenuExt, ValuatorExt, WidgetExt};
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

pub fn vis_image(box: &mut Frame, hour: u32, grid_vector: &mut Vec<GenData>, state: &ViewState) {
    let mut i: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(16, 16);
    for component in grid_vector.as_slice() {
        let p = &mut cube_vector[component.index.0 as usize + 16 *(component.index.1 as usize + 6 * component.index.2 as usize)];
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
                    -60..=-10 => Color::from_rgba8(30, 92, 179, 255),
                    -11..=-1 => Color::from_rgba8(4, 161, 230, 255),
                    0..=5 => Color::from_rgba8(102, 204, 206, 255),
                    6..=10 => Color::from_rgba8(192, 229, 136, 255),
                    11..=15 => Color::from_rgba8(204, 230, 75, 255),
                    16..=20 => Color::from_rgba8(243, 240, 29, 255),
                    21..=25 => Color::from_rgba8(248, 157, 14, 255),
                    26..=30 => Color::from_rgba8(219, 30, 38, 255),
                    31..=90 => Color::from_rgba8(164, 38, 44, 255),
                    _ => Color::from_rgba8(255, 255, 255, 255)
                };
                let color_rgba = color.to_linear_rgba_u8();

            
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
                    opacity = 10;
                }
                i.put_pixel(component.index.x, component.index.z, Rgba[color.r, color.g, color.b, opacity]);
                box.set_image_scaled(None::<SharedImage>);
                box.set_image_scaled(Some(SharedImage::from_image(i).unwrap()));
                box.redraw();
            }
        },
        WeatherVisualization::Pressure => {
            if !component.pressure.is_empty() {
                let median = (component.pressure[range.clone()].iter().sum::<OrderedFloat<f64>>().0 as usize) / range.clone().len();
                let color = match median {
                    50..=950 => Color::from_rgba8(40, 40, 255, 255) ,
                    951..=990 => Color::from_rgba8(102, 102, 255, 255),
                    991..=1000 => Color::from_rgba8(161, 161, 255, 255),
                    1001..=1015 => Color::from_rgba8(203, 203, 255, 255),
                    1016..=1030 => Color::from_rgba8(255, 138, 138, 255),
                    1031..=1060 => Color::from_rgba8(255, 103, 103, 255),
                    1061..=2000 => Color::from_rgba8(255, 41, 41, 255),
                    _ => Color::from_rgba8(255, 255, 255, 0)
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
                i.put_pixel(component.index.x, component.index.z, Rgba[color.r, color.g, color.b, opacity]);
                box.set_image_scaled(None::<SharedImage>);
                box.set_image_scaled(Some(SharedImage::from_image(i).unwrap()));
                box.redraw();
            }
        },
        WeatherVisualization::Humidity => {
            let median = (component.humidity[range.clone()].iter().sum::<OrderedFloat<f64>>().0 as usize) / range.clone().len();
            let color = match median {
                0..=10 => Color::from_rgba8(255, 255, 217, 255),
                11..=20 => Color::from_rgba8(237, 248, 177, 255),
                21..=30 => Color::from_rgba8(199, 233, 180, 255),
                31..=40 => Color::from_rgba8(127, 205, 187, 255),
                41..=50 => Color::from_rgba8(65, 182, 196, 255),
                51..=60 => Color::from_rgba8(29, 145, 192, 255),
                61..=70 => Color::from_rgba8(34, 94, 168, 255),
                71..=80 => Color::from_rgba8(37, 52, 148, 255),
                81..=90 => Color::from_rgba8(8, 29, 88, 255),
                91..=100 => Color::from_rgba8(3, 20, 70, 255),
                _ => Color::from_rgba8(255, 255, 255, 0)
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
            i.put_pixel(component.index.x, component.index.z, Rgba[color.r, color.g, color.b, opacity]);
            box.set_image_scaled(None::<SharedImage>);
            box.set_image_scaled(Some(SharedImage::from_image(i).unwrap()));
            box.redraw();        }
    }
}
box.show();
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
