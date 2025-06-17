pub type RegisterIdType = usize;
pub enum RegisterType {
    ScalarRegister(RegisterIdType),
    VectorRegister(RegisterIdType),
    FloatRegister(RegisterIdType),
}