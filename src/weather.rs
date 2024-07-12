use std::f32::consts::E;
use std::fs::File;
use std::io::{Read, Write};
use nalgebra::Vector3;
use noise::{Fbm, NoiseFn, Perlin};
use rand::{Rng, thread_rng};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::ops::Range;
use std::string::ToString;

use ordered_float::OrderedFloat;
use crate::weather::HumidDry::{Dry, Humid};

const COLD_SPRING: Range<f32> = 0.0..15.0;
const COLD_WINTER: Range<f32> = -25.0..5.0;
const COLD_FALL: Range<f32> = -10.0..5.0;
const COLD_SUMMER: Range<f32> = 5.0..17.0;

const TEMPERATE_SPRING: Range<f32> = 15.0..20.0;
const TEMPERATE_WINTER: Range<f32> = 0.0..15.0;
const TEMPERATE_FALL: Range<f32> = 7.0..15.0;
const TEMPERATE_SUMMER: Range<f32> = 20.0..27.0;

const WARM_SPRING: Range<f32> = 18.0..25.0;
const WARM_WINTER: Range<f32> = 5.0..15.0;
const WARM_FALL: Range<f32> = 10.0..20.0;
const WARM_SUMMER: Range<f32> = 25.0..33.0;

const HOT_SPRING: Range<f32> = 25.0..35.0;
const HOT_WINTER: Range<f32> = 15.0..22.0;
const HOT_FALL: Range<f32> = 17.0..25.0;
const HOT_SUMMER: Range<f32> = 30.0..45.0;

pub const EQUATOR_TEMP_RANGE: Range<f32> = 0.0..3.0;
pub const TEMPERATE_TEMP_RANGE: Range<f32> = 7.0..10.0;
pub const CONTINENTAL_POLAR_TEMP_RANGE: Range<f32> = 12.0..25.0;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum HumidDry {
    Humid,
    Dry,
    None,
}

pub struct GridComponent {

    pub index: Vector3<i32>,

    pub mean_altitude: f32,

    pub temperature: f32,

    pub wind_p: Vector3<f32>,

    pub pressure: f32,

    pub humidity: f32,

    pub td: f32,

}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Climate {
    pub name: String,
    pub general_type: char,
    pub second_type: char,
    pub third_type: char,
    pub spring: (HumidDry, Range<f32>),
    pub winter: (HumidDry, Range<f32>),
    pub fall: (HumidDry, Range<f32>),
    pub summer: (HumidDry, Range<f32>),
    pub diurnal_range: Range<f32>
}

// TODO: evapotranspiration? Precipitation type?

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum Season {
    Winter,
    Spring,
    Fall,
    Summer
}

pub fn koppen_et() -> Climate {
    Climate {
        name: "Tundra".to_string(),
        general_type: 'E',
        second_type: 'T',
        third_type: '_',
        spring: (Dry, COLD_SPRING),
        winter: (Dry, COLD_WINTER),
        fall: (Dry, COLD_FALL),
        summer: (Dry, COLD_SUMMER),
        diurnal_range: CONTINENTAL_POLAR_TEMP_RANGE
    }
}

pub fn koppen_afam() -> Climate {
    Climate {
        name: "Tropical rainforest with monsoon".to_string(),
        general_type: 'A',
        second_type: 'F',
        third_type: '_',
        spring: (HumidDry::None, HOT_SPRING),
        winter: (HumidDry::None, HOT_WINTER),
        fall: (HumidDry::None, HOT_FALL),
        summer: (Humid, HOT_SUMMER),
        diurnal_range: EQUATOR_TEMP_RANGE
    }
}
pub fn koppen_as() -> Climate {
    Climate {
        name: "Tropical dry savanna".to_string(),
        general_type: 'A',
        second_type: 'F',
        third_type: '_',
        spring: (HumidDry::None, HOT_SPRING),
        winter: (HumidDry::None, HOT_WINTER),
        fall: (HumidDry::None, HOT_FALL),
        summer: (HumidDry::Dry, HOT_SUMMER),
        diurnal_range: EQUATOR_TEMP_RANGE
    }
}
pub fn koppen_aw() -> Climate {
    Climate {
        name: "Tropical wet savanna".to_string(),
        general_type: 'A',
        second_type: 'F',
        third_type: '_',
        spring: (HumidDry::None, HOT_SPRING),
        winter: (Humid, HOT_WINTER),
        fall: (HumidDry::None, HOT_FALL),
        summer: (HumidDry::None, HOT_SUMMER),
        diurnal_range: EQUATOR_TEMP_RANGE
    }
}
pub fn koppen_bsh() -> Climate {
    Climate {
        name: "Hot steppe".to_string(),
        general_type: 'B',
        second_type: 'S',
        third_type: 'H',
        spring: (HumidDry::None, WARM_SPRING),
        winter: (Dry, WARM_WINTER),
        fall: (HumidDry::Dry, TEMPERATE_FALL),
        summer: (HumidDry::None, HOT_SUMMER),
        diurnal_range: CONTINENTAL_POLAR_TEMP_RANGE
    }
}

