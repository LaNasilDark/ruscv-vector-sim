use std::fmt::format;

use riscv_isa::Instruction;

use crate::sim::{function_unit::FunctionUnitKeyType, register::RegisterType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncInst {
    raw : riscv_isa::Instruction,
    pub destination : RegisterType,
    pub resource : Vec<RegisterType>,
    pub func_unit_key : FunctionUnitKeyType
}

impl FuncInst {
    pub fn new(riscv_isa : riscv_isa::Instruction) -> FuncInst {
        // If you need more instructions, please extend this table

        let (destination, resource, func_unit_key) = match riscv_isa {
            Instruction::ADD { rd, rs1, rs2 } 
            | Instruction::ADDW { rd, rs1, rs2 }=> {
                (RegisterType::ScalarRegister(rd),
            vec![RegisterType::ScalarRegister(rs1), RegisterType::ScalarRegister(rs2)],
                FunctionUnitKeyType::IntegerAlu)
            },
            Instruction::XORI { rd, rs1, imm }
            |Instruction::ADDIW { rd, rs1, imm } => {
                (RegisterType::ScalarRegister(rd),
                vec![RegisterType::ScalarRegister(rs1)],
                FunctionUnitKeyType::IntegerAlu)
            },
            Instruction::SLLI { rd, rs1, shamt } => {
                (RegisterType::ScalarRegister(rd),
                vec![RegisterType::ScalarRegister(rs1)],
                FunctionUnitKeyType::IntegerAlu)
            },
            Instruction::VFADD_VV { vrd, vrs1, vrs2 } 
             => {
                (RegisterType::VectorRegister(vrd),
                vec![RegisterType::VectorRegister(vrs1), RegisterType::VectorRegister(vrs2)],
                FunctionUnitKeyType::VectorAlu)
            },
            Instruction::VFMUL_VV { vrd, vrs1, vrs2 }  => {
                (RegisterType::VectorRegister(vrd),
                vec![RegisterType::VectorRegister(vrs1), RegisterType::VectorRegister(vrs2)],
                FunctionUnitKeyType::VectorMul)
            }
            Instruction::VFSLIDE1DOWN_VF { vrd, frs1, vrs2 }
            | Instruction::VFSLIDE1UP_VF { vrd, frs1, vrs2 } => {
                (RegisterType::VectorRegister(vrd),
            vec![RegisterType::FloatRegister(frs1), RegisterType::VectorRegister(vrd), ], FunctionUnitKeyType::VectorSlide)
            }
            _ => unimplemented!("Not supported instruction")
        };
        FuncInst {
            raw : riscv_isa,
            destination,
            resource,
            func_unit_key
        }
    }

    pub fn get_key_type(&self) -> FunctionUnitKeyType {
        self.func_unit_key
    }

    pub fn is_float(&self) -> bool {
        let s = format!("{:?}", self.raw);
        s.contains("F")
    }
}