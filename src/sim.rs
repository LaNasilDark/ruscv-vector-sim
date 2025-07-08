use std::{cell::RefCell, rc::Rc};
use std::collections::HashMap;
use fetch::Fetch;
use unit::function_unit::{FunctionUnit, FunctionUnitKeyType};
use unit::latency_calculator::calc_func_cycle; // 添加这一行
use log::debug;
use unit::memory_unit::LoadStoreUnit;

use crate::config::SimulatorConfig;
use crate::sim::register::RegisterTaskHandler;
use crate::inst::func::FuncInst;
use crate::inst::mem::{self, Direction, MemInst};
use crate::inst::Inst;
use crate::sim::register::RegisterFile;
use crate::sim::unit::buffer::{BufferEvent, BufferEventResult};
use crate::sim::unit::memory_unit::MemoryUnitKeyType;
use crate::sim::unit::UnitKeyType;
pub mod fetch;
pub mod unit;
pub mod register;

// 虽然
struct Simulator {
    fetch_unit : Fetch,
    function_unit : HashMap<FunctionUnitKeyType,FunctionUnit>,
    memory_unit : LoadStoreUnit,
    register_file : Rc<RefCell<RegisterFile>>
}

impl Simulator { 
    pub fn new() -> Simulator {
        let config = SimulatorConfig::get_global_config().expect("Global config not initialized");
        Simulator {
            fetch_unit : Fetch::new(),
            function_unit : todo!("make a hashmap with all function units"),
            memory_unit : LoadStoreUnit::new_from_config(&config.memory_units.load_store_unit),
            register_file : Rc::new(RefCell::new(RegisterFile::new()))
        }
    }

    fn handle_buffer_event(&mut self, key: UnitKeyType, event : BufferEvent) -> BufferEventResult {
        match key {
            UnitKeyType::FuncKey(func_key) => {
                let fu = self.function_unit.get_mut(&func_key).unwrap();
                
                fu.handle_buffer_event(event)
            },
            UnitKeyType::MemKey(mem_key) => {
                self.memory_unit.handle_buffer_event(mem_key, event)
                
            }
        }
    }

    fn handle_register_file(&mut self) {
        // self.register_file.iter_mut_tasks()
        // .for_each(|r| {
        //         if r.task_queue().len() == 0 {
        //             return;
        //         }
        //         r.init_current_index();

        //         while let Some(e) = r.generate_event() {
        //             let unit_key = r.get_current_task_unit_key();
        //             let result = self.handle_buffer_event(unit_key, e);
        //             r.handle_event_result(result);
        //         }

        //     }
        // );
        let r = self.register_file.clone();
        let mut register_file = r.borrow_mut();
        // let mut register_file = self.register_file.borrow_mut();
        register_file.iter_mut_tasks()
        .for_each(|r| {
            if r.task_queue().len() == 0 {
                return;
            }
            r.init_current_index();
            while let Some(e) = r.generate_event() {
                let unit_key = r.get_current_task_unit_key();
                let result = self.handle_buffer_event(unit_key, e);
                r.handle_event_result(result);
            }
        })
    }

    fn handle_unit_event_queue(&mut self) -> anyhow::Result<()>{
        self.function_unit.values_mut()
        .try_for_each(|v| {
            v.handle_event()
        })?;

        self.memory_unit.handle_event_queue()?;

        Ok(())
    }

    fn try_issue(&mut self) -> anyhow::Result<()> {
        if let Some(inst) = self.fetch_unit.fetch() {
            match inst {
                Inst::Func(func_inst) => {
                    let cycle = calc_func_cycle(&func_inst);
                    let fu = self.function_unit.get_mut(&func_inst.func_unit_key).unwrap();
                    if fu.is_empty() {
                        fu.issue(func_inst.clone())?;
                        self.fetch_unit.next_pc();
                        // TODO: Check if can issue the instruction on register file in if
                        self.register_file.borrow_mut().add_task(&func_inst);
                    }
                },
                Inst::Mem(mem_inst) => {
                    if self.memory_unit.has_free_port(mem_inst.dir) {
                        self.memory_unit.issue(mem_inst)?;
                    }
                }
            }
            
        }
        Ok(())
    }

    fn is_simulation_end(&self) -> bool {
        self.fetch_unit.is_empty() &&
        self.function_unit.values().all(|v| v.is_empty()) &&
        self.memory_unit.is_empty()
    }
    pub fn main_sim_loop(&mut self) -> anyhow::Result<()> { 
        
        let mut total_cycle : u32 = 0;
        while !self.is_simulation_end() { 
            
            debug!("START THE SIMULATION OF CYCLE {total_cycle}");
            
            // Step 1: Process the content of the result area of all Units with the content of RegisterFile
            // In this step, operate on the task queue of VectorRegisters from back to front
            self.handle_register_file();

            // Step 2: Update the event queues of all Units
            self.handle_unit_event_queue()?;
            
            // Step 3: Fetch new instructions and check if they can be issued
            self.try_issue()?;
            total_cycle += 1;
        }
        Ok(())
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