pub fn koppen_bsk() -> Climate {
    Climate {
        name: "Cold steppe".to_string(),
        general_type: 'B',
        second_type: 'S',
        third_type: 'H',
        spring: (HumidDry::None, COLD_SPRING),
        winter: (Dry, COLD_WINTER),
        fall: (HumidDry::Dry, COLD_FALL),
        summer: (HumidDry::None, COLD_SUMMER),
        diurnal_range: CONTINENTAL_POLAR_TEMP_RANGE
    }
}

pub fn koppen_bwh() -> Climate {
    Climate {
        name: "Hot desert".to_string(),
        general_type: 'B',
        second_type: 'W',
        third_type: 'H',
        spring: (Dry,HOT_SPRING),
        winter: (Dry, HOT_WINTER),
        fall: (Dry, HOT_FALL),
        summer: (Dry, HOT_SUMMER),
        diurnal_range: TEMPERATE_TEMP_RANGE
    }
}
pub fn koppen_bwk() -> Climate {
    Climate {
        name: "Cold desert".to_string(),
        general_type: 'B',
        second_type: 'W',
        third_type: 'K',
        spring: (Dry, TEMPERATE_SPRING),
        winter: (Dry, COLD_WINTER),
        fall: (Dry, TEMPERATE_FALL),
        summer: (Dry, COLD_SUMMER),
        diurnal_range: TEMPERATE_TEMP_RANGE
    }
}
pub fn koppen_cfa() -> Climate {
    Climate {
        name: "Humid subtropical".to_string(),
        general_type: 'C',
        second_type: 'F',
        third_type: 'A',
        spring: (HumidDry::None, TEMPERATE_SPRING),
        winter: (Humid, TEMPERATE_WINTER),
        fall: (HumidDry::None, TEMPERATE_FALL),
        summer: (Humid, HOT_SUMMER),
        diurnal_range: TEMPERATE_TEMP_RANGE
    }
}
pub fn koppen_cfb() -> Climate {
    Climate {
        name: "Temperate oceanic".to_string(),
        general_type: 'C',
        second_type: 'F',
        third_type: 'B',
        spring: (HumidDry::None, TEMPERATE_SPRING),
        winter: (Humid, TEMPERATE_WINTER),
        fall: (HumidDry::None, TEMPERATE_FALL),
        summer: (Humid, WARM_SUMMER),
        diurnal_range: TEMPERATE_TEMP_RANGE
    }
}

pub fn koppen_cfc() -> Climate {
    Climate {
        name: "Subpolar oceanic".to_string(),
        general_type: 'C',
        second_type: 'F',
        third_type: 'C',
        spring: (HumidDry::None, TEMPERATE_SPRING),
        winter: (Humid, COLD_WINTER),
        fall: (HumidDry::None, COLD_FALL),
        summer: (Humid, TEMPERATE_WINTER),
        diurnal_range: TEMPERATE_TEMP_RANGE
    }
}

pub fn koppen_csa() -> Climate {
    Climate {
        name: "Hot-summer mediterranean".to_string(),
        general_type: 'C',
        second_type: 'S',
        third_type: 'A',
        spring: (HumidDry::None, WARM_SPRING),
        winter: (HumidDry::None, TEMPERATE_WINTER),
        fall: (HumidDry::None, TEMPERATE_FALL),
        summer: (Dry, HOT_SUMMER),
        diurnal_range: TEMPERATE_TEMP_RANGE
    }
}
pub fn koppen_csb() -> Climate {
    Climate {
        name: "Warm-summer mediterranean".to_string(),
        general_type: 'B',
        second_type: 'W',
        third_type: 'H',
        spring: (HumidDry::None, WARM_SPRING),
        winter: (HumidDry::None, TEMPERATE_WINTER),
        fall: (HumidDry::None, TEMPERATE_FALL),
        summer: (Dry, WARM_SUMMER),
        diurnal_range: TEMPERATE_TEMP_RANGE
    }
}

