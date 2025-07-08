use std::collections::{HashMap, VecDeque};





pub type RegisterIdType = u32;



pub mod task;

use task::RegisterTask;

use crate::{config::SimulatorConfig, inst::func::FuncInst, sim::unit::{buffer::{BufferEvent, BufferEventResult}, UnitBehavior, UnitKeyType}};
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
                config.get_vector_register_bytes()
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

pub trait RegisterTaskHandler {
    fn task_queue(&self) -> &VecDeque<RegisterTask>;
    fn task_queue_mut(&mut self) -> &mut VecDeque<RegisterTask>;
    fn get_total_bytes(&self) -> u32;
    fn init_current_index(&mut self);

    fn current_handle_index(&self) -> usize;
    fn update_handle_index(&mut self, index : usize);
    fn get_current_task_unit_key(&self) -> UnitKeyType {
        self.task_queue()[self.current_handle_index()].unit_key.clone()
    }
    fn handle_one_task(&self, index : usize) -> Option<BufferEvent>{
        let q = self.task_queue();
        let forward_bytes = SimulatorConfig::get_global_config().unwrap().get_maximum_forward_bytes().min(self.get_total_bytes() - q[index].current_place);
        match index == q.len() - 1 {
            true => {
                Some(q[index].generate_event(forward_bytes))
            },
            false => {
                let mut update_length = q[index+1].current_place - q[index].current_place;
                update_length = update_length.min(forward_bytes);
                if update_length == 0 {
                    None
                } else {
                    Some(q[index].generate_event(update_length))
                }
            }
        }
    }

    fn generate_event(&mut self) -> Option<BufferEvent> {
        let mut index = self.current_handle_index();
        while index > 0 {
            index -= 1;
            if let Some(event) = self.handle_one_task(index) {
                self.update_handle_index(index);
                return Some(event);
            }
        }
        None
    }

    fn handle_event_result(&mut self, result : BufferEventResult) {
        let index = self.current_handle_index();
        let total_bytes = self.get_total_bytes();
        let q = self.task_queue_mut();
        q[index].handle_result(result);
        if index == q.len() - 1 && q[index].current_place == total_bytes {
            q.pop_back();
        }
    }


}


impl RegisterTaskHandler for VectorRegister {
    fn init_current_index(&mut self) {
        self.current_index = self.task_queue.len() - 1;
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
}

impl RegisterTaskHandler for CommonRegister {
    fn init_current_index(&mut self) {
        self.current_index = self.task_queue.len() - 1;
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
        let vector_register_bytes = config.get_vector_register_bytes();
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
        func_inst.resource.iter().enumerate().for_each(|(i,r)| {
            match r {
                RegisterType::ScalarRegister(id) => {
                    self.scalar_registers[*id as usize].task_queue_mut().push_back(RegisterTask::new(i, UnitBehavior::Read , unit_key.clone()));
                },
                RegisterType::VectorRegister(id) => {
                    self.vector_registers[*id as usize].task_queue_mut().push_back(RegisterTask::new(i, UnitBehavior::Read , unit_key.clone()));
                },
                RegisterType::FloatRegister(id) => {
                    self.float_registers[*id as usize].task_queue_mut().push_back(RegisterTask::new(i, UnitBehavior::Read , unit_key.clone()));
                }
            }
        });
    }
}

