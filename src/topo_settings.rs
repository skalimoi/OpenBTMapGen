#[derive(Copy, Clone, Debug)]
pub enum NoiseTypesUi {
    Simplex,
    Perlin,
    BillowPerlin
}

#[derive(Copy, Clone, Debug)]
pub struct TopoSettings {
    pub seed: Option<u32>,
    pub noise_type: Option<NoiseTypesUi>,
    pub noise_octaves: Option<u32>,
    pub noise_frequency: Option<f64>,
    pub noise_lacunarity: Option<f64>,
    pub mountain_pct: f64,
    pub sea_pct: f64,
    pub min_height: i32,
    pub max_height: i32,
    pub erosion_cycles: u64,

}

impl TopoSettings {
    pub fn set_seed(&mut self, seed: Option<u32>) {
        self.seed = seed;
    }
    pub fn set_type(&mut self, noise_type: Option<NoiseTypesUi>) {
        self.noise_type = noise_type;
    }
    pub fn set_octaves(&mut self, oct: Option<u32>) {
        self.noise_octaves = oct;
    }
    pub fn set_frequency(&mut self, freq: Option<f64>) {
        self.noise_frequency = freq;
    }
    pub fn set_lacunarity(&mut self, lac: Option<f64>) {
        self.noise_lacunarity = lac;
    }
    
    pub fn set_mtn_pct(&mut self, pct: f64) { self.mountain_pct = pct}
    pub fn set_sea_pct(&mut self, pct: f64) { self.sea_pct = pct}

    pub fn set_cycles(&mut self, cycles: u64) { self.erosion_cycles = cycles }



}