// pub const KOPPEN_CSC: Climate = Climate {
//     name: "Cool-summer mediterranean",
//     general_type: 'C',
//     second_type: 'S',
//     third_type: 'c',
//     winter: (None, Temperate),
//     summer: (Dry, Temperate),
// };

pub fn koppen_cwa() -> Climate {
    Climate {
        name: "Monsoon subtropical".to_string(),
        general_type: 'C',
        second_type: 'W',
        third_type: 'A',
        spring: (HumidDry::None, WARM_SPRING),
        winter: (Dry, TEMPERATE_WINTER),
        fall: (HumidDry::Dry, TEMPERATE_FALL),
        summer: (Humid, HOT_SUMMER),
        diurnal_range: TEMPERATE_TEMP_RANGE
    }
}
pub fn koppen_cwb() -> Climate {
    Climate {
        name: "Subtropical highland".to_string(),
        general_type: 'C',
        second_type: 'W',
        third_type: 'B',
        spring:(HumidDry::Dry, WARM_SPRING),
        winter: (Dry, TEMPERATE_WINTER),
        fall: (HumidDry::None, TEMPERATE_FALL),
        summer: (Humid, WARM_SUMMER),
        diurnal_range: CONTINENTAL_POLAR_TEMP_RANGE
    }
}

pub fn koppen_cwc() -> Climate {
    Climate {
        name: "Cold subtropical highland".to_string(),
        general_type: 'C',
        second_type: 'W',
        third_type: 'C',
        spring: (HumidDry::None, TEMPERATE_SPRING),
        winter: (Dry, TEMPERATE_WINTER),
        fall: (HumidDry::Dry, COLD_FALL),
        summer: (Humid, COLD_SUMMER),
        diurnal_range: CONTINENTAL_POLAR_TEMP_RANGE
    }
}
pub fn koppen_dfa() -> Climate {
    Climate {
        name: "Hot humid continental".to_string(),
        general_type: 'D',
        second_type: 'F',
        third_type: 'A',
        spring: (HumidDry::Dry, TEMPERATE_SPRING),
        winter: (Humid, COLD_WINTER),
        fall: (HumidDry::Dry, TEMPERATE_FALL),
        summer: (Humid, HOT_SUMMER),
        diurnal_range: CONTINENTAL_POLAR_TEMP_RANGE
    }
}

pub fn koppen_dfb() -> Climate {
    Climate {
        name: "Warm humid continental".to_string(),
        general_type: 'D',
        second_type: 'F',
        third_type: 'B',
        spring: (HumidDry::Dry, TEMPERATE_SPRING),
        winter: (Humid, COLD_WINTER),
        fall: (HumidDry::Dry, TEMPERATE_FALL),
        summer: (Humid, WARM_SUMMER),
        diurnal_range: TEMPERATE_TEMP_RANGE
    }
}

pub fn koppen_dfc() -> Climate {
    Climate {
        name: "Subarctic".to_string(),
        general_type: 'D',
        second_type: 'F',
        third_type: 'C',
        spring: (HumidDry::None, TEMPERATE_SPRING),
        winter: (Humid, COLD_WINTER),
        fall: (HumidDry::None, COLD_FALL),
        summer: (Humid, COLD_SUMMER),
        diurnal_range: CONTINENTAL_POLAR_TEMP_RANGE
    }
}

pub fn koppen_dsa() -> Climate {
    Climate {
        name: "Hot continental".to_string(),
        general_type: 'D',
        second_type: 'S',
        third_type: 'A',
        spring: (HumidDry::Dry, HOT_SPRING),
        winter: (HumidDry::None, COLD_WINTER),
        fall: (HumidDry::None, COLD_FALL),
        summer: (Dry, HOT_SUMMER),
        diurnal_range: CONTINENTAL_POLAR_TEMP_RANGE
    }
}

pub fn koppen_dsb() -> Climate {
    Climate {
        name: "Warm continental".to_string(),
        general_type: 'D',
        second_type: 'S',
        third_type: 'B',
        spring: (Dry, WARM_SPRING),
        winter: (HumidDry::None, COLD_WINTER),
        fall: (HumidDry::None, COLD_FALL),
        summer: (Dry, WARM_SUMMER),
        diurnal_range: CONTINENTAL_POLAR_TEMP_RANGE
    }
}

