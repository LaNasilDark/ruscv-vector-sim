use std::collections::HashMap;
use serde::Deserialize;
use std::sync::RwLock;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use lazy_static::lazy_static;

use crate::sim::unit::function_unit::FunctionUnitKeyType;

lazy_static! {
    static ref CONFIG: RwLock<Option<SimulatorConfig>> = RwLock::new(None);
}

#[derive(Debug, Deserialize, Clone)]
pub struct SimulatorConfig {
    pub function_units: FunctionUnits,
    pub memory_units: MemoryUnits,
    pub vector_config: VectorConfig,
    pub vector_register: VectorRegister,
    pub buffer: BufferConfig,  // 添加buffer配置
    pub register: RegisterConfig,  // 添加register配置
}

#[derive(Debug, Deserialize, Clone)]
pub struct FunctionUnits {
    pub interger_alu: Unit,
    pub interger_multiplier: Unit,
    pub float_alu: Unit,
    pub float_multiplier: Unit,
    pub interger_divider: Unit,
    pub float_divider: Unit,
    pub branch_unit: Unit,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Unit {
    pub latency: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MemoryUnits {
    pub load_store_unit: LoadStoreUnit,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoadStoreUnit {
    pub latency: u32,
    pub max_access_width: u32,
    pub read_ports_limit: u32,
    pub write_ports_limit: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VectorConfig {
    pub software: SoftwareConfig,
    pub hardware: HardwareConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VectorRegister {
    pub ports: VectorRegisterPorts,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VectorRegisterPorts {
    pub read_ports_limit: u32,
    pub write_ports_limit: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BufferConfig {
    pub input_maximum_size: u32,  // 输入缓冲区最大大小（字节）
    pub result_maximum_size : u32 // 结果缓冲区最大大小（字节）
}

impl VectorConfig {
    /// 获取向量寄存器的总字节数
    pub fn get_vector_register_bytes(&self) -> u32 {
        self.hardware.vlen / 8 // 将位转换为字节
    }

    pub fn get_vector_register_using_bytes(&self) -> u32 {
        (self.software.sew / 8) * self.software.vl  // 将sew从位转换为字节
    }
    
    /// 获取向量寄存器的元素数量
    pub fn get_vector_elements_count(&self) -> u32 {
        self.software.vl
    }
    
    /// 获取向量元素的字节数
    pub fn get_element_bytes(&self) -> u32 {
        self.software.sew / 8 // 将位转换为字节
    }
    
    /// 获取向量操作的总字节数
    pub fn get_total_bytes(&self) -> u32 {
        self.get_vector_elements_count() * self.get_element_bytes()
    }
    
    /// 检查向量配置是否有效
    pub fn is_valid(&self) -> bool {
        // 检查vl * sew <= vlen
        (self.software.vl as u64 * self.software.sew as u64) <= self.hardware.vlen as u64
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SoftwareConfig {
    pub vl: u32,   // Vector Length，向量长度寄存器值
    pub sew: u32,  // Scalar Element Width，标量元素宽度（位）
    pub lmul: u32, // Lane Multiplier，通道乘数
}

#[derive(Debug, Deserialize, Clone)]
pub struct HardwareConfig {
    pub vlen: u32,       // 向量寄存器长度（位）
    pub lane_number: u32, // 向量处理通道数量
}

impl Default for Unit {
    fn default() -> Self {
        Unit {
            latency: 1,
        }
    }
}

impl Default for LoadStoreUnit {
    fn default() -> Self {
        LoadStoreUnit {
            latency: 1,
            max_access_width: 64,
            read_ports_limit: 2,
            write_ports_limit: 2,
        }
    }
}

impl Default for MemoryUnits {
    fn default() -> Self {
        MemoryUnits {
            load_store_unit: LoadStoreUnit::default(),
        }
    }
}

impl Default for FunctionUnits {
    fn default() -> Self {
        FunctionUnits {
            interger_alu: Unit::default(),
            interger_multiplier: Unit::default(),
            float_alu: Unit::default(),
            float_multiplier: Unit::default(),
            interger_divider: Unit::default(),
            float_divider: Unit::default(),
            branch_unit: Unit::default(),
        }
    }
}

impl Default for SoftwareConfig {
    fn default() -> Self {
        SoftwareConfig {
            vl: 64,   // 默认向量长度
            sew: 32,  // 默认标量元素宽度（位）
            lmul: 1,  // 默认通道乘数
        }
    }
}

impl Default for HardwareConfig {
    fn default() -> Self {
        HardwareConfig {
            vlen: 4096,      // 默认向量寄存器长度（位）
            lane_number: 4,  // 默认向量处理通道数量
        }
    }
}

impl Default for VectorConfig {
    fn default() -> Self {
        VectorConfig {
            software: SoftwareConfig::default(),
            hardware: HardwareConfig::default(),
        }
    }
}

impl Default for VectorRegisterPorts {
    fn default() -> Self {
        VectorRegisterPorts {
            read_ports_limit: 2,
            write_ports_limit: 1,
        }
    }
}

impl Default for VectorRegister {
    fn default() -> Self {
        VectorRegister {
            ports: VectorRegisterPorts::default(),
        }
    }
}

impl Default for BufferConfig {
    fn default() -> Self {
        BufferConfig {
            input_maximum_size: 64,  // 默认缓冲区大小为64字节
            result_maximum_size: 64, // 默认缓冲区大小为64字节
        }
    }
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        SimulatorConfig {
            function_units: FunctionUnits::default(),
            memory_units: MemoryUnits::default(),
            vector_config: VectorConfig::default(),
            vector_register: VectorRegister::default(),
            buffer: BufferConfig::default(),  // 添加buffer默认配置
            register: RegisterConfig::default(),  // 添加register默认配置
        }
    }
}

impl SimulatorConfig {
    /// 从指定路径加载 TOML 配置文件并初始化全局配置
    pub fn init_global_config(path: &str) -> anyhow::Result<()>{
        let contents = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        
        // 验证向量配置是否有效
        if !config.vector_config.is_valid() {
            return anyhow::bail!(format!("Invalid vector configuration: vl * sew > vlen ({}*{} > {})", 
                config.vector_config.software.vl, 
                config.vector_config.software.sew, 
                config.vector_config.hardware.vlen));
        }
        
        let mut global_config = CONFIG.write().unwrap();
        *global_config = Some(config);
        Ok(())
    }

    /// 获取全局配置的引用
    pub fn get_global_config() -> Option<SimulatorConfig> {
        CONFIG.read().unwrap().clone()
    }

    /// 从指定路径加载 TOML 配置文件
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        
        // 验证向量配置是否有效
        if !config.vector_config.is_valid() {
            return Err(format!("Invalid vector configuration: vl * sew > vlen ({}*{} > {})", 
                config.vector_config.software.vl, 
                config.vector_config.software.sew, 
                config.vector_config.hardware.vlen).into());
        }
        
        Ok(config)
    }
    
    /// 获取向量寄存器的总字节数
    pub fn get_vector_register_bytes(&self) -> u32 {
        self.vector_config.get_vector_register_bytes()
    }

    /// 获取向量寄存器正在使用的字节数
    pub fn get_vector_register_using_bytes(&self) -> u32 {
        self.vector_config.get_vector_register_using_bytes()
    }
    
    /// 获取向量寄存器的元素数量
    pub fn get_vector_elements_count(&self) -> u32 {
        self.vector_config.get_vector_elements_count()
    }
    
    /// 获取向量元素的字节数
    pub fn get_element_bytes(&self) -> u32 {
        self.vector_config.get_element_bytes()
    }
    
    /// 获取向量操作的总字节数
    pub fn get_total_vector_bytes(&self) -> u32 {
        self.vector_config.get_total_bytes()
    }
    
    /// 获取向量处理通道数量
    pub fn get_vector_lane_number(&self) -> u32 {
        self.vector_config.hardware.lane_number
    }
    
    pub fn get_maximum_forward_bytes(&self) -> u32 {
        self.register.maximum_forward_bytes
    }

    pub fn get_memory_read_ports_limit(&self) -> usize {
        self.memory_units.load_store_unit.read_ports_limit as usize
    }

    pub fn get_memory_write_ports_limit(&self) -> usize {
        self.memory_units.load_store_unit.write_ports_limit as usize
    }

    pub fn get_max_access_width(&self) -> u32 {
        self.memory_units.load_store_unit.max_access_width
    }

    pub fn get_data_length(&self) -> u32 {
        self.vector_config.hardware.lane_number * self.vector_config.software.sew / 8
    }
    
    // 新增：获取向量寄存器读端口限制
    pub fn get_vector_register_read_ports_limit(&self) -> u32 {
        self.vector_register.ports.read_ports_limit
    }
    
    // 新增：获取向量寄存器写端口限制
    pub fn get_vector_register_write_ports_limit(&self) -> u32 {
        self.vector_register.ports.write_ports_limit
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RegisterConfig {
    pub maximum_forward_bytes: u32,  // 最大前向字节数
}

impl Default for RegisterConfig {
    fn default() -> Self {
        RegisterConfig {
            maximum_forward_bytes: 32,  // 默认最大前向字节数为32字节
        }
    }
}