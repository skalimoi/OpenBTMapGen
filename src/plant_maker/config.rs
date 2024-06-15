


use image_crate::ImageBuffer;
use image_crate::Luma;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::plant_maker::edaphology::calculate_soil_depth;
use crate::plant_maker::hydrology::calculate_hydrology_map;
use crate::plant_maker::insolation::calculate_actual_insolation;
use crate::plant_maker::orography::calculate_normal_map;
use crate::plant_maker::probabilities::calculate_probabilities;
use crate::soil_def::{VegetationCollection, VegetationMaps};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GreyscaleImage<T> {
    pub image: Vec<T>,
    pub len: usize,
}

impl<T> GreyscaleImage<T> {
    pub fn new(image: Vec<T>) -> Self {
        let len = (image.len() as f64).sqrt() as usize;
        Self { image, len }
    }
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<T> std::ops::Index<(usize, usize)> for GreyscaleImage<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.image.index(index.0 + index.1 * self.len)
    }
}

impl<T> std::ops::IndexMut<(usize, usize)> for GreyscaleImage<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.image.index_mut(index.0 + index.1 * self.len)
    }
}

#[derive(Deserialize, Serialize)]
pub struct Map {
    pub biom: String,
    pub height_map_path: GreyscaleImage<f64>,
    pub texture_map_path: Vec<u8>,
    pub height_conversion: f64, // The factor to convert a height value of the height-map to the actual height
    pub max_soil_depth: f64,    // in cm, states the maximal depth the ground can have when it has no tilt
    pub pixel_size: f64,        // the size that a pixel covers of the real map in m
}

#[derive(Deserialize)]
pub struct Biom {
    // this value corresponds to the diffuse sun beam scattering by the atmosphere
    pub atmospheric_diffusion: f64,  // in percent
    pub atmospheric_absorption: f64, // in percent
    pub cloud_reflection: f64,       // in percent
    pub avg_rainfall_per_day: f64,   // in l/cm²
    pub groundwater: f64,            // in l/cm²
}

#[derive(Deserialize)]
pub struct Soil {
    pub id: u8,
    pub albedo: f64,
    pub water_absorption: f64,
}

#[derive(Deserialize)]
pub struct Vegetation {
    pub energy_demand: f64,
    pub water_demand: f64,
    pub soil_demand: String,
    pub soil_depth_demand: f64,
}

pub struct Sun {
    pub azimuth: f64,
    pub elevation: f64,
}

impl Sun {
    pub fn convert_to_uv_coordinates(&self) -> Vector3<f64> {
        let u = self.azimuth.to_radians().cos() * self.elevation.to_radians().cos();
        let v = self.azimuth.to_radians().sin() * self.elevation.to_radians().cos();
        let w = self.elevation.to_radians().sin();
        Vector3::new(round(u, 4), round(v, 4), round(w, 4))
    }
}

pub fn round(x: f64, n: i32) -> f64 {
    let p = 10_f64.powi(n);
    (x * p).round() / p
}

pub fn clamp_idx(c: usize, o: i32, len: usize) -> usize {
    (c as i32 + o).clamp(0, len as i32) as usize
}

pub struct SunConfig {
    pub daylight_hours: i32,
    pub sun_start_elevation: f64,
    pub sun_start_azimuth: f64,
    pub sun_max_elevation: f64,
}

pub struct SimArgs<'a> {
    pub height_map: GreyscaleImage<f64>,
    pub soil_ids_map: Vec<u8>,
    pub soils: &'a HashMap<u8, Soil>,
    pub sun_config: &'a SunConfig,
    pub reflection_coefficient: f64,
    pub map: &'a Map,
    pub vegetation: &'a Vegetation,
    pub biom: &'a Biom,
}

#[derive(Deserialize)]
pub struct SimConfig {
    maps: Map,
    bioms: HashMap<String, Biom>,
    #[serde(deserialize_with = "deserialize_soils")]
    soil_ids: HashMap<u8, Soil>,
    soil_names: HashMap<String, u8>,
    vegetations: HashMap<String, Vegetation>,
}

fn deserialize_soils<'de, D>(deserializer: D) -> Result<HashMap<u8, Soil>, D::Error>
    where
        D: serde::Deserializer<'de>,
{
    let s: HashMap<String, Soil> = Deserialize::deserialize(deserializer)?;
    Ok(s.into_values().map(|soil| (soil.id, soil)).collect())
}

