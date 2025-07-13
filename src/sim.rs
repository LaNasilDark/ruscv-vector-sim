use std::{cell::RefCell, rc::Rc};
use std::collections::HashMap;
use fetch::Fetch;
use riscv_isa::Instruction;
use unit::function_unit::{FunctionUnit, FunctionUnitKeyType};
use unit::latency_calculator::calc_func_cycle; // 添加这一行
use log::{debug, info};
use unit::memory_unit::LoadStoreUnit;

use crate::config::SimulatorConfig;
use crate::inst::mem::{self, Direction, MemInst};
use crate::inst::Inst;
use crate::sim::register::RegisterFile;
use crate::sim::unit::buffer::{BufferEvent, BufferEventResult};
use crate::sim::unit::memory_unit::MemoryUnitKeyType;
use crate::sim::unit::UnitKeyType;
pub mod fetch;
pub mod unit;
pub mod register;


pub struct Simulator {
    fetch_unit : Fetch,
    function_unit : HashMap<FunctionUnitKeyType,FunctionUnit>,
    memory_unit : LoadStoreUnit,
    register_file : Rc<RefCell<RegisterFile>>
}

impl Simulator { 
    pub fn new() -> Simulator {
        let config = SimulatorConfig::get_global_config().expect("Global config not initialized");
        
        // 创建所有功能单元的HashMap
        let mut function_units = HashMap::new();
        
        // 为每种功能单元类型创建实例
        // 使用默认的事件队列大小和每事件字节数
        let max_event_queue_size = 10; // 可以根据需要调整
        let bytes_per_event = 8; 
        let vector_bytes_per_event = SimulatorConfig::get_global_config().unwrap().get_data_length();
        
        // 修改这里，为每个FunctionUnit传递对应的unit_type
        function_units.insert(FunctionUnitKeyType::VectorAlu, FunctionUnit::new(max_event_queue_size, vector_bytes_per_event, FunctionUnitKeyType::VectorAlu));
        function_units.insert(FunctionUnitKeyType::VectorMul, FunctionUnit::new(max_event_queue_size, vector_bytes_per_event, FunctionUnitKeyType::VectorMul));
        function_units.insert(FunctionUnitKeyType::VectorDiv, FunctionUnit::new(max_event_queue_size, vector_bytes_per_event, FunctionUnitKeyType::VectorDiv));
        function_units.insert(FunctionUnitKeyType::VectorSlide, FunctionUnit::new(max_event_queue_size, vector_bytes_per_event, FunctionUnitKeyType::VectorSlide));
        function_units.insert(FunctionUnitKeyType::FloatAlu, FunctionUnit::new(max_event_queue_size, bytes_per_event, FunctionUnitKeyType::FloatAlu));
        function_units.insert(FunctionUnitKeyType::FloatMul, FunctionUnit::new(max_event_queue_size, bytes_per_event, FunctionUnitKeyType::FloatMul));
        function_units.insert(FunctionUnitKeyType::FloatDiv, FunctionUnit::new(max_event_queue_size, bytes_per_event, FunctionUnitKeyType::FloatDiv));
        function_units.insert(FunctionUnitKeyType::IntegerAlu, FunctionUnit::new(max_event_queue_size, bytes_per_event, FunctionUnitKeyType::IntegerAlu));
        function_units.insert(FunctionUnitKeyType::IntergerDiv, FunctionUnit::new(max_event_queue_size, bytes_per_event, FunctionUnitKeyType::IntergerDiv));
        
        Simulator {
            fetch_unit: Fetch::new(),
            function_unit: function_units,
            memory_unit: LoadStoreUnit::new_from_config(&config.memory_units.load_store_unit),
            register_file: Rc::new(RefCell::new(RegisterFile::new()))
        }
    }

    pub fn load_instructions(&mut self, inst : Vec<Instruction> ) {
        self.fetch_unit.load(inst);
    }
    fn handle_buffer_event(&mut self, key: UnitKeyType, event : BufferEvent) -> BufferEventResult {
        debug!("Processing buffer event: {:?}, event: {:?}", key, event);
        let result = match key {
            UnitKeyType::FuncKey(func_key) => {
                let fu = self.function_unit.get_mut(&func_key).unwrap();
                fu.handle_buffer_event(event)
            },
            UnitKeyType::MemKey(mem_key) => {
                self.memory_unit.handle_buffer_event(mem_key, event)
            }
        };
        debug!("Buffer event processing result: {:?}", result);
        result
    }

    fn handle_register_file(&mut self) {
        debug!("Start processing register file");
        let r = self.register_file.clone();
        let mut register_file = r.borrow_mut();
        register_file.iter_mut_tasks()
        .for_each(|r| {
            if r.task_queue().len() == 0 {
                return;
            }
            debug!("Processing register task queue, length: {}", r.task_queue().len());
            r.init_current_index();
            while let Some(e) = r.generate_event() {
                let unit_key = r.get_current_task_unit_key();
                debug!("Generated register event: {:?}, target unit: {:?}", e, unit_key);
                let result = self.handle_buffer_event(unit_key, e);
                r.handle_event_result(result);
            }
        });
        debug!("Register file processing completed");
    }

