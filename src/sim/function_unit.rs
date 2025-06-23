use crate::inst::{func::FuncInst, MemoryPlace};

use super::register::RegisterType;
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum FunctionUnitKeyType {
    VectorAlu,
    VectorMul,
    VectorDiv,
    VectorSlide,
    FloatAlu,
    FloatMul,
    FloatDiv,
    IntegerAlu,
    IntergerDiv
}
pub enum FunctionUnitType {
    VectorAdd,
    VectorMul,
    VectorDiv,
    VectorSlide,
    FloatAdd,
    FloatMul,
    FloatDiv,
    IntegerALU,

}

// pub struct FunctionUnit {
//     pub function_unit_type : FunctionUnitType,
//     pub name : String,
//     pub latency : usize,
//     pub throughput : usize,
//     pub issue_width : usize,
//     // 这里缺少FunctionUnit需要利用的资源定义
// }

pub trait FunctionUnit {
    fn is_occupied(&self) -> bool;

    fn issue(&mut self, inst : FuncInst, cycle : u32);
}

