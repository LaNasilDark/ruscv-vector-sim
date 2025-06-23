use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SimulatorConfig {
    pub function_units: FunctionUnits,
    pub memory_units: MemoryUnits,
}

#[derive(Debug, Deserialize)]
pub struct FunctionUnits {
    pub interger_alu: Unit,
    pub interger_multiplier: Unit,
    pub float_alu: Unit,
    pub float_multiplier: Unit,
    pub interger_divider: Unit,
    pub float_divider: Unit,
    pub branch_unit: Unit,
}

#[derive(Debug, Deserialize)]
pub struct Unit {
    pub latency: u32,
}

#[derive(Debug, Deserialize)]
pub struct MemoryUnits {
    pub load_store_unit: LoadStoreUnit,
}

#[derive(Debug, Deserialize)]
pub struct LoadStoreUnit {
    pub latency: u32,
    pub max_access_width: u32,
}

impl SimulatorConfig {
    /// 从指定路径加载 TOML 配置文件
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
}