impl SimConfig {
    pub fn from_configs(
        maps: Map,
        bioms: HashMap<String, Biom>,
        soils: HashMap<String, Soil>,
        vegetations: HashMap<String, Vegetation>,
    ) -> Self {
        let (soil_names, soil_ids): (HashMap<String, u8>, HashMap<u8, Soil>) = soils
            .into_iter()
            .map(|(name, soil)| ((name, soil.id), (soil.id, soil)))
            .unzip();
        Self {
            maps,
            bioms,
            soil_ids,
            soil_names,
            vegetations,
        }
    }
    pub fn calculate_maps(
        &self,
        sun_config: &SunConfig,
        reflection_coefficient: f64,
        mapdata: &mut VegetationMaps
    ) {
        let map = &self.maps;

        let height_map_for_insolation = &map.height_map_path.clone();

        let soil_ids_map = &map.texture_map_path.clone();

        let sim_args = SimArgs {
            height_map: height_map_for_insolation.clone(),
            soil_ids_map: soil_ids_map.clone(),
            soils: &self.soil_ids,
            sun_config,
            reflection_coefficient,
            map,
            vegetation: &Vegetation {
                energy_demand: 0.0,
                water_demand: 0.0,
                soil_demand: "".to_string(),
                soil_depth_demand: 0.0,
            },
            biom: &self.bioms[&map.biom],
        };
        let sim_args_for_insolation = SimArgs {
            height_map: height_map_for_insolation.clone(),
            soil_ids_map: soil_ids_map.clone(),
            soils: &self.soil_ids,
            sun_config,
            reflection_coefficient,
            map,
            vegetation: &Vegetation {
                energy_demand: 0.0,
                water_demand: 0.0,
                soil_demand: "".to_string(),
                soil_depth_demand: 0.0,
            },
            biom: &self.bioms[&map.biom],
        };
        let insolation_map = calculate_actual_insolation(&sim_args_for_insolation);
        let orographic_map = calculate_normal_map(&sim_args);
        let edaphic_map = calculate_soil_depth(&orographic_map, sim_args.map);
        let hydrology_map = calculate_hydrology_map(&sim_args, &edaphic_map, &insolation_map);

        mapdata.insolation = insolation_map;
        mapdata.edaphology = edaphic_map;
        mapdata.hydrology = hydrology_map;
        mapdata.orography = orographic_map;
        //std::fs::write("insolation_rust.json", serde_json::to_string(&insolation_map.image).unwrap()).unwrap();

        // let orographic_image = ImageBuffer::<Luma<u16>, Vec<u16>>::from_raw(
        //     orographic_map.len() as u32,
        //     orographic_map.len() as u32,
        //     orographic_map.image,
        // )
        // .unwrap();
    }

    // TODO: HACER STRUCT SOLO PARA ESTAS IMAGENES Y NO TENER QUE GUARDARLAS
    // TODO: COMPROBAR EL TEMA IMAGEN SI LO COGE BIEN O NO
    // TODO: TERMINAR DE CONFIGURAR SOIL_DEF.RS
    pub fn calculate_probabilities(&self, mapdata: &mut VegetationMaps, vegetation_names: &[&str], _daylight_hours: i32, vegetation_collection: &mut VegetationCollection) {
        let soil_ids_map = GreyscaleImage::new(self.maps.texture_map_path.clone());
        let x: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_raw(1024, 1024, self.maps.texture_map_path.clone()).unwrap();
        x.save("soil_veg_test.png");
        for vegetation in vegetation_names {
            let probabilities_map = calculate_probabilities(
                &self.vegetations[*vegetation],
                &soil_ids_map,
                &self.soil_names,
                &mapdata.insolation,
                &mapdata.edaphology,
                &mapdata.hydrology,
            );
            let mut probabilities_image = ImageBuffer::<Luma<u8>, Vec<u8>>::from_raw(
                probabilities_map.len() as u32,
                probabilities_map.len() as u32,
                probabilities_map.image.into_iter().map(|x| (x * 1000.0) as u8).collect(),
            )
                .unwrap();
            imageproc::contrast::stretch_contrast_mut(&mut probabilities_image, 100, 255);
            vegetation_collection.generated.insert(vegetation.parse().unwrap(), probabilities_image.into_raw());
        }
    }
}