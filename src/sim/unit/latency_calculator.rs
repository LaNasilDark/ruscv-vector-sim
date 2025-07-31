use crate::config::SimulatorConfig;
use crate::inst::func::FuncInst;
use crate::sim::unit::function_unit::FunctionUnitKeyType;

/// 计算功能单元执行指令所需的周期数
pub fn calc_func_cycle(inst: &FuncInst) -> u32 {
    let config = SimulatorConfig::get_global_config().expect("Global config not initialized");
    
    match inst.get_key_type() {
        FunctionUnitKeyType::IntegerAlu => {
            config.function_units.interger_alu.latency
        },
        FunctionUnitKeyType::IntergerDiv => {
            config.function_units.interger_divider.latency
        },
        FunctionUnitKeyType::FloatAlu => {
            config.function_units.float_alu.latency
        },
        FunctionUnitKeyType::FloatDiv => {
            config.function_units.float_divider.latency
        },
        FunctionUnitKeyType::FloatMul => {
            config.function_units.float_multiplier.latency
        },
        FunctionUnitKeyType::VectorAlu => {
            if inst.is_float() {
                config.function_units.float_alu.latency
            } else {
                config.function_units.interger_alu.latency
            }
        },
        FunctionUnitKeyType::VectorDiv => {
            if inst.is_float() {
                config.function_units.float_divider.latency
            } else {
                config.function_units.interger_divider.latency
            }
        },
        FunctionUnitKeyType::VectorMul => {
            if inst.is_float() {
                config.function_units.float_multiplier.latency
            } else {
                config.function_units.interger_multiplier.latency
            }
        },
        FunctionUnitKeyType::VectorMacc => {
            if inst.is_float() {
                config.function_units.float_multiplier.latency
            } else {
                config.function_units.interger_multiplier.latency
            }
        },
        FunctionUnitKeyType::VectorSlide => {
            1 // 向量滑动操作的固定延迟
        }
    }
}