pub type RegisterIdType = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterType {
    ScalarRegister(RegisterIdType),
    VectorRegister(RegisterIdType),
    FloatRegister(RegisterIdType),
}