use std::collections::VecDeque;

use crate::inst::{func::FuncInst, MemoryPlace};

use crate::sim::register::RegisterType;
use crate::sim::unit::buffer::{BufferEvent, BufferEventResult, BufferPair};
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

    pub fn generate_next_event(&mut self) -> Option<Event> {
        if self.processed_bytes >= self.total_bytes {
            return None;
        }

        let remaining_bytes = self.total_bytes - self.processed_bytes;
        let bytes_this_event = std::cmp::min(self.bytes_per_event, remaining_bytes);
        
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
    buffer_pair : BufferPair
}

impl FunctionUnit {
    // 其他方法...
    
    // 添加这个方法来直接调用buffer_pair的handle_buffer_event方法
    pub fn handle_buffer_event(&mut self, event: BufferEvent) -> BufferEventResult {
        match self.buffer_pair.handle_buffer_event(event) {
            Ok(result) => result,
            Err(err) => panic!("Buffer event handling error: {}", err)
            // 或者您可以选择其他错误处理方式
        }
    }
}