pub fn koppen_dsc() -> Climate {
    Climate {
        name: "Dry subarctic".to_string(),
        general_type: 'D',
        second_type: 'S',
        third_type: 'C',
        spring: (HumidDry::Dry, COLD_SPRING),
        winter: (HumidDry::None, COLD_WINTER),
        fall: (HumidDry::None, COLD_FALL),
        summer: (Dry, COLD_SUMMER),
        diurnal_range: CONTINENTAL_POLAR_TEMP_RANGE
    }
}
use savefile::prelude::*;
use savefile_derive::*;
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GenData {
    pub index: (i32, i32, i32),
    pub temperature: Vec<OrderedFloat<f64>>,
    pub altitude: f64,
    pub pressure: Vec<OrderedFloat<f64>>,
    pub humidity: Vec<OrderedFloat<f64>>,
    pub wind: Vec<(f32, f32, f32)>,
    pub td: Vec<f64>,
}

impl GenData {

    pub fn calculate_temperature(
        position: (i32, i32, i32),
        noise: Fbm<Perlin>,
        altitude: f64,
        base_temp: f64,
        offset: Vector3<f32>,
    ) -> f64 {
        let _noise_factor = noise.get([
            position.0 as f64 * offset.x as f64,
            position.1 as f64 * offset.y as f64,
            position.2 as f64 * offset.z as f64,
        ]) * 5.;

        // Fórmula: rT - (m/100 * 0.6) = T

        base_temp - ((altitude / 100.) * 0.6) /* + (noise_factor) as f32 */
    }

    pub fn calculate_pressure(altitude: f32, temperature: f32, land_pressure: f32) -> f32 {
        let frac = (0.0065 * altitude) / (temperature + (0.0065 * altitude) + 273.15);

        land_pressure * (1.0 - frac).powf(5.257) // land pressure is given in hectopascals
    }

    pub fn calculate_rel_hum(
        temperature: f32,
        factor: f32,
    ) -> (f32, f32) {
        let mut td: f32 = 0.0;

        td = temperature - (factor.abs());

        // if is_prec {
        //     td = tdprev;
        // }

        let frac1 = E.powf((17.625 * td) / (243.04 + td));

        let frac2 = E.powf((17.625 * temperature) / (243.04 + temperature));

        let hum = 100.0 * (frac1 / frac2);

        (hum, td)
    }

    #[deny(clippy::eq_op)]
    fn factor(season: Season, climate: &Climate, t: f64) -> f64 {
        let mut rng = thread_rng();
        let r: Range<f64>;
        if season == Season::Summer {
            r = match climate.summer.0 {
                Humid => 0.0..(0.5 * t.abs()),
                Dry => (0.75 * t)..t.abs(),
                HumidDry::None => 0.0..t.abs()
            };
        } else if season == Season::Winter {
            r = match climate.winter.0 {
                Humid => 0.0..(0.5 * t.abs()),
                Dry => (0.75 * t)..t.abs(),
                HumidDry::None => 0.0..t.abs()
            };
        } else if season == Season::Fall {
            r = match climate.fall.0 {
                Humid => 0.0..(0.5 * t.abs()),
                Dry => (0.75 * t)..t.abs(),
                HumidDry::None => 0.0..t.abs()
            };
        } else if season == Season::Spring {
            r = match climate.spring.0 {
                Humid => 0.0..(0.5 * t.abs()),
                Dry => (0.75 * t)..t.abs(),
                HumidDry::None => 0.0..t.abs()
            };
        }
        else {
            r = (0.70 * t)..t.abs();
        }

        rng.gen_range(r)

    }


