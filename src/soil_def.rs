use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use fltk::browser::CheckBrowser;
use fltk::button::Button;
use fltk::prelude::{BrowserExt, MenuExt};
use serde::{Deserialize, Serialize};
use crate::soil::config::Vegetation;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SoilType {
    Dirt,
    Silt,
    Stone,
    Loam,
    Clay,
    Sand,
    Gravel
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VegetationData {
    pub base: SoilType,
    pub blocklist: HashMap<bool, SoilType>,
    pub vegetationlist: HashMap<bool, String>
}

// one must create as many as needed
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VegetationMaps {
    map: Vec<Vec<u8>>
}

// TODO: SIGUE EN CONFIG.RS

pub fn generate_selected_do(c: &mut CheckBrowser, w: &mut Button, data: &mut VegetationData, filedata: &mut FileData, soilmap: Vec<u8>) {
    collect_values(c, data);
    let h: ImageBuffer = ImageBuffer::from_raw(filedata.eroded_full, 8192, 8192).unwrap();
    let h = image_latest::imageops::resize(h, 1024, 1024, FilterType::Nearest);
    let h = GreyScaleImage::new(h.into_raw().into_iter()
    .map(|x| x as f64)
    .collect());
    let m = Map {
        biom: TemperateZone,
        height_map_path: h,
        texture_map_path: soilmap,
        height_conversion: 0.2,
        max_soil_depth: 300.0,
        pixel_size: 100.0
    };
    let conf = SimConfig::from_configs(

    )
}

pub fn collect_values(w: &mut CheckBrowser, data: &mut VegetationData) {
    let nitems = w.nitems();
    for i in 0..nitems {
        data.vegetationlist.clear();
        data.vegetationlist.insert(w.checked(i as i32), w.text(i as i32).unwrap());
    }
}

//TODO init soil choice on main to keep track of state

pub fn base_choice_init(w: &mut impl MenuExt) {
    w.add_choice("Dirt");
    w.add_choice("Loam");
    w.add_choice("Silt");
    w.add_choice("Clay");
    w.add_choice("Stone");
    w.add_choice("Sand");
    w.add_choice("Gravel");
}

pub fn load_and_show_veg(w: &mut CheckBrowser) {
    let mut data = String::new();
    File::open("vegetation_types.yaml")
        .unwrap()
        .read_to_string(&mut data)
        .unwrap();
    let vegetations: HashMap<String, Vegetation> = serde_yaml::from_str(&data).unwrap();
    for vegetation in vegetations.iter() {
        w.add(vegetation.0.clone().as_str(), false);
    }
}