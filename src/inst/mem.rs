use crate::sim::{unit::memory_unit, register::{self, RegisterType}};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct MemInst {
    pub raw : riscv_isa::Instruction,
    pub dir : Direction,
    pub mem_addr : MemAddr,
    pub reg : RegisterType
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Direction {
    Read,
    Write
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct MemAddr {
    pub dependency : RegisterType
}

impl MemAddr {
    pub fn new(dependency : RegisterType) -> MemAddr {
        MemAddr {
            dependency
        }
    }
}

impl MemInst {
    pub fn new(riscv_isa : riscv_isa::Instruction) -> MemInst {
        // if you need more instructions, please extend this table
        let (dir, mem_addr, reg) = match riscv_isa {
            riscv_isa::Instruction::LD { rd, rs1, offset } => {
                (Direction::Read, MemAddr::new(RegisterType::ScalarRegister(rs1)), RegisterType::ScalarRegister(rd))
            },
            riscv_isa::Instruction::FLD { frd, rs1, offset } => {
                (Direction::Read, MemAddr::new(RegisterType::ScalarRegister(rs1)), RegisterType::FloatRegister(frd))
            },
            riscv_isa::Instruction::VLE { vrd, rs1, width } => {
                (Direction::Read, MemAddr::new(RegisterType::ScalarRegister(rs1)), RegisterType::VectorRegister(vrd))
            },
            riscv_isa::Instruction::VSE { vrd, rs1, width } => {
                (Direction::Write, MemAddr::new(RegisterType::ScalarRegister(rs1)), RegisterType::ScalarRegister(vrd))
            }
            _ => panic!("Not a memory instruction")
        };
        MemInst {
            raw : riscv_isa,
            dir,
            mem_addr,
            reg,
        }
    }

    pub fn get_total_bytes(&self) -> u32 {
        self.reg.get_bytes()
    }
}