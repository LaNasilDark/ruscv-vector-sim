use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SimulatorConfig {
    functional_units: FunctionalUnits,
    memory_units: MemoryUnits,
}

#[derive(Debug, Deserialize)]
pub struct FunctionalUnits {
    interger_alu: Unit,
    interger_multiplier: Unit,
    float_alu: Unit,
    float_multiplier: Unit,
    interger_divider: Unit,
    float_divider: Unit,
    branch_unit: Unit,
}

#[derive(Debug, Deserialize)]
pub struct Unit {
    latency: u32,
}

#[derive(Debug, Deserialize)]
pub struct MemoryUnits {
    load_store_unit: LoadStoreUnit,
}

#[derive(Debug, Deserialize)]
pub struct LoadStoreUnit {
    latency: u32,
    max_access_width: u32,
}

impl SimulatorConfig {
    /// 从指定路径加载 TOML 配置文件
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
}