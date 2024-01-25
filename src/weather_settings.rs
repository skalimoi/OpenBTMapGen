use crate::topo_settings::NoiseTypesUi;
use crate::weather::Climate;

#[derive(Clone)]
pub struct WeatherSettings {
    pub seed: Option<u32>,
    pub koppen: Option<Climate>,
    pub latitude: u32,
    pub grid_size: u8,
}

impl WeatherSettings {
    pub fn set_latitude(&mut self, latitude: u32) {
        self.latitude = latitude;
    }
    pub fn set_seed(&mut self, seed: u32) {
        self.seed = Some(seed);
    }
    pub fn set_climate(&mut self, climate: Climate) {
        self.koppen = Some(climate);
    }
    pub fn set_grid_size(&mut self, size: u8) {
        self.grid_size = size;
    }

}