    pub fn gen_year_data(
        latitude: i32,
        altitude: f64,
        index: (i32, i32, i32),
        noise: Fbm<Perlin>,
        climate: Climate
    ) -> GenData {
        let mut temperature_vec: Vec<OrderedFloat<f64>> = vec![];
        let mut pressure_vec: Vec<OrderedFloat<f64>> = vec![];
        let mut wind_vec: Vec<(f32, f32, f32)> = vec![];
        let mut hum_vec: Vec<OrderedFloat<f64>> = vec![];
        let mut td_vec: Vec<f64> = vec![];
        let mut current_season = Season::Winter;

        let night_variation_range = &climate.diurnal_range;

        //TODO
        // let curve: Gd<Curve> = load("resources/diurnal_temp_curve.tres");

        use flo_curves::*;


        let point1 = Coord2(0.0, 0.3);
        let point2 = Coord2(0.25, 0.0);
        let point3 = Coord2(0.63, 1.0);
        // let point4 = Coord2(0.84, 0.25);
        let point5 = Coord2(1.0, 0.15);

        let curve = flo_curves::bezier::Curve {
            start_point: (point1),
            end_point: (point5),
            control_points: (point2, point3),
        };

        let mut rng = thread_rng();

        for day in 1..=360 {
            match day {
                1..=90 => current_season = Season::Winter,
                91..=180 => current_season = Season::Spring,
                181..=270 => current_season = Season::Summer,
                271..=360 => current_season = Season::Fall,
                _ => current_season = Season::Winter,
            }

            let base_temp_range = match current_season {
                Season::Fall => climate.fall.1.clone(),
                Season::Spring => climate.spring.1.clone(),
                Season::Summer => climate.summer.1.clone(),
                Season::Winter => climate.winter.1.clone()
            };

            let night_val = rng.gen_range(night_variation_range.clone());
            let base_temp = rng.gen_range(base_temp_range);

            // ---- TEMPERATURE ---- //

            let day_temp = Self::calculate_temperature(
                index,
                noise.clone(),
                altitude,
                base_temp as f64,
                Vector3::new(0.1, 0.1, 0.1),
            );

            let night_temp = day_temp - (night_val as f64);

            for hour in 1..=24 {
                let factor = curve.point_at_pos(0.042 * hour as f64).1; // 0.042 -> 1h
                let temp = (day_temp * factor) + (night_temp * (1.0 - factor));
                temperature_vec.push(OrderedFloat(temp));
            }

            // ---- PRESSURE ---- //

            let base_pressure = rng.gen_range(995..=1060);

            for hour in 1..=24 {
                let index = hour * day;
                let temp_value = temperature_vec.get(index - 1).unwrap();
                let pres = Self::calculate_pressure(altitude as f32, temp_value.0 as f32, base_pressure as f32);
                pressure_vec.push(OrderedFloat(pres as f64));
            }

            // ---- WIND ---- //

            for hour in 1..=24 {
                let index = hour * day;

                let pressure = pressure_vec.get(index - 1).unwrap();
                let wind = (
                    pressure.to_degrees().cos() as f32,
                    pressure.to_degrees().sin() as f32,
                    pressure.to_degrees().cos() as f32
                );
                wind_vec.push(wind);
            }



            for hour in 1..=24 {
                let t = temperature_vec.get((hour * day) - 1).unwrap().0;
                let factor = Self::factor(current_season.clone(), &climate, t.clone());
                // let water = altitude <= -0.6;
                let rel = Self::calculate_rel_hum(
                    t.clone() as f32,
                    factor as f32,
                );
                hum_vec.push(OrderedFloat(rel.0 as f64));
                td_vec.push(rel.1 as f64);
            }
        }
        GenData {
            index,
            temperature: temperature_vec,
            altitude,
            pressure: pressure_vec,
            humidity: hum_vec,
            wind: wind_vec,
            td: td_vec,
        }
    }

    pub fn save_data(data: GenData) -> std::io::Result<()> {
        let file_name = format!("data/weather_grid_data/{}_{}_{}.ron", data.index.0, data.index.1, data.index.2);
        let mut file = File::create(file_name)?;
        file.write_all(ron::ser::to_string_pretty(&data, PrettyConfig::default()).unwrap().as_ref())
    }
}

impl GridComponent {
    pub fn generate_data(&mut self, latitude: i32, climate: &str, _td: f32) -> GenData {
        let mut fetched: Climate = koppen_et();
        let climates: [Climate; 18] = [koppen_cfa(), koppen_cfb(), koppen_cfc(), koppen_dfb(), koppen_dfc(), koppen_dfa(), koppen_cwc(), koppen_cwb(), koppen_cwa(), koppen_et(), koppen_afam(), koppen_as(), koppen_aw(), koppen_dsc(), koppen_bsh(), koppen_bsk(), koppen_bwh(), koppen_bwk()];
        for climate_iter in climates {
            let formatted = format!("{}{}{}", climate_iter.general_type, climate_iter.second_type, climate_iter.third_type);
            let name = formatted.as_str();
            let formatted_two = climate.to_string();
            let typed_name = formatted_two.as_str();
            if typed_name == name {
                fetched = climate_iter;
                break
            } else {
                continue
            }
        }
        let noise: Fbm<Perlin> = Fbm::new(345435);
        let gen_data = GenData::gen_year_data(latitude, self.mean_altitude as f64, (self.index.x, self.index.y, self.index.z), noise, fetched);
        gen_data
    }
    pub fn save_data(data: &GenData) {
        GenData::save_data(data.clone()).expect("Error exporting GenData!");
    }
}