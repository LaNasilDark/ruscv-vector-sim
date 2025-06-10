pub struct VectorConfig {
    pub vl :  usize, // number of elments, same defination with 'vl' in the spec
    pub sew : usize, // selected element width (unit is bit)
    pub lmul: usize, // vector register group multiplier

}

pub struct HardwareConfig{
    pub vlen : usize, // bits of a single vector register
    pub lane_number : usize 
}


pub struct Configuration {
    pub vector_config : VectorConfig,
    pub hardware_config : HardwareConfig
}
impl VectorConfig {
    pub fn new(vl : usize, sew : usize, lmul : usize) -> VectorConfig {
        VectorConfig {
            vl,
            sew,
            lmul
        }
    }

    pub fn bytes_per_element(&self) -> usize {
        self.sew / 8
    }

    pub fn total_length(&self) -> usize {
        self.vl * self.bytes_per_element()
    }
}

impl HardwareConfig {
    pub fn new(vlen : usize, lane_number : usize) -> HardwareConfig {
        HardwareConfig {
            vlen,
            lane_number
        }
    }

}

impl Configuration {
    pub fn new(vector_config : VectorConfig, hardware_config : HardwareConfig) -> Configuration {
        Configuration {
            vector_config,
            hardware_config
        }
    }
}