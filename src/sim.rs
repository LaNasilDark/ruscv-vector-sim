use std::{cell::RefCell, rc::Rc};
use std::collections::HashMap;
use fetch::Fetch;
use function_unit::{FunctionUnit, FunctionUnitKeyType};
use log::debug;
use memory_unit::LoadStoreUnit;

use crate::config::SimulatorConfig;
use crate::inst::func::FuncInst;
use crate::inst::mem::{self, MemInst};
use crate::inst::Inst;
pub mod fetch;
pub mod execute;
pub mod vector_config;
pub mod function_unit;
pub mod register;
pub mod memory_unit;

// 虽然
struct Simulator {
    fetch_unit : Fetch,
    function_unit : HashMap<FunctionUnitKeyType,Box<dyn FunctionUnit>>,
    memory_unit : LoadStoreUnit,
    config : SimulatorConfig

}

impl Simulator { 
    pub fn new(config : SimulatorConfig) -> Simulator {
        Simulator {
            fetch_unit : Fetch::new(),
            function_unit : HashMap::new(),
            memory_unit : LoadStoreUnit::new_from_config(&config.memory_units.load_store_unit),
            config
        }
    }

    fn calc_func_cycle(config : &SimulatorConfig, inst: &FuncInst) -> u32 {
        match inst.get_key_type() {
            FunctionUnitKeyType::IntegerAlu => {
                config.function_units.interger_alu.latency
            },
            FunctionUnitKeyType::IntergerDiv => {
                config.function_units.interger_divider.latency
            },
            FunctionUnitKeyType::FloatAlu => {
                config.function_units.float_alu.latency
            },
            FunctionUnitKeyType::FloatDiv => {
                config.function_units.float_divider.latency
            },
            FunctionUnitKeyType::FloatMul => {
                config.function_units.float_multiplier.latency
            },
            FunctionUnitKeyType::VectorAlu => {
                if inst.is_float() {
                    config.function_units.float_alu.latency
                } else {
                    config.function_units.interger_alu.latency
                }
            },
            FunctionUnitKeyType::VectorDiv => {
                if inst.is_float() {
                    config.function_units.float_divider.latency
                } else {
                    config.function_units.interger_divider.latency
                }

            },
            FunctionUnitKeyType::VectorMul => {
                if inst.is_float() {
                    config.function_units.float_multiplier.latency
                } else {
                    config.function_units.interger_multiplier.latency
                }
            },
            FunctionUnitKeyType::VectorSlide => {
                1
            }

        }
    }

    pub fn main_sim_loop(&mut self) { 
        
        let mut total_cycle : u32 = 0;
        while !self.fetch_unit.is_empty() { 
            
            debug!("START THE SIMULATION OF CYCLE {total_cycle}");
            
            let current_instruction = self.fetch_unit.fetch();
            debug!("current instruction: {:?}", current_instruction);
            let inst = current_instruction.clone().unwrap();
            match inst {
                Inst::Mem(mem_inst) => {

                },
                Inst::Func(func_inst) => {
                    let func_unit = self.function_unit.get_mut(&func_inst.get_key_type()).unwrap();
                    match func_unit.is_occupied() {
                        true => {
                            // do nothing if the function unit is occupied
                        },
                        false => {

                            let cycle = Self::calc_func_cycle(&self.config, &func_inst);
                            func_unit.issue(func_inst, cycle);
                            self.fetch_unit.next_pc();
                        }

                    }
                }
            }
            total_cycle += 1;
        }
    
    }
}

// #[cfg(test)]
// mod tests {
//     use core::num;
//     use std::ops::Not;

//     use super::*;
//     use super::vector_config::{Configuration, VectorConfig, HardwareConfig};
//     use super::fetch::Fetch;
//     use crate::inst::{FuncInstruction, MemoryPlace, Resource, Destination};
//     use super::execute::Execute;
//     use log::{debug, info, warn, error};
//     use log::LevelFilter;
//     use std::io::Write;
//     use simplelog::*;
//     use std::fs::File;
//     fn init() {
//         CombinedLogger::init(
//             vec![
//                 TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
//                 WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("sim.log").unwrap()),
//             ]
//         ).unwrap();
//     }
//     #[test]
//     fn test_serial() {

