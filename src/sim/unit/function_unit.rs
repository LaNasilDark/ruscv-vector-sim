use std::collections::VecDeque;

use crate::inst::{func::FuncInst, MemoryPlace};

use crate::sim::register::RegisterType;
use crate::sim::unit::buffer::{BufferEvent, BufferEventResult, BufferPair, ResourceType};
use crate::sim::unit::latency_calculator::calc_func_cycle;
use crate::config::SimulatorConfig;
use log::debug; // 添加log模块导入
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
        debug!("[EventGenerator] Generating next event: processed_bytes={}/{} bytes, current_input_bytes={} bytes", 
               self.processed_bytes, self.total_bytes, current_bytes);
        
        if self.processed_bytes >= self.total_bytes {
            debug!("[EventGenerator] All bytes processed, no more events to generate");
            return None;
        }
    
        // Add detailed debug info for bytes_this_event calculation
        debug!("[BYTES-CALC] ===== Calculating bytes_this_event =====");
        debug!("[BYTES-CALC] bytes_per_event: {} bytes", self.bytes_per_event);
        debug!("[BYTES-CALC] total_bytes - processed_bytes: {} - {} = {} bytes", 
               self.total_bytes, self.processed_bytes, self.total_bytes - self.processed_bytes);
        debug!("[BYTES-CALC] current_bytes - processed_bytes: {} - {} = {} bytes", 
               current_bytes, self.processed_bytes, current_bytes - self.processed_bytes);
        
        let bytes_this_event = self.bytes_per_event.min(self.total_bytes - self.processed_bytes).min(current_bytes - self.processed_bytes);
        
        debug!("[BYTES-CALC] Final bytes_this_event: {} bytes", bytes_this_event);
        debug!("[BYTES-CALC] ==========================================");
    
        if bytes_this_event == 0 {
            debug!("[EventGenerator] No bytes available for this event, waiting for more input data");
            return None;
        }
        
        let event = Event {
            remained_cycle: self.cycle_per_event,
            target_register: self.func_inst.destination.clone(),
            result_bytes: bytes_this_event,
        };
    
        self.processed_bytes += bytes_this_event;
        
        // 添加更明显的转发信息格式
        debug!("[FORWARD-INFO] ===== EventGenerator created new event =====");
        debug!("[FORWARD-INFO] Unit type: {:?}", self.func_inst.get_key_type());
        debug!("[FORWARD-INFO] Target register: {:?}", event.target_register);
        debug!("[FORWARD-INFO] Forward bytes: {} bytes (max allowed: {})", 
               bytes_this_event, 
               SimulatorConfig::get_global_config().expect("Global config not initialized").get_maximum_forward_bytes());
        debug!("[FORWARD-INFO] Total progress: {}/{} bytes ({:.2}%)", 
               self.processed_bytes, self.total_bytes, 
               (self.processed_bytes as f32 / self.total_bytes as f32) * 100.0);
        debug!("[FORWARD-INFO] ==========================================");
    
        Some(event)
    }

    pub fn is_complete(&self) -> bool {
        let completed = self.processed_bytes >= self.total_bytes;
        debug!("[EventGenerator] Checking if complete: processed_bytes={}/{} bytes ({:.2}%), result={}", 
               self.processed_bytes, self.total_bytes, 
               (self.processed_bytes as f32 / self.total_bytes as f32) * 100.0, 
               completed);
        completed
    }
}




pub struct FunctionUnit {
    occupied : bool,
    max_event_queue_size : usize,
    event_queue : VecDeque<Event>,
    current_event : Option<EventGenerator>,
    bytes_per_event : u32,
    buffer_pair : BufferPair,
    unit_type : FunctionUnitKeyType
}

impl FunctionUnit {
    pub fn new(max_event_queue_size: usize, bytes_per_event: u32, unit_type: FunctionUnitKeyType) -> Self {
        use crate::sim::unit::buffer::BufferOwnerType;
        
        FunctionUnit {
            occupied: false,
            max_event_queue_size,
            event_queue: VecDeque::new(),
            current_event: None,
            bytes_per_event,
            buffer_pair: BufferPair::new_with_owner(BufferOwnerType::FunctionUnit(unit_type)),
            unit_type
        }
    }

    
    // 添加这个方法来直接调用buffer_pair的handle_buffer_event方法
    pub fn handle_buffer_event(&mut self, event: BufferEvent) -> BufferEventResult {
        match self.buffer_pair.handle_buffer_event(event) {
            Ok(result) => result,
            Err(err) => panic!("Buffer event handling error: {}", err)
            // 或者您可以选择其他错误处理方式
        }
    }


    fn free_unit(&mut self) {
        debug!("[{:?}] ResultBuffer check passed, clearing occupied flag and current event", self.unit_type);
        self.occupied = false;
        self.current_event = None;
        
        // 使用 BufferPair 的 clear 方法清空所有缓冲区
        self.buffer_pair.clear();
        
        debug!("[{:?}] Unit freed successfully", self.unit_type);
    }

