use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum NoiseTypesUi {
    Simplex,
    Perlin,
    BillowPerlin
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct TopoSettings {
    pub max_alt: f64,
    pub seed: Option<u32>,
    pub min_bound: (f64, f64),
    pub max_bound: (f64, f64),
    pub lod: f64,
    pub erod_scale: f64,
    pub mountain_pct: f64,
    pub sea_pct: f64,
    pub min_height: i32,
    pub max_height: i32,
    pub erosion_cycles: u64,

}

impl TopoSettings {
    pub fn set_min_bounds(&mut self, x_y: f64) { self.min_bound = (x_y, x_y) }
    pub fn set_max_bounds(&mut self, x_y: f64) { self.max_bound = (x_y, x_y) }
    pub fn set_lod(&mut self, x: f64) { self.lod = x }
    pub fn set_erod_scale(&mut self, y: f64) { self.erod_scale = y }
    pub fn set_seed(&mut self, seed: Option<u32>) {
        self.seed = seed;
    }
    pub fn set_mtn_pct(&mut self, pct: f64) { self.mountain_pct = pct}
    pub fn set_sea_pct(&mut self, pct: f64) { self.sea_pct = pct}

    pub fn set_cycles(&mut self, cycles: u64) { self.erosion_cycles = cycles }



}