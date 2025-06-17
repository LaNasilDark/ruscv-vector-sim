use crate::inst::MemoryPlace;

use super::register::RegisterType;

pub enum FunctionUnitType {
    VectorAdd,
    VectorMul,
    VectorDiv,
    VectorSlide,
    FloatAdd,
    FloatMul,
    FloatDiv,
    IntegerALU,
    MemoryLoad(RegisterType),
    MemoryStore(RegisterType),
}

pub struct FunctionUnit {
    pub function_unit_type : FunctionUnitType,
    pub name : String,
    pub latency : usize,
    pub throughput : usize,
    pub issue_width : usize,
    // 这里缺少FunctionUnit需要利用的资源定义
}

