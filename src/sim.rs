use std::{cell::RefCell, rc::Rc};
use std::collections::HashMap;
use fetch::Fetch;
use riscv_isa::Instruction;
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
        let bytes_per_event = 8; // 可以根据需要调整，或从配置中获取
        
        function_units.insert(FunctionUnitKeyType::VectorAlu, FunctionUnit::new(max_event_queue_size, bytes_per_event));
        function_units.insert(FunctionUnitKeyType::VectorMul, FunctionUnit::new(max_event_queue_size, bytes_per_event));
        function_units.insert(FunctionUnitKeyType::VectorDiv, FunctionUnit::new(max_event_queue_size, bytes_per_event));
        function_units.insert(FunctionUnitKeyType::VectorSlide, FunctionUnit::new(max_event_queue_size, bytes_per_event));
        function_units.insert(FunctionUnitKeyType::FloatAlu, FunctionUnit::new(max_event_queue_size, bytes_per_event));
        function_units.insert(FunctionUnitKeyType::FloatMul, FunctionUnit::new(max_event_queue_size, bytes_per_event));
        function_units.insert(FunctionUnitKeyType::FloatDiv, FunctionUnit::new(max_event_queue_size, bytes_per_event));
        function_units.insert(FunctionUnitKeyType::IntegerAlu, FunctionUnit::new(max_event_queue_size, bytes_per_event));
        function_units.insert(FunctionUnitKeyType::IntergerDiv, FunctionUnit::new(max_event_queue_size, bytes_per_event));
        
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
        debug!("处理缓冲区事件: {:?}, 事件: {:?}", key, event);
        let result = match key {
            UnitKeyType::FuncKey(func_key) => {
                let fu = self.function_unit.get_mut(&func_key).unwrap();
                fu.handle_buffer_event(event)
            },
            UnitKeyType::MemKey(mem_key) => {
                self.memory_unit.handle_buffer_event(mem_key, event)
            }
        };
        debug!("缓冲区事件处理结果: {:?}", result);
        result
    }

    fn handle_register_file(&mut self) {
        debug!("开始处理寄存器文件");
        let r = self.register_file.clone();
        let mut register_file = r.borrow_mut();
        register_file.iter_mut_tasks()
        .for_each(|r| {
            if r.task_queue().len() == 0 {
                return;
            }
            debug!("处理寄存器任务队列，长度: {}", r.task_queue().len());
            r.init_current_index();
            while let Some(e) = r.generate_event() {
                let unit_key = r.get_current_task_unit_key();
                debug!("生成寄存器事件: {:?}, 目标单元: {:?}", e, unit_key);
                let result = self.handle_buffer_event(unit_key, e);
                r.handle_event_result(result);
            }
        });
        debug!("寄存器文件处理完成");
    }

    fn handle_unit_event_queue(&mut self) -> anyhow::Result<()>{
        debug!("开始处理功能单元事件队列");
        for (key, unit) in self.function_unit.iter_mut() {
            debug!("处理功能单元 {:?} 的事件队列", key);
            unit.handle_event()?;
        }
    
        debug!("开始处理内存单元事件队列");
        self.memory_unit.handle_event_queue()?;
    
        Ok(())
    }

    fn try_issue(&mut self) -> anyhow::Result<()> {
        if let Some(inst) = self.fetch_unit.fetch() {
            debug!("尝试发射指令: {:?}", inst);
            match inst {
                Inst::Func(func_inst) => {
                    let cycle = calc_func_cycle(&func_inst);
                    debug!("功能指令周期数: {}", cycle);
                    let fu = self.function_unit.get_mut(&func_inst.func_unit_key).unwrap();
                    if fu.is_empty() {
                        debug!("功能单元 {:?} 空闲，发射指令", func_inst.func_unit_key);
                        fu.issue(func_inst.clone())?;
                        self.fetch_unit.next_pc();
                        // TODO: Check if can issue the instruction on register file in if
                        self.register_file.borrow_mut().add_task(&func_inst);
                        debug!("指令发射成功，PC前进");
                    } else {
                        debug!("功能单元 {:?} 忙，指令等待", func_inst.func_unit_key);
                    }
                },
                Inst::Mem(mem_inst) => {
                    if self.memory_unit.has_free_port(mem_inst.dir) {
                        debug!("内存单元有空闲端口，发射内存指令: {:?}", mem_inst);
                        self.memory_unit.issue(mem_inst)?;
                        debug!("内存指令发射成功");
                    } else {
                        debug!("内存单元没有空闲端口，内存指令等待");
                    }
                }
            }
        } else {
            debug!("没有更多指令可发射");
        }
        Ok(())
    }

    fn is_simulation_end(&self) -> bool {
        let fetch_empty = self.fetch_unit.is_empty();
        let function_units_empty = self.function_unit.values().all(|v| v.is_empty());
        let memory_unit_empty = self.memory_unit.is_empty();
        
        debug!("检查模拟是否结束: 取指单元空闲: {}, 功能单元空闲: {}, 内存单元空闲: {}", 
               fetch_empty, function_units_empty, memory_unit_empty);
        
        fetch_empty && function_units_empty && memory_unit_empty
    }


    // 添加一个新函数，用于自动增加内存数据
    fn auto_increase_memory_data(&mut self) -> anyhow::Result<()> {
        debug!("步骤 1.5: 自动增加内存数据");
        self.memory_unit.auto_increase_memory_data()
    }
    pub fn main_sim_loop(&mut self) -> anyhow::Result<()> { 
        let mut total_cycle : u32 = 0;
        debug!("开始主模拟循环");
        
        while !self.is_simulation_end() { 
            debug!("========== 开始周期 {} 的模拟 ==========", total_cycle);
            
            // Step 1.1: Process the content of the result area of all Units with the content of RegisterFile
            // In this step, operate on the task queue of VectorRegisters from back to front
            debug!("步骤 1.1: 处理所有单元的结果区域与寄存器文件的内容");
            self.handle_register_file();
    
            // Step 1.2: Auto increase memory data
            debug!("步骤 1.2: 增加input buffer里type为memory的内容");
            self.auto_increase_memory_data()?;
    
            // Step 2: Update the event queues of all Units
            debug!("步骤 2: 更新所有单元的事件队列");
            self.handle_unit_event_queue()?;
            
            // Step 3: Fetch new instructions and check if they can be issued
            debug!("步骤 3: 获取新指令并检查是否可以发射");
            self.try_issue()?;
            
            total_cycle += 1;
            debug!("========== 周期 {} 的模拟结束 ==========", total_cycle - 1);
        }
        debug!("主模拟循环结束，总周期数: {}", total_cycle);
        Ok(())
    }
}