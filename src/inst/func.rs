use std::fmt::format;

use riscv_isa::Instruction;

use crate::sim::{unit::function_unit::FunctionUnitKeyType, register::RegisterType};
use crate::config::SimulatorConfig;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncInst {
    pub raw : riscv_isa::Instruction,
    pub destination : RegisterType,
    pub resource : Vec<RegisterType>,
    pub func_unit_key : FunctionUnitKeyType
}

impl FuncInst {
    pub fn new(riscv_isa : riscv_isa::Instruction) -> FuncInst {
        // If you need more instructions, please extend this table

        let (destination, resource, func_unit_key) = match riscv_isa {
            Instruction::ADD { rd, rs1, rs2 } 
            | Instruction::ADDW { rd, rs1, rs2 } 
            | Instruction::SUB { rd, rs1, rs2 }=> {
                (RegisterType::ScalarRegister(rd),
            vec![RegisterType::ScalarRegister(rs1), RegisterType::ScalarRegister(rs2)],
                FunctionUnitKeyType::IntegerAlu)
            },
            Instruction::XORI { rd, rs1, imm }
            |Instruction::ADDIW { rd, rs1, imm } 
            | Instruction::ADDI { rd, rs1, imm }=> {
                (RegisterType::ScalarRegister(rd),
                vec![RegisterType::ScalarRegister(rs1)],
                FunctionUnitKeyType::IntegerAlu)
            },
            Instruction::SLLI { rd, rs1, shamt } | Instruction::SRLI { rd, rs1, shamt } => {
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
            Instruction::VMACC_VV { vrd, vrs1, vrs2 } => {
                (RegisterType::VectorRegister(vrd),
                vec![RegisterType::VectorRegister(vrd), RegisterType::VectorRegister(vrs1), RegisterType::VectorRegister(vrs2)],
                FunctionUnitKeyType::VectorMacc)
            },
            Instruction::VFSLIDE1DOWN_VF { vrd, frs1, vrs2 }
            | Instruction::VFSLIDE1UP_VF { vrd, frs1, vrs2 } => {
                (RegisterType::VectorRegister(vrd),
            vec![RegisterType::FloatRegister(frs1), RegisterType::VectorRegister(vrd), RegisterType::VectorRegister(vrs2)], FunctionUnitKeyType::VectorSlide)
            },
            _ => unimplemented!("Not supported instruction {:?}", riscv_isa),
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
    // 需要处理的长度，如果是scalar，这个值是8，否则是vlen / 8
    pub fn total_process_bytes(&self) -> u32 {
        if self.resource.iter().any(|v| matches!(v, RegisterType::VectorRegister(_))) {
            // 从全局获得vlen的信息
            let config = SimulatorConfig::get_global_config().expect("Global config not initialized");
            config.get_vector_register_using_bytes()
        } else {
            8
        }
    }
}