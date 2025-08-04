use std::collections::{HashMap, VecDeque};





pub type RegisterIdType = u32;



pub mod task;

use task::RegisterTask;
use log::debug; // 添加log模块导入

use crate::{config::SimulatorConfig, inst::{func::FuncInst, mem::{Direction, MemInst}, Inst}, sim::unit::{buffer::{BufferEvent, BufferEventResult}, UnitBehavior, UnitKeyType}};
use crate::sim::MemoryUnitKeyType;
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum RegisterType {
    ScalarRegister(RegisterIdType),
    VectorRegister(RegisterIdType),
    FloatRegister(RegisterIdType),
}

impl RegisterType {
    /// 获取寄存器的字节数
    pub fn get_bytes(&self) -> u32 {
        match self {
            RegisterType::ScalarRegister(_) => 8, // 64位寄存器，8字节
            RegisterType::FloatRegister(_) => 8, // 64位浮点寄存器，8字节
            RegisterType::VectorRegister(_) => {
                // 从配置中获取向量寄存器的字节数
                let config = crate::config::SimulatorConfig::get_global_config()
                    .expect("Global config not initialized");
                config.get_vector_register_using_bytes()
            }
        }
    }
}


// 如果处理多读多写，可能会有一种状态是 2读 2写 1读（其中2读利用2写的数据，1读利用初始数据），这个序列还可以持续延长，这个时候对于数据的管理就变成了一个Register的状态是一个有序的FunctionUnit数组
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorRegister {
    pub id : RegisterIdType,
    pub total_bytes : u32,
    pub write_count : u32,
    pub read_count : u32,
    pub current_index : usize,
    pub task_queue : VecDeque<RegisterTask>
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonRegister { // 8 bytes Register
    pub id : RegisterIdType,
    pub write_instruction : Option<Inst>,
}


// 为 VectorRegister 实现新增的方法
impl VectorRegister {
    pub fn get_current_task_unit_key(&self) -> UnitKeyType {
        self.task_queue()[self.current_handle_index()].unit_key.clone()
    }
    pub fn init_current_index(&mut self) {
        self.current_index = self.task_queue.len();
    }

    fn current_handle_index(&self) -> usize {
        self.current_index
    }

    pub fn task_queue(&self) -> &VecDeque<RegisterTask> {
        &self.task_queue
    }

    fn task_queue_mut(&mut self) -> &mut VecDeque<RegisterTask> {
        &mut self.task_queue
    }

    fn get_total_bytes(&self) -> u32 {
        self.total_bytes
    }

    fn update_handle_index(&mut self, index : usize) {
        self.current_index = index;
    }
    
    fn get_register_type_info(&self) -> String {
        format!("VectorRegister({})", self.id)
    }

    
    pub fn handle_one_task(&self, index : usize) -> Option<BufferEvent>{
        let q = self.task_queue();
        let forward_bytes = SimulatorConfig::get_global_config().unwrap().get_maximum_forward_bytes().min(self.get_total_bytes() - q[index].current_place);
        
        // 使用新方法获取寄存器类型信息
        let reg_type_info = self.get_register_type_info();
        
        debug!("Processing {} Task [{}]: UnitKey: {:?}, Behavior: {:?}, Progress: {}/{} bytes, Resource Index: {}", 
            reg_type_info, index, q[index].unit_key, q[index].behavior, 
            q[index].current_place, self.get_total_bytes(), q[index].resource_index);
        
        match index == q.len() - 1 {
            true => {
                debug!("[FORWARD-INFO] ===== Register forwarding (last task) =====");
                debug!("[FORWARD-INFO] Register: {}, Task: {}", reg_type_info, index);
                debug!("[FORWARD-INFO] Forward bytes: {} bytes (max allowed: {})", 
                       forward_bytes, 
                       SimulatorConfig::get_global_config().unwrap().get_maximum_forward_bytes());
                debug!("[FORWARD-INFO] ==========================================");
                Some(q[index].generate_event(forward_bytes))
            },
            false => {
                // The task is not the last one, we need to calculate the maximum bytes we can forward
                let mut update_length = q[index+1].current_place - q[index].current_place;
                update_length = update_length.min(forward_bytes);
                debug!("[FORWARD-INFO] ===== Register forwarding (middle task) =====");
                debug!("[FORWARD-INFO] Register: {}, Task: {}", reg_type_info, index);
                debug!("[FORWARD-INFO] Update length: {} bytes, Next task position: {}", 
                       update_length, q[index+1].current_place);
                debug!("[FORWARD-INFO] Max forward bytes: {}", forward_bytes);
                debug!("[FORWARD-INFO] ==========================================");
                
                if update_length == 0 {
                    debug!("[FORWARD-INFO] Update length is 0, no event generated");
                    None
                } else {
                    debug!("[FORWARD-INFO] Generating event with update length: {} bytes", update_length);
                    Some(q[index].generate_event(update_length))
                }
            }
        }
    }

    pub fn handle_event_result(&mut self, result : BufferEventResult) {
        let index = self.current_handle_index();
        let total_bytes = self.get_total_bytes();
        let reg_type_info = self.get_register_type_info(); // 使用新方法获取寄存器类型信息
        let mut q = self.task_queue_mut();
        
        debug!("Processing event result for {} Task [{}]: UnitKey: {:?}, Behavior: {:?}, Current Progress: {}/{} bytes", 
            reg_type_info, index, q[index].unit_key, q[index].behavior, q[index].current_place, total_bytes);
        
        q[index].handle_result(result);
        
        debug!("Task status after processing: {} Task [{}]: Progress: {}/{} bytes ({:.2}%), UnitKey: {:?}, Behavior: {:?}", 
            reg_type_info, index, q[index].current_place, total_bytes, 
            (q[index].current_place as f32 / total_bytes as f32) * 100.0,
            q[index].unit_key, q[index].behavior);
        
        let mut need_decrease_read = false;
        let mut need_decrease_write = false;
        if index == q.len() - 1 && q[index].current_place == total_bytes {
            debug!("Task completed for {} Task [{}], removing from queue", reg_type_info, index);
            match q[index].behavior {
                UnitBehavior::Read => {
                    need_decrease_read = true;
                },
                UnitBehavior::Write => {
                    need_decrease_write = true;
                },
            }
            q.pop_back();
        }
        if need_decrease_read {
            // 减少读取计数：任务完成后释放读取资源
            debug!("Decreasing read count for completed {} read task", reg_type_info);
            self.decrease_read_count();
            debug!("Read count after decrease: {} for {}", self.get_read_count(), reg_type_info);
        }
        if need_decrease_write {
            // 减少写入计数：任务完成后释放写入资源
            debug!("Decreasing write count for completed {} write task", reg_type_info);
            self.decrease_write_count();
            debug!("Write count after decrease: {} for {}", self.get_write_count(), reg_type_info);
        }
    }

    pub fn generate_event(&mut self) -> Option<BufferEvent> {
        let mut index = self.current_handle_index();
        let reg_type_info = self.get_register_type_info(); // 使用新方法获取寄存器类型信息
        
        
            
        while index > 0 {
            index -= 1;
            debug!("Starting event generation for {}, current index: {}, queue length: {}", 
            reg_type_info, index, self.task_queue().len());
            debug!("Attempting to process {} Task [{}], UnitKey: {:?}, Behavior: {:?}, Progress: {}/{} bytes", 
                reg_type_info, index, 
                self.task_queue()[index].unit_key,
                self.task_queue()[index].behavior,
                self.task_queue()[index].current_place,
                self.get_total_bytes());
                
            if let Some(event) = self.handle_one_task(index) {
                debug!("Successfully generated event for {} Task [{}]: {:?}, updating current index to: {}", 
                    reg_type_info, index, event, index);
                self.update_handle_index(index);
                return Some(event);
            }
        }
        debug!("No events generated for {}", reg_type_info);
        None
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterFile {
    pub scalar_registers : Vec<CommonRegister>,
    pub vector_registers : Vec<VectorRegister>,
    pub float_registers : Vec<CommonRegister>,
}


impl CommonRegister {
    pub fn is_in_an_unfinished_write(&self) -> bool { 
        self.write_instruction.is_some()
    }
    pub(super) fn set_write_instruction(&mut self, inst : Inst) {
        self.write_instruction = Some(inst);
    }
}

impl VectorRegister {
    pub fn get_read_count(&self) -> u32 {
        self.read_count
    }

    pub fn get_write_count(&self) -> u32 {
        self.write_count
    }
    pub fn decrease_read_count(&mut self) {
        self.read_count -= 1
    }
    pub fn decrease_write_count(&mut self) {
        self.write_count -= 1
    }
    pub fn increase_read_count(&mut self) {
        self.read_count += 1
    }
    pub fn increase_write_count(&mut self) {
        self.write_count += 1
    }

}
impl RegisterFile {
    pub fn new() -> Self {
        let mut scalar_registers = Vec::with_capacity(32);
        let mut vector_registers = Vec::with_capacity(32);
        let mut float_registers = Vec::with_capacity(32);
        
        // 创建32个整数寄存器
        for id in 0..32 {
            scalar_registers.push(CommonRegister {
                id,
                write_instruction: None,
            });
        }
        
        // 创建32个向量寄存器
        let config = SimulatorConfig::get_global_config().expect("Global config not initialized");
        let vector_register_bytes = config.get_vector_register_using_bytes();
        for id in 0..32 {
            vector_registers.push(VectorRegister {
                id,
                total_bytes: vector_register_bytes,
                write_count: 0,
                read_count: 0,
                current_index: 0,
                task_queue: VecDeque::new()
            });
        }
        
        // 创建32个浮点寄存器
        for id in 0..32 {
            float_registers.push(CommonRegister {
                id,
                write_instruction: None,
            });
        }
        
        RegisterFile {
            scalar_registers,
            vector_registers,
            float_registers
        }
    }

    pub fn iter_mut_tasks(&mut self) -> impl Iterator<Item = &mut VectorRegister> {
        // 将三种寄存器切片转换为 trait 对象切片并连接起来
        // let scalar_iter = self.scalar_registers.iter_mut().map(|r| r as &mut dyn RegisterTaskHandler);
        let vector_iter = self.vector_registers.iter_mut();
        // let float_iter = self.float_registers.iter_mut().map(|r| r as &mut dyn RegisterTaskHandler);
        
        // 使用 chain 方法将三个迭代器连接成一个
        vector_iter

        // 本来是三种寄存器，现在删成一个了
    }
    pub fn add_common_task(&mut self, func_inst : &FuncInst) {
        let unit_key = UnitKeyType::FuncKey(func_inst.func_unit_key);
        debug!("Adding common task: {:?}, target unit: {:?}", func_inst.raw, unit_key);
        
        // 将目标寄存器标记为正在被写入
        match func_inst.destination {
            RegisterType::ScalarRegister(id) => {
                debug!("Marking scalar register {} as being written by instruction: {:?}", id, func_inst.raw);
                self.scalar_registers[id as usize].write_instruction = Some(crate::inst::Inst::Func(func_inst.clone()));
            },
            RegisterType::FloatRegister(id) => {
                debug!("Marking float register {} as being written by instruction: {:?}", id, func_inst.raw);
                self.float_registers[id as usize].write_instruction = Some(crate::inst::Inst::Func(func_inst.clone()));
            },
            RegisterType::VectorRegister(_) => {
                unreachable!("Common Function Unit cannot write to Vector Registers")
            }
        }
    }
    pub fn add_vector_task(&mut self, func_inst : &FuncInst) {
        let unit_key = UnitKeyType::FuncKey(func_inst.func_unit_key);
        debug!("Adding vector task: {:?}, target unit: {:?}", func_inst.raw, unit_key);
        
        // 为源寄存器添加读任务并更新读计数
        func_inst.resource.iter().enumerate().for_each(|(i,r)| {
            match r {
                RegisterType::VectorRegister(id) => {
                    let old_count = self.vector_registers[*id as usize].get_read_count();
                    self.vector_registers[*id as usize].task_queue_mut().push_front(RegisterTask::new(i, UnitBehavior::Read , unit_key.clone()));
                    self.vector_registers[*id as usize].increase_read_count();
                    let new_count = self.vector_registers[*id as usize].get_read_count();
                    debug!("[READ_COUNT_DEBUG] Vector register {} read count: {} -> {} (added read task)", id, old_count, new_count);
                },
                _ => {}
            }
        });
        
        // 为目标寄存器添加写任务并更新写计数
        match &func_inst.destination {
            RegisterType::ScalarRegister(id) => {
                debug!("Marking scalar register {} as being written by instruction: {:?}", id, func_inst.raw);
                self.scalar_registers[*id as usize].set_write_instruction(Inst::Func(func_inst.clone()));
            },
            RegisterType::FloatRegister(id) => {
                debug!("Marking float register {} as being written by instruction: {:?}", id, func_inst.raw);
                self.float_registers[*id as usize].set_write_instruction(Inst::Func(func_inst.clone()));
            },
            RegisterType::VectorRegister(id) => {
                let old_count = self.vector_registers[*id as usize].get_write_count();
                self.vector_registers[*id as usize].task_queue_mut().push_front(RegisterTask::new(0, UnitBehavior::Write , unit_key.clone()));
                self.vector_registers[*id as usize].increase_write_count();
                let new_count = self.vector_registers[*id as usize].get_write_count();
                debug!("[WRITE_COUNT_DEBUG] Vector register {} write count: {} -> {} (added write task)", id, old_count, new_count);
            },
        }
    }
    
    // 为MemInst添加任务
    pub fn add_mem_task(&mut self, mem_inst: &MemInst, port_index: usize) {
        // 根据指令方向创建不同的UnitKey
        let unit_key = match mem_inst.dir {
            Direction::Read => UnitKeyType::MemKey(MemoryUnitKeyType::Load(port_index)),
            Direction::Write => UnitKeyType::MemKey(MemoryUnitKeyType::Store(port_index)),
        };
        
        debug!("Adding memory task: {:?}, target unit: {:?}", mem_inst.raw, unit_key);
        
        
        // 处理数据寄存器，根据指令方向决定行为
        let (behavior, resource_index) = match mem_inst.dir {
            Direction::Read => (UnitBehavior::Write, 0),  // 读内存写寄存器
            Direction::Write => (UnitBehavior::Read, 0), // 读寄存器写内存
        };
        
        match mem_inst.reg {
            RegisterType::VectorRegister(id) => {
                debug!("Adding vector register task for data: register ID: {}, behavior: {:?}", id, behavior);
                self.vector_registers[id as usize].task_queue_mut().push_front(RegisterTask::new(resource_index, behavior.clone(), unit_key.clone()));
                match behavior {
                    UnitBehavior::Read => {
                        let old_count = self.vector_registers[id as usize].get_read_count();
                        self.vector_registers[id as usize].increase_read_count();
                        let new_count = self.vector_registers[id as usize].get_read_count();
                        debug!("[READ_COUNT_DEBUG] Vector register {} read count: {} -> {} (added read task)", id, old_count, new_count);
                    },
                    UnitBehavior::Write => {
                        let old_count = self.vector_registers[id as usize].get_write_count();
                        self.vector_registers[id as usize].increase_write_count();
                        let new_count = self.vector_registers[id as usize].get_write_count();
                        debug!("[WRITE_COUNT_DEBUG] Vector register {} write count: {} -> {} (added write task)", id, old_count, new_count);        
                    },
                }
            },
            RegisterType::FloatRegister(id) if mem_inst.dir == Direction::Read => {
                self.float_registers[id as usize].set_write_instruction(Inst::Mem(*mem_inst));
            },
            RegisterType::ScalarRegister(id) if mem_inst.dir == Direction::Read => {
                self.scalar_registers[id as usize].set_write_instruction(Inst::Mem(*mem_inst));
            },
            _ => {
                // do nothing here
            }
        };
    }

    
    /// 检查寄存器是否有未完成的写操作
    pub fn has_unfinished_writes(&self, reg: &RegisterType) -> bool {
        let result = match reg {
            RegisterType::ScalarRegister(id) => {
                let register = &self.scalar_registers[*id as usize];
                let has_write = register.is_in_an_unfinished_write();
                if has_write {
                    if let Some(ref write_inst) = register.write_instruction {
                        debug!("Register {:?} has unfinished write from instruction: {:?}", reg, write_inst);
                    } else {
                        unreachable!("Register {:?} has unfinished write but no instruction info", reg);
                    }
                }
                has_write
            },
            
            RegisterType::FloatRegister(id) => {
                let register = &self.float_registers[*id as usize];
                let has_write = register.is_in_an_unfinished_write();
                if has_write {
                    if let Some(ref write_inst) = register.write_instruction {
                        debug!("Register {:?} has unfinished write from instruction: {:?}", reg, write_inst);
                    } else {
                        debug!("Register {:?} has unfinished write but no instruction info", reg);
                    }
                }
                has_write
            },
            _ => unreachable!("Only check non-vector register"),
        };
        
        result
    }
    
    /// 检查common指令的所有操作数是否可以issue（没有未完成的写）
    pub fn can_issue_common_instruction(&self, func_inst: &FuncInst) -> bool {
        // 检查所有源操作数是否有未完成的写
        for operand in &func_inst.resource {
            if self.has_unfinished_writes(operand) {
                debug!("Cannot issue instruction {:?}: source operand {:?} has unfinished writes", func_inst.raw, operand);
                return false;
            }
        }

        // 同时以暂停issue的方式禁止WRW，毕竟WRW是很少的情况，可以这么做
        if self.has_unfinished_writes(&func_inst.destination) {
            return false;
        }
        true
    }
    
    /// 检查vector指令是否可以issue
    /// 如果源操作数包含common寄存器，必须等待其写完成
    pub fn can_issue_vector_instruction(&self, func_inst: &FuncInst) -> bool {
        // 获取配置中的端口限制
        let config = crate::config::SimulatorConfig::get_global_config()
            .expect("Global config not initialized");
        let read_ports_limit = config.get_vector_register_read_ports_limit();
        let write_ports_limit = config.get_vector_register_write_ports_limit();
        
        debug!("[ISSUE_CHECK_DEBUG] Checking vector instruction: {:?}", func_inst.raw);
        debug!("[ISSUE_CHECK_DEBUG] Port limits - read: {}, write: {}", read_ports_limit, write_ports_limit);
        
        // 检查源操作数
        for operand in &func_inst.resource {
            match operand {
                RegisterType::ScalarRegister(_) | RegisterType::FloatRegister(_) => {
                    if self.has_unfinished_writes(operand) {
                        debug!("[ISSUE_CHECK_DEBUG] Cannot issue: common register {:?} has unfinished writes", operand);
                        return false;
                    }
                },
                RegisterType::VectorRegister(id) => {
                    let current_read_count = self.vector_registers[*id as usize].get_read_count();
                    debug!("[ISSUE_CHECK_DEBUG] Vector register {} current read count: {}, limit: {}", id, current_read_count, read_ports_limit);
                    if current_read_count + 1 > read_ports_limit {
                        debug!("[ISSUE_CHECK_DEBUG] Cannot issue: vector register {} read count would exceed limit ({} + 1 > {})", 
                               id, current_read_count, read_ports_limit);
                        return false;
                    }
                }
            }
        }
        
        // 检查目标寄存器
        match &func_inst.destination {
            RegisterType::FloatRegister(_) | RegisterType::ScalarRegister(_) => {
                if self.has_unfinished_writes(&func_inst.destination) {
                    debug!("[ISSUE_CHECK_DEBUG] Cannot issue: destination register {:?} has unfinished writes", func_inst.destination);
                    return false;
                }
            },
            RegisterType::VectorRegister(id) => {
                let current_write_count = self.vector_registers[*id as usize].get_write_count();
                debug!("[ISSUE_CHECK_DEBUG] Vector register {} current write count: {}, limit: {}", id, current_write_count, write_ports_limit);
                if current_write_count + 1 > write_ports_limit {
                    debug!("[ISSUE_CHECK_DEBUG] Cannot issue: vector register {} write count would exceed limit ({} + 1 > {})", 
                           id, current_write_count, write_ports_limit);
                    return false;
                }
            }
        }
        
        debug!("[ISSUE_CHECK_DEBUG] Vector instruction can be issued");
        true
    }
    pub(crate) fn can_issue_memory_instruction(&self, mem_inst: &MemInst) -> bool {
        // 检查地址依赖的寄存器是否没有未完成的写
        if self.has_unfinished_writes(&mem_inst.mem_addr.dependency) {
            return false;
        }

        // 如果是读操作 （也就是说会写寄存器）
        if mem_inst.dir == Direction::Read {
            match mem_inst.reg {
                RegisterType::ScalarRegister(_) | RegisterType::FloatRegister(_) => {
                    if self.has_unfinished_writes(&mem_inst.reg) {
                        return false;
                    }
                },
                RegisterType::VectorRegister(_) => {
                    // TODO: 可以检查也可以不检查，在这里限制读写任务的个数
                }
            }
        }

        true
    }
    pub(crate) fn clean_write(&mut self, reg : &RegisterType) {
        match reg {
            RegisterType::FloatRegister(id) => {
                self.float_registers[*id as usize].write_instruction = None;    
            },
            RegisterType::ScalarRegister(id) => {
                self.scalar_registers[*id as usize].write_instruction = None;
            },
            _ => unreachable!("Only clean non-vector register"),
        }
    }
}

