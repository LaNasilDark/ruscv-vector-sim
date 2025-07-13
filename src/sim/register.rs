use std::collections::{HashMap, VecDeque};





pub type RegisterIdType = u32;



pub mod task;

use task::RegisterTask;
use log::debug; // 添加log模块导入

use crate::{config::SimulatorConfig, inst::func::FuncInst, inst::mem::{MemInst, Direction}, sim::unit::{buffer::{BufferEvent, BufferEventResult}, UnitBehavior, UnitKeyType}};
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
    pub task_queue : VecDeque<RegisterTask>,
    pub current_index : usize
}

pub trait RegisterTaskHandler: 'static {
    fn task_queue(&self) -> &VecDeque<RegisterTask>;
    fn task_queue_mut(&mut self) -> &mut VecDeque<RegisterTask>;
    fn get_total_bytes(&self) -> u32;
    fn init_current_index(&mut self);
    fn current_handle_index(&self) -> usize;
    fn update_handle_index(&mut self, index : usize);
    fn get_register_type_info(&self) -> String; // 新增方法，用于获取寄存器类型信息
    
    fn get_current_task_unit_key(&self) -> UnitKeyType {
        self.task_queue()[self.current_handle_index()].unit_key.clone()
    }
    
    fn handle_one_task(&self, index : usize) -> Option<BufferEvent>{
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

    fn handle_event_result(&mut self, result : BufferEventResult) {
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
            
        if index == q.len() - 1 && q[index].current_place == total_bytes {
            debug!("Task completed for {} Task [{}], removing from queue", reg_type_info, index);
            q.pop_back();
        }
    }

    fn generate_event(&mut self) -> Option<BufferEvent> {
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

// 为 VectorRegister 实现新增的方法
impl RegisterTaskHandler for VectorRegister {
    fn init_current_index(&mut self) {
        self.current_index = self.task_queue.len();
    }

    fn current_handle_index(&self) -> usize {
        self.current_index
    }

    fn task_queue(&self) -> &VecDeque<RegisterTask> {
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
}

// 为 CommonRegister 实现新增的方法
impl RegisterTaskHandler for CommonRegister {
    fn init_current_index(&mut self) {
        self.current_index = self.task_queue.len();
    }
    fn current_handle_index(&self) -> usize {
        self.current_index
    }
    fn task_queue(&self) -> &VecDeque<RegisterTask> {
        &self.task_queue
    }
    fn task_queue_mut(&mut self) -> &mut VecDeque<RegisterTask> {
        &mut self.task_queue
    }
    fn get_total_bytes(&self) -> u32 {
        8
    }
    fn update_handle_index(&mut self, index : usize) {
        self.current_index = index;
    }
    
    fn get_register_type_info(&self) -> String {
        if self.get_total_bytes() == 8 {
            if self.id < 32 {
                format!("ScalarRegister({})", self.id)
            } else {
                format!("FloatRegister({})", self.id)
            }
        } else {
            format!("CommonRegister({})", self.id)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterFile {
    pub scalar_registers : Vec<CommonRegister>,
    pub vector_registers : Vec<VectorRegister>,
    pub float_registers : Vec<CommonRegister>,
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
                task_queue: VecDeque::new(),
                current_index: 0
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
                task_queue: VecDeque::new(),
                current_index: 0
            });
        }
        
        RegisterFile {
            scalar_registers,
            vector_registers,
            float_registers
        }
    }

    pub fn iter_mut_tasks(&mut self) -> impl Iterator<Item = &mut dyn RegisterTaskHandler> {
        // 将三种寄存器切片转换为 trait 对象切片并连接起来
        let scalar_iter = self.scalar_registers.iter_mut().map(|r| r as &mut dyn RegisterTaskHandler);
        let vector_iter = self.vector_registers.iter_mut().map(|r| r as &mut dyn RegisterTaskHandler);
        let float_iter = self.float_registers.iter_mut().map(|r| r as &mut dyn RegisterTaskHandler);
        
        // 使用 chain 方法将三个迭代器连接成一个
        scalar_iter.chain(vector_iter).chain(float_iter)
    }
    
    pub fn add_task(&mut self, func_inst : &FuncInst) {
        let unit_key = UnitKeyType::FuncKey(func_inst.func_unit_key);
        debug!("Adding task: {:?}, target unit: {:?}", func_inst.raw, unit_key);
        func_inst.resource.iter().enumerate().for_each(|(i,r)| {
            match r {
                RegisterType::ScalarRegister(id) => {
                    debug!("Adding scalar register task: register ID: {}, resource index: {}", id, i);
                    self.scalar_registers[*id as usize].task_queue_mut().push_front(RegisterTask::new(i, UnitBehavior::Read , unit_key.clone()));
                },
                RegisterType::VectorRegister(id) => {
                    debug!("Adding vector register task: register ID: {}, resource index: {}", id, i);
                    self.vector_registers[*id as usize].task_queue_mut().push_front(RegisterTask::new(i, UnitBehavior::Read , unit_key.clone()));
                },
                RegisterType::FloatRegister(id) => {
                    debug!("Adding float register task: register ID: {}, resource index: {}", id, i);
                    self.float_registers[*id as usize].task_queue_mut().push_front(RegisterTask::new(i, UnitBehavior::Read , unit_key.clone()));
                }
            }
        });
        match &func_inst.destination {

            RegisterType::ScalarRegister(id) => {
                debug!("Adding scalar register task: register ID: {}, resource index: {}", id, 0);
                self.scalar_registers[*id as usize].task_queue_mut().push_front(RegisterTask::new(0, UnitBehavior::Write , unit_key.clone()));
            },
            RegisterType::VectorRegister(id) => {
                debug!("Adding vector register task: register ID: {}, resource index: {}", id, 0);
                self.vector_registers[*id as usize].task_queue_mut().push_front(RegisterTask::new(0, UnitBehavior::Write , unit_key.clone()));
            },
            RegisterType::FloatRegister(id) => {
                debug!("Adding float register task: register ID: {}, resource index: {}", id, 0);
                self.float_registers[*id as usize].task_queue_mut().push_front(RegisterTask::new(0, UnitBehavior::Write , unit_key.clone()));
            }

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
        
        // 处理内存地址依赖的寄存器（通常是基址寄存器）
        match mem_inst.mem_addr.dependency {
            RegisterType::ScalarRegister(id) => {
                debug!("Adding scalar register task for memory address: register ID: {}", id);
                self.scalar_registers[id as usize].task_queue_mut().push_back(RegisterTask::new(0, UnitBehavior::Read, unit_key.clone()));
            },
            RegisterType::VectorRegister(id) => {
                debug!("Adding vector register task for memory address: register ID: {}", id);
                self.vector_registers[id as usize].task_queue_mut().push_back(RegisterTask::new(0, UnitBehavior::Read, unit_key.clone()));
            },
            RegisterType::FloatRegister(id) => {
                debug!("Adding float register task for memory address: register ID: {}", id);
                self.float_registers[id as usize].task_queue_mut().push_back(RegisterTask::new(0, UnitBehavior::Read, unit_key.clone()));
            }
        };
        
        // 处理数据寄存器，根据指令方向决定行为
        let (behavior, resource_index) = match mem_inst.dir {
            Direction::Read => (UnitBehavior::Write, 0),  // 读内存写寄存器
            Direction::Write => (UnitBehavior::Read, 1), // 读寄存器写内存
        };
        
        match mem_inst.reg {
            RegisterType::ScalarRegister(id) => {
                debug!("Adding scalar register task for data: register ID: {}, behavior: {:?}", id, behavior);
                self.scalar_registers[id as usize].task_queue_mut().push_back(RegisterTask::new(resource_index, behavior, unit_key.clone()));
            },
            RegisterType::VectorRegister(id) => {
                debug!("Adding vector register task for data: register ID: {}, behavior: {:?}", id, behavior);
                self.vector_registers[id as usize].task_queue_mut().push_back(RegisterTask::new(resource_index, behavior, unit_key.clone()));
            },
            RegisterType::FloatRegister(id) => {
                debug!("Adding float register task for data: register ID: {}, behavior: {:?}", id, behavior);
                self.float_registers[id as usize].task_queue_mut().push_back(RegisterTask::new(resource_index, behavior, unit_key.clone()));
            }
        };
    }
}

