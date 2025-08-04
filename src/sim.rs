use std::{cell::RefCell, rc::Rc};
use std::collections::HashMap;
use fetch::Fetch;
use riscv_isa::Instruction;
use unit::function_unit::{FunctionUnitType, VectorFunctionUnit, CommonFunctionUnit, FunctionUnitKeyType};
use unit::latency_calculator::calc_func_cycle; // 添加这一行
use log::{debug, info};
use unit::memory_unit::LoadStoreUnit;
use crate::sim::unit::function_unit::CommonEventResult;
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
    function_unit : HashMap<FunctionUnitKeyType, FunctionUnitType>,
    memory_unit : LoadStoreUnit,
    register_file : Rc<RefCell<RegisterFile>>
}

impl Simulator { 
    pub fn new() -> Simulator {
        let config = SimulatorConfig::get_global_config().expect("Global config not initialized");
        
        let mut function_units = HashMap::new();
        
        let bytes_per_event = 8; 
        let vector_bytes_per_event = SimulatorConfig::get_global_config().unwrap().get_data_length();
        
        // Vector类型使用VectorFunctionUnit
        function_units.insert(FunctionUnitKeyType::VectorAlu, FunctionUnitType::Vector(VectorFunctionUnit::new(vector_bytes_per_event, FunctionUnitKeyType::VectorAlu)));
        function_units.insert(FunctionUnitKeyType::VectorMul, FunctionUnitType::Vector(VectorFunctionUnit::new(vector_bytes_per_event, FunctionUnitKeyType::VectorMul)));
        function_units.insert(FunctionUnitKeyType::VectorMacc, FunctionUnitType::Vector(VectorFunctionUnit::new(vector_bytes_per_event, FunctionUnitKeyType::VectorMacc)));
        function_units.insert(FunctionUnitKeyType::VectorDiv, FunctionUnitType::Vector(VectorFunctionUnit::new(vector_bytes_per_event, FunctionUnitKeyType::VectorDiv)));
        function_units.insert(FunctionUnitKeyType::VectorSlide, FunctionUnitType::Vector(VectorFunctionUnit::new(vector_bytes_per_event, FunctionUnitKeyType::VectorSlide)));
        
        // 普通类型使用CommonFunctionUnit
        function_units.insert(FunctionUnitKeyType::FloatAlu, FunctionUnitType::Common(CommonFunctionUnit::new(FunctionUnitKeyType::FloatAlu)));
        function_units.insert(FunctionUnitKeyType::FloatMul, FunctionUnitType::Common(CommonFunctionUnit::new(FunctionUnitKeyType::FloatMul)));
        function_units.insert(FunctionUnitKeyType::FloatDiv, FunctionUnitType::Common(CommonFunctionUnit::new( FunctionUnitKeyType::FloatDiv)));
        function_units.insert(FunctionUnitKeyType::IntegerAlu, FunctionUnitType::Common(CommonFunctionUnit::new(FunctionUnitKeyType::IntegerAlu)));
        function_units.insert(FunctionUnitKeyType::IntergerDiv, FunctionUnitType::Common(CommonFunctionUnit::new( FunctionUnitKeyType::IntergerDiv)));
        
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
                match fu {
                    FunctionUnitType::Vector(fu) => {
                        fu.handle_buffer_event(event)
                    },
                    _ => unreachable!("Common Function Unit does not have buffer")
                }
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
            match unit {
                FunctionUnitType::Common(fu) => {
                    let res = fu.handle_event()?;
                    match res {
                        CommonEventResult::Nothing => {
                            debug!("[{:?}] Function unit {:?} has no task end", key, key);
                        },
                        CommonEventResult::WriteResultTo(reg) => {
                            debug!("[{:?}] Function unit {:?} write result to register {:?}", key, key, reg);
                            self.register_file.borrow_mut().clean_write(&reg);
                        }
                    }
                },
                FunctionUnitType::Vector(fu) => {
                    fu.handle_event()?;
                }
            }
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
                    match fu {
                        FunctionUnitType::Common(fu) => {
                            // 检查功能单元是否可以接受新指令
                            let can_issue_register = self.register_file.borrow().can_issue_common_instruction(&func_inst);
                            
                            debug!("Issue check for {:?}: register_file_ready={}", 
                                   func_inst.raw, can_issue_register);
                            
                            if can_issue_register {
                                fu.issue(func_inst.clone(), self.fetch_unit.get_pc())?;
                                self.fetch_unit.next_pc();
                                self.register_file.borrow_mut().add_common_task(&func_inst);
                                debug!("Instruction issued successfully, PC advanced");
                            } else {
                                if !can_issue_register {
                                    debug!("Function unit {:?} cannot accept new instruction: register file not ready", func_inst.func_unit_key);
                                }
                            }
                        },
                        FunctionUnitType::Vector(fu) => {
                            // 检查功能单元是否可以接受新指令
                            if self.register_file.borrow().can_issue_vector_instruction(&func_inst) && fu.can_accept_new_instruction() {
                                fu.issue(func_inst.clone(), self.fetch_unit.get_pc())?;
                                self.fetch_unit.next_pc();
                                self.register_file.borrow_mut().add_vector_task(&func_inst);
                                debug!("Instruction issued successfully, PC advanced");
                            } else {
                                debug!("Function unit {:?} cannot accept new instruction yet, waiting", func_inst.func_unit_key);
                            }
                        }
                    }
                },
                Inst::Mem(mem_inst) => {
                    // 在try_issue函数中，处理Inst::Mem之前添加
                    self.memory_unit.debug_port_status();
                    // 使用新的判断函数
                    if self.memory_unit.can_accept_new_instruction(mem_inst.dir) && self.register_file.borrow().can_issue_memory_instruction(&mem_inst) {
                        debug!("Memory unit can accept new instruction, issuing memory instruction: {:?}", mem_inst);
                        
                        // issue 这条指令
                        let port_index = self.memory_unit.issue(mem_inst.clone(), self.fetch_unit.get_pc())?;
                        
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
    // 添加一个新函数，用于自动增加内存写入的已消耗字节数
    fn auto_increase_memory_write_consumed_bytes(&mut self) -> anyhow::Result<()> {
        self.memory_unit.auto_increase_memory_write_consumed_bytes()
    }

    // 完成memory port需要的写
    fn finish_read_port_writing_to_register(&mut self) -> anyhow::Result<()> {
        let res = self.memory_unit.clean_read_port_result_buffer()?;
        for reg in res {
            self.register_file.borrow_mut().clean_write(&reg);
        }
        Ok(())
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
            
            // Step 1.3: Auto increase memory write consumed bytes
            info!("Step 1.3: Increasing consumed bytes for memory-type in write port result buffer");
            self.auto_increase_memory_write_consumed_bytes()?;
            
            info!("Step 1.4: Clean the read port information(if it is a scalar register.");
            self.finish_read_port_writing_to_register()?;
            
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
