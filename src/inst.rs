use std::{fmt, sync::{Arc, Mutex}};

use crate::sim::unit::function_unit::FunctionUnitKeyType;
pub(crate) mod func;
pub(crate) mod mem;

use crate::config::SimulatorConfig;
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