//         init();

//         info!("This is a test for serial execution");

//         // 可能需要特判一下，或者Configuration用Result格式
//         // 因为vl和sew的设定是vl * sew <= vlen
//         let config = Configuration::new(
//             VectorConfig::new(64, 64, 1),
//             HardwareConfig::new(4096, 4)
//         );

//         let target_bytes = config.vector_config.total_length();
        
//         let inst_memory = vec![
//             FuncInstruction::new(
//                 Destination::new(MemoryPlace::VectorRegister(1), target_bytes),
//                 vec![Resource::new(MemoryPlace::Memory, target_bytes)],
//                 1,
//                 "vle64.v v1, (s1)"
//             ), // vle64.v	v1, (s1)
//             FuncInstruction::new(
//                 Destination::new(MemoryPlace::VectorRegister(2), target_bytes),
//                 vec![Resource::new(MemoryPlace::Memory, target_bytes)],
//                 1,
//                 "vle64.v v2, (s0)"
//              ), // vle64.v	v2, (s0)
//             FuncInstruction::new(
//                 Destination::new(MemoryPlace::VectorRegister(3), target_bytes),
//                 vec![Resource::new(MemoryPlace::VectorRegister(1), target_bytes),
//                  Resource::new(MemoryPlace::VectorRegister(2), target_bytes)],
//                  3,
//                 "vadd.vv v3, v1, v2"
//             ) // vadd.vv	v3, v1, v2
//         ];
//         let mut fetch = Fetch::new();
//         fetch.load(inst_memory);

//         // 这里的16和32 都是临时设置的
//         let mut execute = Execute::new(16, 32, config);
        
//         // 模拟运行的部分
//         let mut num_cycle : usize = 0;
        
//         while !fetch.is_empty() || !execute.is_empty() {
//             info!("Now simulate the cycle {}", num_cycle);
            

//             execute.execute_serial();

//             let inst = fetch.fetch();
//             match inst {
//                 Some(inst) => {
//                     info!("Fetch instruction: {:?}", inst);
//                     match execute.push(inst) {
//                         Ok(_) => {
//                             info!("Push instruction to execute queue success");
//                             fetch.next_pc();
//                         },
//                         Err(s) => {
//                             info!("Push instruction to execute queue failed: {}", s);
//                         }
//                     }
//                 },
//                 None => {}
//             }
//             num_cycle += 1;
//         }

//         info!("The simulation is finished");
//         info!("The number of cycles is {num_cycle}");

//     }

//     #[test]
//     fn test_jaccobi() {

//     }


//     fn jacobi_instruction_stream() -> Vec<FuncInstruction> {
//         vec![
//             FuncInstruction::new(
//                 Destination::new(MemoryPlace::ScalarRegister(15), 8),
//                 vec![
//                     Resource::new(MemoryPlace::Memory, 8)
//                 ],
//                 1,
//                 "ld	a5, -8(s0)"
//             ),

//         ]
//     }
//     fn init_cpu_helper(vlen: usize, lane_number : usize) {
//         init();

//         // 可能需要特判一下，或者Configuration用Result格式
//         // 因为vl和sew的设定是vl * sew <= vlen

//         let sew = 32;
//         let config = Configuration::new(
//             VectorConfig::new(vlen / sew, sew, 1),
//             HardwareConfig::new(vlen, lane_number)
//         );

//         let target_bytes = config.vector_config.total_length();
        
//         let inst_memory = jacobi_instruction_stream();

//     }
    
//     #[test]
//     fn test_with_logging() {
//         init();

//         debug!("This is a debug log in test");
//         info!("This is an info log in test");
//         warn!("This is a warning log in test");
//         error!("This is an error log in test");

//         assert!(true);
//     }
// }