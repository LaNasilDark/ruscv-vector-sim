use std::collections::VecDeque;

use crate::inst::{func::FuncInst, MemoryPlace};

use crate::sim::register::RegisterType;
use crate::sim::unit::buffer::{BufferEvent, BufferEventResult, BufferPair, ResourceType};
use crate::sim::unit::latency_calculator::calc_func_cycle;
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum FunctionUnitKeyType {
    VectorAlu,
    VectorMul,
    VectorDiv,
    VectorSlide,
    FloatAlu,
    FloatMul,
    FloatDiv,
    IntegerAlu,
    IntergerDiv
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub remained_cycle : u32,
    pub target_register : RegisterType,
    pub result_bytes : u32
}

// Generate Event Every Cycle
pub struct EventGenerator {
    func_inst: FuncInst,
    cycle_per_event: u32,
    bytes_per_event: u32,
    total_bytes: u32,
    processed_bytes: u32,
}

impl EventGenerator {
    pub fn new(func_inst: FuncInst, cycle_per_event: u32, bytes_per_event: u32, total_bytes: u32) -> Self {
        
        EventGenerator {
            func_inst,
            cycle_per_event,
            bytes_per_event,
            total_bytes,
            processed_bytes: 0,
        }
    }

    pub fn generate_next_event(&mut self, current_bytes: u32) -> Option<Event> {
        if self.processed_bytes >= self.total_bytes {
            return None;
        }

        let bytes_this_event = self.bytes_per_event.min(self.total_bytes - self.processed_bytes).min(current_bytes - self.processed_bytes);

        if bytes_this_event == 0 {
            return None;
        }
        
        let event = Event {
            remained_cycle: self.cycle_per_event,
            target_register: self.func_inst.destination.clone(),
            result_bytes: bytes_this_event,
        };

        self.processed_bytes += bytes_this_event;

        Some(event)
    }

    pub fn is_complete(&self) -> bool {
        self.processed_bytes >= self.total_bytes
    }
}




pub struct FunctionUnit {
    occupied : bool,
    max_event_queue_size : usize,
    event_queue : VecDeque<Event>,
    current_event : Option<EventGenerator>,
    bytes_per_event : u32,
    buffer_pair : BufferPair
}

impl FunctionUnit {
    pub fn new(max_event_queue_size: usize, bytes_per_event: u32) -> Self {
        FunctionUnit {
            occupied: false,
            max_event_queue_size,
            event_queue: VecDeque::new(),
            current_event: None,
            bytes_per_event,
            buffer_pair: BufferPair::new()
        }
    }
    
    // 其他方法...
    
    // 添加这个方法来直接调用buffer_pair的handle_buffer_event方法
    pub fn handle_buffer_event(&mut self, event: BufferEvent) -> BufferEventResult {
        match self.buffer_pair.handle_buffer_event(event) {
            Ok(result) => result,
            Err(err) => panic!("Buffer event handling error: {}", err)
            // 或者您可以选择其他错误处理方式
        }
    }


    fn free_unit(&mut self) {
        self.occupied = false;
        self.current_event = None;
        // 清除当前指令信息
        self.buffer_pair.current_instruction = None;
    }

    pub(crate) fn is_empty(&self) -> bool{
        !self.occupied
    }
    pub fn handle_event(&mut self) -> anyhow::Result<()>{
        self.event_queue.iter_mut().for_each(
            |v| {
                v.remained_cycle -= 1;
            }
        );

        // 清除队列最后的事件
        while let Some(event) = self.event_queue.back() {
            if event.remained_cycle == 0 {
                self.buffer_pair.increase_result(event.result_bytes)?;
                self.event_queue.pop_back();
            } else {
                break;
            }
        }

        // 保证每次只加入一个事件
        if let Some(event_gen) = self.current_event.as_mut() {
            if event_gen.is_complete() {
                self.free_unit();
            } else {
                let current_bytes = self.buffer_pair.get_memory_input_current_bytes()?;
                if let Some(event) = event_gen.generate_next_event(current_bytes) {
                    self.event_queue.push_back(event);
                }
            }
        }
        Ok(())
    }

    fn set_occupied(&mut self) {
        assert!(self.occupied == false);
        self.occupied = true;
    }
    pub fn issue(&mut self, func_inst : FuncInst) -> anyhow::Result<()> {
        self.set_occupied();
        self.current_event = Some(EventGenerator::new(func_inst.clone(), calc_func_cycle(&func_inst), self.bytes_per_event, func_inst.total_process_bytes()));
        use crate::sim::unit::buffer::EnhancedResource;
        use crate::sim::unit::buffer::Resource;
        self.buffer_pair.set_input(func_inst.resource.iter().map(|r| Resource{
            resource_type: ResourceType::Register(r.clone()), 
            current_size: 0, 
            target_size: r.get_bytes()
        }).collect::<Vec<_>>())?;
    
        self.buffer_pair.set_output(EnhancedResource{ 
            resource_type: ResourceType::Register(func_inst.destination.clone()), 
            current_size: 0, 
            target_size: func_inst.destination.get_bytes(),
            consumed_bytes: 0
        });
        
        // 记录当前正在处理的指令信息
        self.buffer_pair.set_current_instruction(crate::inst::Inst::Func(func_inst.clone()));
        
        Ok(())
    }
}


