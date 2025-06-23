

pub struct LoadStoreUnit {
    latency: u32,
    max_access_width: u32,
}

impl LoadStoreUnit {
    pub fn new(latency: u32, max_access_width: u32) -> LoadStoreUnit {
        LoadStoreUnit {
            latency,
            max_access_width,
        }
    }

    pub fn new_from_config(config: &crate::config::LoadStoreUnit) -> LoadStoreUnit {
        LoadStoreUnit::new(config.latency, config.max_access_width)
    }
}