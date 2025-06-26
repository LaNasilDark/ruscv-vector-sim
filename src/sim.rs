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
use crate::inst::mem::{self, MemInst};
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
            function_unit : HashMap::new(),
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

    fn handle_unit_event_queue(&mut self) {

    }

    fn try_issue(&mut self) {
    }
    pub fn main_sim_loop(&mut self) { 
        
        let mut total_cycle : u32 = 0;
        while !self.fetch_unit.is_empty() { 
            
            debug!("START THE SIMULATION OF CYCLE {total_cycle}");
            
            // 第一步，用RegisterFile的内容处理所有Unit的结果区的内容
            // 在这一步中，从后往前对VectorRegister的队列进行操作，分别为
            // 如果是写：则从对应Unit处获得结果，且后面没有可能覆盖的读
            // 如果是读：后面没有可能覆盖的写，则向Unit的缓冲区写入结果
            self.handle_register_file();

            // 第二步：更新所有Unit的事件队列
            self.handle_unit_event_queue();

            // 第三步：获取新的指令，并查看是否可以issue
            self.try_issue();
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