use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use fltk::browser::CheckBrowser;
use fltk::prelude::{BrowserExt, MenuExt};
use image_old::{ImageBuffer, Luma};
use image_old::imageops::FilterType;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use crate::FileData;
use crate::plant_maker::config::{Biom, GreyscaleImage, Map, SimConfig, Soil, SunConfig, Vegetation};

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum SoilType {
    Dirt,
    Silt,
    Stone,
    Loam,
    Clay,
    Sand,
    Gravel,
    Placeholder1,
    Placeholder2,
    Placeholder3,
    Placeholder4,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VegetationData {
    pub base: SoilType,
    pub blocklist: HashMap<SoilType, bool>,
    pub vegetationlist: HashMap<String, bool>
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VegetationMaps {
    pub insolation: GreyscaleImage<f64>,
    pub edaphology: GreyscaleImage<f64>,
    pub hydrology: GreyscaleImage<f64>,
    pub orography: GreyscaleImage<Vector3<f64>>,
}



#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VegetationCollection {
    pub generated: HashMap<String, Vec<u8>>
}

pub fn generate_selected_do(c: &mut CheckBrowser, veglist: HashMap<String, Vegetation>, soils: HashMap<String, Soil>, filedata: &mut FileData) {
    let mut to_be_generated: Vec<String> = vec![];
    for item in 0..=c.nitems() {
        if c.checked(item as i32) {
            to_be_generated.push(c.text(item as i32).unwrap());
        }
    }
    let h: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::from_raw(8192, 8192, filedata.eroded_full.clone()).unwrap();
    let h = image_old::imageops::resize(&h, 512, 512, FilterType::CatmullRom);
    let h = GreyscaleImage::new(h.into_raw().into_iter()
    .map(|x| x as f64)
    .collect());
    let s: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_raw(8192, 8192, filedata.soil.clone()).unwrap();
    let s = image_old::imageops::resize(&s, 512, 512, FilterType::Nearest);
    let m = Map {
        biom: "PolarZone".parse().unwrap(),
        height_map_path: h,
        texture_map_path: s.into_raw(),
        height_conversion: 1.0,
        max_soil_depth: 300.0,
        pixel_size: 100.0
    };
    let mut data = String::new();
    File::open("bioms.yml").unwrap().read_to_string(&mut data).unwrap();
    let bioms: HashMap<String, Biom> = serde_yaml::from_str(&data).unwrap();
    // File::open("vegetation_types.yaml")
    //     .unwrap()
    //     .read_to_string(&mut data)
    //     .unwrap();
    let sun_config = SunConfig { // sample parameters for Hellion
        daylight_hours: 13,
        sun_start_elevation: -5.0,
        sun_start_azimuth: 92.0,
        sun_max_elevation: 50.0,
    };
    let sim_config = SimConfig::from_configs(m, bioms, soils, veglist);
    
    let reflection_coefficient = 0.1;
    
    if filedata.datamaps.insolation.image.is_empty() {
        sim_config.calculate_maps(&sun_config, reflection_coefficient, &mut filedata.datamaps);
    }
    
    sim_config.calculate_probabilities(&mut filedata.datamaps, to_be_generated.as_slice(), sun_config.daylight_hours, &mut filedata.vegetation_maps);
}