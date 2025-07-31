use std::any;

use crate::sim::register::RegisterType;
use crate::config::SimulatorConfig;
use crate::sim::unit::{FunctionUnitKeyType, MemoryUnitKeyType};
use log::{debug, info}; // 添加log模块导入


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceType {
    Register(RegisterType),
    Memory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    pub resource_type : ResourceType,
    pub target_size : u32,
    pub current_size : u32
}

impl Resource {
    pub fn new(resource_type: ResourceType, target_size: u32) -> Self {
        Resource {
            resource_type,
            target_size,
            current_size: 0
        }
    }
    
    pub fn is_full(&self) -> bool {
        self.current_size >= self.target_size
    }
    
    pub fn remaining_capacity(&self) -> u32 {
        if self.target_size > self.current_size {
            self.target_size - self.current_size
        } else {
            0
        }
    }
    
    // 添加数据到资源
    pub fn append_data(&mut self, append_length: u32) -> u32 {
        let available_space = self.remaining_capacity();
        let actual_append = std::cmp::min(available_space, append_length);
        self.current_size += actual_append;
        actual_append
    }
    
    // 从资源消费数据
    pub fn consume_data(&mut self, consume_length: u32) -> u32 {
        let actual_consume = std::cmp::min(self.current_size, consume_length);
        self.current_size -= actual_consume;
        actual_consume
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferEvent {
    Producer(ProducerEvent),
    Consumer(ConsumerEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferEventResult{
    Producer(ProducerEventResult),
    Consumer(ConsumerEventResult),
}


// 生产者事件处理结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProducerEventResult {
    pub resource_index: usize,
    pub accepted_length: u32,
    pub remaining_bytes: u32,
}

impl ProducerEventResult {
    pub fn new(resource_index: usize, accepted_length: u32, remaining_length: u32) -> Self {
        ProducerEventResult {
            resource_index,
            accepted_length,
            remaining_bytes: remaining_length,
        }
    }
}


// 消费者事件处理结果：要求读取的字节中有多少成功了，剩下的失败了
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerEventResult {
    pub consumed_bytes: u32,
    pub remaining_bytes: u32,
}

impl ConsumerEventResult {
    pub fn new(consumed_bytes: u32, remaining_bytes: u32) -> Self {
        ConsumerEventResult {
            consumed_bytes,
            remaining_bytes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputBuffer {
    pub resource : Vec<Resource>
}

impl InputBuffer {
    
    // 添加使用全局配置的构造函数
    pub fn new_from_global() -> Self {
        InputBuffer {
            resource: Vec::new()
        }
    }
    
    
    pub fn set_resource(&mut self, resource : Vec<Resource>) -> anyhow::Result<()> {
        self.resource = resource;
        Ok(())
    }
    
    pub fn is_empty(&self) -> bool {
        self.resource.is_empty()
    }
    
    // 处理生产者事件
    pub fn handle_producer_event(&mut self, event: &ProducerEvent) -> anyhow::Result<ProducerEventResult> {
        if event.resource_index >= self.resource.len() {
            return anyhow::bail!("Resource index out of bounds");
        }
        
        let resource = &mut self.resource[event.resource_index];
        let accepted_length = resource.append_data(event.append_length);
        let remaining_length = event.append_length - accepted_length;
        
        Ok(ProducerEventResult {
            resource_index: event.resource_index,
            accepted_length,
            remaining_bytes: remaining_length,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultBuffer {
    pub destination: Option<EnhancedResource>,
    pub handle_pc : Option<usize>
}

impl ResultBuffer {
    // 添加使用全局配置的构造函数
    pub fn new_from_global() -> Self {
        ResultBuffer {
            destination: None,
            handle_pc: None,
        }
    }
    
    pub fn set_destination(&mut self, destination: EnhancedResource, pc : usize) -> Result<(), &'static str> {
        self.destination = Some(destination);
        self.handle_pc = Some(pc);
        Ok(())  
    }
    
    // 处理消费者事件
    pub fn handle_consumer_event(&mut self, event: &ConsumerEvent) -> anyhow::Result<ConsumerEventResult> {
        if let Some(ref mut destination) = self.destination {
            let consume_length = std::cmp::min(event.maximum_consume_length as u32, destination.current_size);
            let consumed_bytes = destination.consume_data(consume_length);
            let remaining_bytes = destination.current_size;
            
            Ok(ConsumerEventResult {
                consumed_bytes,
                remaining_bytes,
            })
        } else {
            anyhow::bail!("No destination resource set in ResultBuffer")
        }
    }
    
    // 检查是否有可消费的数据
    pub fn has_consumable_data(&self) -> bool {
        if let Some(ref destination) = self.destination {
            destination.current_size > 0
        } else {
            false
        }
    }
    
    pub fn increase_result_inner(&mut self, new_bytes: u32) -> anyhow::Result<()> {
        if let Some(ref mut destination) = self.destination {
            destination.append_data(new_bytes);
            Ok(())
        } else {
            anyhow::bail!("No destination resource set in ResultBuffer")
        }
    }

    // 新增：获取已消耗的字节数
    pub fn get_consumed_bytes(&self) -> anyhow::Result<u32> {
        if let Some(ref destination) = self.destination {
            Ok(destination.consumed_bytes)
        } else {
            anyhow::bail!("No destination resource set in ResultBuffer")
        }
    }
    
    // 新增：检查是否已完成所有处理
    pub fn is_completed(&self) -> bool {
        if let Some(ref destination) = self.destination {
            destination.is_completed()
        } else {
            false
        }
    }

    pub fn all_data_ready(&self) -> bool {
        if let Some(d) = &self.destination { 
            if d.current_size == d.target_size {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerEvent {
    pub maximum_consume_length : u32,
}

impl ConsumerEvent {
    pub fn new(maximum_consume_length: u32) -> Self {
        ConsumerEvent {
            maximum_consume_length
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProducerEvent {
    pub resource_index : usize,
    pub append_length : u32,
}

impl ProducerEvent {
    pub fn new(resource_index: usize, append_length: u32) -> Self {
        ProducerEvent {
            resource_index,
            append_length
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferOwnerType {
    /// 功能单元
    FunctionUnit(FunctionUnitKeyType),
    /// 内存单元端口
    MemoryUnit(MemoryUnitKeyType),
    /// 未分配（初始状态）
    Unassigned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferPair {
    pub input_buffer: InputBuffer,
    pub result_buffer: ResultBuffer,
    pub current_instruction: Option<crate::inst::Inst>,  // 存储当前正在处理的指令信息
    pub owner: BufferOwnerType,  // 新增：表示buffer所属的单元类型，不再是Option
}

impl BufferPair {
    pub fn new() -> Self {
        BufferPair {
            input_buffer: InputBuffer::new_from_global(),
            result_buffer: ResultBuffer::new_from_global(),
            current_instruction: None,
            owner: BufferOwnerType::Unassigned, // 默认为未分配状态
        }
    }
    
    // 新增：创建时直接指定所属单元
    pub fn new_with_owner(owner: BufferOwnerType) -> Self {
        BufferPair {
            input_buffer: InputBuffer::new_from_global(),
            result_buffer: ResultBuffer::new_from_global(),
            current_instruction: None,
            owner,
        }
    }
    
    // 修改：设置buffer所属的单元类型
    pub fn set_owner(&mut self, owner: BufferOwnerType) {
        self.owner = owner;
    }
    
    // 修改：获取buffer所属的单元类型
    pub fn get_owner(&self) -> &BufferOwnerType {
        &self.owner
    }
    
    pub fn handle_buffer_event(&mut self, event: BufferEvent) -> anyhow::Result<BufferEventResult> {
        debug!("[BufferPair] Processing buffer event: {:?} for owner: {:?}", event, self.owner);
        
        let result = match event {
            BufferEvent::Producer(producer_event) => {
                debug!("[FORWARD-INFO] ===== BufferPair handling producer event =====");
                debug!("[FORWARD-INFO] Owner: {:?}", self.owner);
                debug!("[FORWARD-INFO] Resource index: {}, append length: {} bytes", 
                    producer_event.resource_index, producer_event.append_length);
                
                let result = self.input_buffer.handle_producer_event(&producer_event)?;
                
                debug!("[FORWARD-INFO] Producer result: accepted={} bytes, remaining={} bytes", 
                    result.accepted_length, result.remaining_bytes);
                debug!("[FORWARD-INFO] ==========================================");
                
                Ok(BufferEventResult::Producer(result))
            },
            BufferEvent::Consumer(consumer_event) => {
                debug!("[FORWARD-INFO] ===== BufferPair handling consumer event =====");
                debug!("[FORWARD-INFO] Owner: {:?}", self.owner);
                debug!("[FORWARD-INFO] Maximum consume length: {} bytes", 
                    consumer_event.maximum_consume_length);
                
                let result = self.result_buffer.handle_consumer_event(&consumer_event)?;
                
                debug!("[FORWARD-INFO] Consumer result: consumed={} bytes, remaining={} bytes", 
                    result.consumed_bytes, result.remaining_bytes);
                debug!("[FORWARD-INFO] ==========================================");
                
                Ok(BufferEventResult::Consumer(result))
            }
        };
        
        // 显示当前处理状态
        self.debug_status();
        
        result
    }
    
    // 添加一个新方法用于显示当前状态
    pub fn debug_status(&self) {
        debug!("[BufferPair] Current status for {:?}:", self.owner);
        
        // 显示输入缓冲区状态
        debug!("[BufferPair] Input buffer resources:");
        for (i, resource) in self.input_buffer.resource.iter().enumerate() {
            let resource_type = match &resource.resource_type {
                ResourceType::Register(reg_type) => format!("Register({:?})", reg_type),
                ResourceType::Memory => "Memory".to_string(),
            };
            debug!("[BufferPair]   Resource[{}]: Type={}, Progress={}/{} bytes ({:.2}%)", 
                i, resource_type, resource.current_size, resource.target_size,
                (resource.current_size as f32 / resource.target_size as f32) * 100.0);
        }
        
        // 显示输出缓冲区状态
        if let Some(ref dest) = self.result_buffer.destination {
            let resource_type = match &dest.resource_type {
                ResourceType::Register(reg_type) => format!("Register({:?})", reg_type),
                ResourceType::Memory => "Memory".to_string(),
            };
            debug!("[BufferPair] Output buffer: Type={}, Current={}/{} bytes ({:.2}%), Consumed={} bytes, Total processed={}/{} bytes ({:.2}%)", 
                resource_type, dest.current_size, dest.target_size,
                (dest.current_size as f32 / dest.target_size as f32) * 100.0,
                dest.consumed_bytes,
                dest.total_processed_bytes(), dest.target_size,
                (dest.total_processed_bytes() as f32 / dest.target_size as f32) * 100.0);
        } else {
            debug!("[BufferPair] Output buffer: Not set");
        }
        
        // 显示当前指令信息
        if let Some(ref inst) = self.current_instruction {
            debug!("[BufferPair] Current instruction: {:?}", inst);
        } else {
            debug!("[BufferPair] Current instruction: None");
        }
    }
    
    pub fn increase_result(&mut self, new_bytes : u32) -> anyhow::Result<()> {
        debug!("[BufferPair] Increasing result buffer by {} bytes for {:?}", new_bytes, self.owner);
        let result = self.result_buffer.increase_result_inner(new_bytes);
        
        // 显示更新后的状态
        if let Some(ref dest) = self.result_buffer.destination {
            debug!("[BufferPair] Result buffer after increase: Current={}/{} bytes ({:.2}%), Total processed={}/{} bytes ({:.2}%)", 
                dest.current_size, dest.target_size,
                (dest.current_size as f32 / dest.target_size as f32) * 100.0,
                dest.total_processed_bytes(), dest.target_size,
                (dest.total_processed_bytes() as f32 / dest.target_size as f32) * 100.0);
        }
        
        result
    }

    pub(crate) fn get_memory_input_current_bytes(&self) -> anyhow::Result<u32> {
        if self.input_buffer.resource.iter().any(|r| r.resource_type != ResourceType::Memory && !r.is_full()) {
            return Ok(0);
        }

        let v = self.input_buffer.resource.iter()
        .filter(|r| r.resource_type == ResourceType::Memory)
        .map(|r| r.current_size)
        .collect::<Vec<_>>();
        
        match v.len() {
            0 => anyhow::bail!("No memory resource in the input buffer"),
            1 => Ok(v[0]),
            _ => anyhow::bail!("Multiple memory resource in the input buffer")
        }
    }

    pub(crate) fn get_register_input_current_bytes(&self) -> anyhow::Result<u32> {

        let v = self.input_buffer.resource.iter()
        .filter(|r| matches!(r.resource_type, ResourceType::Register(_)))
        .map(|r| r.current_size)
        .collect::<Vec<_>>();

        match v.len() {
            0 | 1 => anyhow::bail!("Not enough register resource in the input buffer"),
            2 => Ok(v[1]),
            _ => anyhow::bail!("Multiple register resource in the input buffer")
        }
    }

    pub(crate) fn get_longest_input_resource_bytes(&self) -> anyhow::Result<u32> {
        Ok(
            self.input_buffer.resource.iter()
            .map(|r| r.current_size)
            .max()
            .unwrap_or(0)
        )
    }

    pub(crate) fn set_input(&mut self, resource : Vec<Resource>) -> anyhow::Result<()> {
        debug!("[BufferPair] Setting input buffer for {:?} with {} resources", self.owner, resource.len());
        
        for (i, res) in resource.iter().enumerate() {
            let resource_type = match &res.resource_type {
                ResourceType::Register(reg_type) => format!("Register({:?})", reg_type),
                ResourceType::Memory => "Memory".to_string(),
            };
            debug!("[BufferPair]   Resource[{}]: Type={}, Target size={} bytes", 
                i, resource_type, res.target_size);
        }
        
        self.input_buffer.set_resource(resource)
    }
    
    pub(crate) fn set_output(&mut self, destination : EnhancedResource, pc : usize) {
        let resource_type = match &destination.resource_type {
            ResourceType::Register(reg_type) => format!("Register({:?})", reg_type),
            ResourceType::Memory => "Memory".to_string(),
        };
        
        debug!("[BufferPair] Setting output buffer for {:?}: Type={}, Target size={} bytes", 
            self.owner, resource_type, destination.target_size);
        
        self.result_buffer.destination = Some(destination);
        self.result_buffer.handle_pc = Some(pc);
    }
    
    // 添加一个方法来设置当前指令信息
    pub fn set_current_instruction(&mut self, instruction: crate::inst::Inst) {
        self.current_instruction = Some(instruction);
    }
    
    // 添加一个方法来获取当前指令信息
    pub fn get_current_instruction(&self) -> Option<&crate::inst::Inst> {
        self.current_instruction.as_ref()
    }

    pub fn get_current_input_bytes(&self) -> anyhow::Result<u32> {
        if self.input_buffer.resource.iter()
        .any(|r| matches!(r.resource_type, ResourceType::Register(RegisterType::VectorRegister(_)))) {
            Ok(self.input_buffer.resource.iter()
                        .filter(|r| matches!(r.resource_type, ResourceType::Register(RegisterType::VectorRegister(_))))
                        .map(|r| r.current_size)
                        .min()
                        .unwrap_or(0))
        } else {
            Ok(self.input_buffer.resource.iter()
                        .map(|r| r.current_size)
                        .min()
                        .unwrap_or(0))
        }
    }
    
    /// 检查结果缓冲区是否已完成读取
    pub fn is_result_completed(&self) -> bool {
        self.result_buffer.is_completed()
    }
    
    /// 清空 BufferPair 的所有缓冲区
    pub fn clear(&mut self) {
        debug!("[BufferPair] Clearing all buffers for {:?}", self.owner);
        
        // 清除当前指令信息
        self.current_instruction = None;
        
        // 清空 input_buffer
        self.input_buffer.resource.clear();
        
        // 清空 result_buffer
        // self.result_buffer.destination = None;
        // 暂时不清空 result_buffer
        
        debug!("[BufferPair] All buffers cleared successfully");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnhancedResource {
    pub resource_type: ResourceType,
    pub target_size: u32,       // 总共需要多少字节
    pub current_size: u32,      // 当前存储了多少字节
    pub consumed_bytes: u32     // 已经消耗了多少字节
}

impl EnhancedResource {
    pub fn new(resource_type: ResourceType, target_size: u32) -> Self {
        EnhancedResource {
            resource_type,
            target_size,
            current_size: 0,
            consumed_bytes: 0
        }
    }
    
    pub fn is_full(&self) -> bool {
        self.current_size >= self.target_size
    }
    
    pub fn remaining_capacity(&self) -> u32 {
        if self.target_size > self.current_size {
            self.target_size - self.current_size
        } else {
            0
        }
    }
    
    // 添加数据到资源
    pub fn append_data(&mut self, append_length: u32) -> u32 {
        let available_space = self.remaining_capacity();
        let actual_append = std::cmp::min(available_space, append_length);
        self.current_size += actual_append;
        actual_append
    }
    
    // 从资源消费数据
    pub fn consume_data(&mut self, consume_length: u32) -> u32 {
        let actual_consume = std::cmp::min(self.current_size, consume_length);
        self.current_size -= actual_consume;
        self.consumed_bytes += actual_consume;
        actual_consume
    }
    
    pub fn total_processed_bytes(&self) -> u32 {
        self.current_size
    }

    // 检查是否已完成所有处理
    pub fn is_completed(&self) -> bool {
        self.consumed_bytes == self.target_size
    }
}


