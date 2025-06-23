use std::{fmt, sync::{Arc, Mutex}};

use crate::sim::function_unit::FunctionUnitKeyType;
pub(crate) mod func;
pub(crate) mod mem;

use func::FuncInst;
use riscv_isa::Instruction;
use mem::MemInst;

#[derive(Debug,Clone, Copy, PartialEq, Eq)]
pub enum MemoryPlace {
    VectorRegister(usize),
    ScalarRegister(usize),
    FloatingPointRegister(usize),
    Memory
}
#[derive(Debug,Clone, Copy, PartialEq)]
pub struct Resource {
    pub source : MemoryPlace,
    pub target_bytes : usize
}


#[derive(Debug,Clone, PartialEq, Copy)]
pub struct Destination {
    pub target : MemoryPlace,
    pub target_bytes : usize,

}
#[derive(Debug,Clone, PartialEq, Eq)]
pub enum Inst {
    Func(FuncInst),
    Mem(MemInst)
}

impl Inst {
    pub fn new(riscv_isa : Instruction) -> Inst {
        // If you need more instructions, please extend this table
        match riscv_isa {
            Instruction::LD {..} | Instruction::FLD{..} | Instruction::VLE{..} | Instruction::VSE {..} => Inst::Mem(MemInst::new(riscv_isa)),
            _ => Inst::Func(FuncInst::new(riscv_isa))
        }
 
    }

}

pub struct MemInstruction {
    pub destination : Destination,
    pub resource : Vec<Resource>,
    pub operation_cycle : usize,
    pub raw : riscv_isa::Instruction
}

#[derive(Clone, PartialEq)]
pub struct FuncInstruction {
    pub destination : Destination,
    pub resource : Vec<Resource>,
    pub operation_cycle : usize,
    pub key_type : FunctionUnitKeyType,
    raw : riscv_isa::Instruction,
}

impl fmt::Debug for FuncInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.raw)
    }
}

impl Resource {
    pub fn new(source : MemoryPlace, target_bytes : usize) -> Resource {
        Resource {
            source,
            target_bytes
        }
    }
}

impl Destination {
    pub fn new(target : MemoryPlace, target_bytes : usize) -> Destination {
        Destination {
            target,
            target_bytes
        }
    }
}
impl FuncInstruction {
    fn key_type_helper(raw : riscv_isa::Instruction) -> FunctionUnitKeyType {
        // If you need more instruction type to recognize, please add this table
        match raw {
            riscv_isa::Instruction::VFADD_VV {..}
            => FunctionUnitKeyType::VectorAlu,
            riscv_isa::Instruction::VFMUL_VV {..}
            => FunctionUnitKeyType::VectorMul,
            riscv_isa::Instruction::VFSLIDE1UP_VF {..} | riscv_isa::Instruction::VFSLIDE1DOWN_VF{..}
            => FunctionUnitKeyType::VectorSlide,

            _ => FunctionUnitKeyType::IntegerAlu
        }
    }
    pub fn new(destination : Destination, resource : Vec<Resource>, operation_cycle : usize, raw : riscv_isa::Instruction) -> FuncInstruction {
        FuncInstruction {
            resource,
            destination,
            operation_cycle,
            raw,
            key_type : FuncInstruction::key_type_helper(raw),
        }
    }
}


impl FuncInstruction {
    pub fn get_key_type(&self) -> FunctionUnitKeyType {
        self.key_type
    }
}