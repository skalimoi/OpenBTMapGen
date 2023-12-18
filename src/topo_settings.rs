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
    pub mountain_pct: u8,
    pub sea_pct: u8,
    pub min_height: i32,
    pub max_height: i32,
    pub erosion_cycles: u64

}

impl TopoSettings {


}