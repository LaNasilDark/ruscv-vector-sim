use crate::sim::unit::function_unit::FunctionUnitKeyType;
use crate::sim::unit::memory_unit::MemoryUnitKeyType;
pub mod function_unit;
pub mod memory_unit;
pub mod latency_calculator;
pub mod buffer;
// 对于不同的Behavior有两种模式
// 如果是写，则去对应的Unit请求写的结果
// 如果是读，则主动把读送到对应的Unit的缓冲区
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitBehavior {
    Read,
    Write
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnitKeyType {
    FuncKey(FunctionUnitKeyType),
    MemKey(MemoryUnitKeyType)
}