    fn handle_unit_event_queue(&mut self) -> anyhow::Result<()>{
        debug!("Start processing function unit event queue");
        for (key, unit) in self.function_unit.iter_mut() {
            debug!("Processing function unit {:?} event queue", key);
            unit.handle_event()?;
        }
    
        debug!("Start processing memory unit event queue");
        self.memory_unit.handle_event_queue()?;
    
        Ok(())
    }

    fn try_issue(&mut self) -> anyhow::Result<()> {
        if let Some(inst) = self.fetch_unit.fetch() {
            debug!("Trying to issue instruction: {:?}", inst);
            match inst {
                Inst::Func(func_inst) => {
                    let cycle = calc_func_cycle(&func_inst);
                    debug!("Function instruction cycles: {}", cycle);
                    let fu = self.function_unit.get_mut(&func_inst.func_unit_key).unwrap();
                    // 使用新的判断函数
                    if fu.can_accept_new_instruction() {
                        debug!("Function unit {:?} can accept new instruction, issuing", func_inst.func_unit_key);
                        fu.issue(func_inst.clone())?;
                        self.fetch_unit.next_pc();
                        
                        self.register_file.borrow_mut().add_task(&func_inst);
                        debug!("Instruction issued successfully, PC advanced");
                    } else {
                        debug!("Function unit {:?} cannot accept new instruction yet, waiting", func_inst.func_unit_key);
                    }
                },
                Inst::Mem(mem_inst) => {
                    // 在try_issue函数中，处理Inst::Mem之前添加
                    self.memory_unit.debug_port_status();
                    // 使用新的判断函数
                    if self.memory_unit.can_accept_new_instruction(mem_inst.dir) {
                        debug!("Memory unit can accept new instruction, issuing memory instruction: {:?}", mem_inst);
                        
                        // issue 这条指令
                        let port_index = self.memory_unit.issue(mem_inst.clone())?;
                        
                        // 添加对寄存器文件的任务
                        self.register_file.borrow_mut().add_mem_task(&mem_inst, port_index);
                        // 更新PC值，这行是新增的
                        self.fetch_unit.next_pc();
                        // 处理完Inst::Mem后再次添加
                        self.memory_unit.debug_port_status();
                        debug!("Memory instruction issued successfully, PC advanced");
                    } else {
                        debug!("Memory unit cannot accept new instruction yet, memory instruction waiting");
                    }
                }
            }
        } else {
            debug!("No more instructions to issue");
        }
        Ok(())
    }

    fn is_simulation_end(&self) -> bool {
        let fetch_empty = self.fetch_unit.is_empty();
        let function_units_empty = self.function_unit.values().all(|v| v.is_empty());
        let memory_unit_empty = self.memory_unit.is_empty();
        
        debug!("Checking if simulation is complete: fetch unit idle: {}, function units idle: {}, memory unit idle: {}", 
               fetch_empty, function_units_empty, memory_unit_empty);
        
        fetch_empty && function_units_empty && memory_unit_empty
    }


    // 添加一个新函数，用于自动增加内存数据
    fn auto_increase_memory_data(&mut self) -> anyhow::Result<()> {
        self.memory_unit.auto_increase_memory_data()
    }
    pub fn main_sim_loop(&mut self) -> anyhow::Result<()> { 
        let mut total_cycle : u32 = 0;
        let max_cycles : u32 = 100; // 设置最大周期数为20
        debug!("Starting main simulation loop");
        
        while !self.is_simulation_end() && total_cycle < max_cycles { // 添加周期限制条件
            info!("========== Starting simulation for cycle {} ==========", total_cycle);
            
            // Step 1.1: Process the content of the result area of all Units with the content of RegisterFile
            // In this step, operate on the task queue of VectorRegisters from back to front
            info!("Step 1.1: Processing result areas of all units with register file content");
            self.handle_register_file();
    
            // Step 1.2: Auto increase memory data
            info!("Step 1.2: Increasing memory-type content in input buffer");
            self.auto_increase_memory_data()?;
    
            // Step 2: Update the event queues of all Units
            info!("Step 2: Updating event queues of all units");
            self.handle_unit_event_queue()?;
            
            // Step 3: Fetch new instructions and check if they can be issued
            info!("Step 3: Fetching new instructions and checking if they can be issued");
            self.try_issue()?;
            
            total_cycle += 1;
            info!("========== Simulation for cycle {} completed ==========", total_cycle - 1);
        }
        
        // 添加周期限制达到时的日志
        if total_cycle >= max_cycles {
            debug!("Reached maximum cycle limit ({}), simulation stopped", max_cycles);
        } else {
            info!("Main simulation loop ended, total cycles: {}", total_cycle);
        }
        
        Ok(())
    }
}