    pub(crate) fn is_empty(&self) -> bool{
        !self.occupied
    }
    pub fn handle_event(&mut self) -> anyhow::Result<()>{
        debug!("[{:?}] Starting handle_event: event_queue_size={}, occupied={}", 
               self.unit_type, self.event_queue.len(), self.occupied);
        
        // 更新所有事件的剩余周期
        self.event_queue.iter_mut().for_each(
            |v| {
                debug!("[{:?}] Decreasing event cycle: remained_cycle={} -> {}, target_register={:?}, result_bytes={} bytes", 
                       self.unit_type, v.remained_cycle, v.remained_cycle - 1, v.target_register, v.result_bytes);
                v.remained_cycle -= 1;
            }
        );
    
        // 清除队列最后的事件
        let mut completed_events = 0;
        while let Some(event) = self.event_queue.back() {
            if event.remained_cycle == 0 {
                debug!("[{:?}] Event completed: target_register={:?}, result_bytes={} bytes", 
                       self.unit_type, event.target_register, event.result_bytes);
                self.buffer_pair.increase_result(event.result_bytes)?;
                self.event_queue.pop_back();
                completed_events += 1;
            } else {
                break;
            }
        }
        debug!("[{:?}] Completed and removed {} events from queue", self.unit_type, completed_events);
    
        // 保证每次只加入一个事件
        if let Some(event_gen) = self.current_event.as_mut() {
            debug!("[{:?}] Processing current event generator", self.unit_type);
            if event_gen.is_complete() {
                debug!("[{:?}] EventGenerator is end", self.unit_type);
                if let Some(ref destination) = self.buffer_pair.result_buffer.destination {
                    if destination.is_completed() {
                        debug!("[{:?}] ResultBuffer is fully consumed, freeing unit", self.unit_type);
                        self.free_unit();
                        
                    } else {
                        debug!("[{:?}] Cannot free unit: ResultBuffer not fully consumed yet. Current: {}/{} bytes, Consumed: {} bytes", 
                       self.unit_type, destination.current_size, destination.target_size, destination.consumed_bytes);
                    }
                }
                
                
            } else {
                debug!("[{:?}] Event generator not complete, checking for new events", self.unit_type);
                let current_bytes = self.buffer_pair.get_current_input_bytes()?;
                debug!("[{:?}] Current input bytes available: {} bytes", self.unit_type, current_bytes);
                
                if let Some(event) = event_gen.generate_next_event(current_bytes) {
                    debug!("[{:?}] Adding new event to queue: remained_cycle={}, target_register={:?}, result_bytes={} bytes", 
                           self.unit_type, event.remained_cycle, event.target_register, event.result_bytes);
                    self.event_queue.push_front(event);
                } else {
                    debug!("[{:?}] No new event generated, waiting for more input data", self.unit_type);
                }
            }
        } else {
            debug!("[{:?}] No current event generator active", self.unit_type);
        }
        
        debug!("[{:?}] Finished handle_event: event_queue_size={}, occupied={}", 
               self.unit_type, self.event_queue.len(), self.occupied);
        Ok(())
    }

    fn set_occupied(&mut self) {
        assert!(self.occupied == false);
        self.occupied = true;
    }
    pub fn issue(&mut self, func_inst : FuncInst) -> anyhow::Result<()> {
        self.set_occupied();
        
        // 添加详细的调试信息
        let total_bytes = func_inst.total_process_bytes();
        let is_vector = func_inst.resource.iter().any(|v| matches!(v, RegisterType::VectorRegister(_)));
        
        debug!("[{:?}] Issuing instruction: {:?}", self.unit_type, func_inst.raw);
        debug!("[{:?}] Is vector instruction: {}", self.unit_type, is_vector);
        if is_vector {
            let config = SimulatorConfig::get_global_config().expect("Global config not initialized");
            debug!("[{:?}] Vector config - vlen: {} bits ({} bytes)", 
                   self.unit_type, config.vector_config.hardware.vlen, 
                   config.vector_config.hardware.vlen / 8);
            debug!("[{:?}] Vector config - sew: {} bits ({} bytes)", 
                   self.unit_type, config.vector_config.software.sew, 
                   config.vector_config.software.sew / 8);
            debug!("[{:?}] Vector config - vl: {}", self.unit_type, config.vector_config.software.vl);
            debug!("[{:?}] Calculated total_bytes: {} (should be: {})", 
                   self.unit_type, total_bytes, 
                   (config.vector_config.software.sew / 8) * config.vector_config.software.vl);
        }
        
        self.current_event = Some(EventGenerator::new(func_inst.clone(), calc_func_cycle(&func_inst), self.bytes_per_event, total_bytes));
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

    pub fn can_accept_new_instruction(&self) -> bool {
        // 首先检查单元是否被占用
        if self.occupied {
            return false;
        }
        
        // 检查事件队列是否为空
        if !self.event_queue.is_empty() {
            return false;
        }
        
        // 检查当前是否有活跃的事件生成器
        if self.current_event.is_some() {
            return false;
        }
        
        // 检查ResultBuffer是否为空
        if let Some(ref destination) = self.buffer_pair.result_buffer.destination {
            if destination.current_size > 0 {
                return false;
            }
        }
        
        // 所有条件都满足，可以接受新指令
        true
    }
}


