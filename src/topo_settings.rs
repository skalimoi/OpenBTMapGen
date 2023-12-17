#[derive(Copy, Clone)]
pub enum NoiseTypesUi {
    Simplex,
    Perlin,
    BillowPerlin
}

#[derive(Copy, Clone)]
pub struct TopoSettings {
    pub seed: Option<u64>,
    pub noise_type: Option<NoiseTypesUi>,
    pub noise_octaves: Option<u64>,
    pub noise_frequency: Option<u64>,
    pub noise_lacunarity: Option<u64>,
    pub mountain_pct: u8,
    pub sea_pct: u8,
    pub min_height: i32,
    pub max_height: i32,
    pub erosion_cycles: u64

}

impl TopoSettings {


}