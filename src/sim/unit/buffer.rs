use std::any;

use crate::sim::register::RegisterType;
use crate::config::SimulatorConfig;

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
    pub destination: Option<EnhancedResource>
}

impl ResultBuffer {
    // 添加使用全局配置的构造函数
    pub fn new_from_global() -> Self {
        ResultBuffer {
            destination: None
        }
    }
    
    pub fn set_destination(&mut self, destination: EnhancedResource) -> Result<(), &'static str> {
        self.destination = Some(destination);
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
    
    pub fn increase_result(&mut self, new_bytes: u32) -> anyhow::Result<()> {
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
pub struct BufferPair {
    pub input_buffer : InputBuffer,
    pub result_buffer : ResultBuffer,
    pub current_instruction: Option<crate::inst::Inst>  // 存储当前正在处理的指令信息
}

impl BufferPair {
    pub fn new() -> Self {
        BufferPair {
            input_buffer: InputBuffer::new_from_global(),
            result_buffer: ResultBuffer::new_from_global(),
            current_instruction: None
        }
    }
    
    pub fn handle_buffer_event(&mut self, event: BufferEvent) -> anyhow::Result<BufferEventResult> {
        match event {
            BufferEvent::Producer(producer_event) => {
                Ok(BufferEventResult::Producer(self.input_buffer.handle_producer_event(&producer_event)?))
            },
            BufferEvent::Consumer(consumer_event) => {
                Ok(BufferEventResult::Consumer(self.result_buffer.handle_consumer_event(&consumer_event)?))
            }
        }
    }

    pub fn increase_result(&mut self, new_bytes : u32) -> anyhow::Result<()> {
        self.result_buffer.increase_result(new_bytes)
    }

    pub(crate) fn get_memory_input_current_bytes(&self) -> anyhow::Result<u32> {
        if self.input_buffer.resource.iter().any(|r| r.resource_type != ResourceType::Memory && !r.is_full()) {
            return anyhow::bail!("There is a non-memory resource in the input buffer that is not full");
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
        if self.input_buffer.resource.iter().any(|r| matches!(r.resource_type, ResourceType::Register(_)) && r.is_full()) {
            return anyhow::bail!("There is a register resource in the input buffer that is full");
        }

        let v = self.input_buffer.resource.iter()
        .filter(|r| matches!(r.resource_type, ResourceType::Register(_)))
        .map(|r| r.current_size)
        .collect::<Vec<_>>();

        match v.len() {
            0 => anyhow::bail!("No register resource in the input buffer"),
            1 => Ok(v[0]),
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
        self.input_buffer.set_resource(resource)
    }

    pub(crate) fn set_output(&mut self, destination : EnhancedResource)  {
        self.result_buffer.destination = Some(destination)
    } 
    
    // 添加一个方法来设置当前指令信息
    pub fn set_current_instruction(&mut self, instruction: crate::inst::Inst) {
        self.current_instruction = Some(instruction);
    }
    
    // 添加一个方法来获取当前指令信息
    pub fn get_current_instruction(&self) -> Option<&crate::inst::Inst> {
        self.current_instruction.as_ref()
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
    
    // 获取总共已处理的字节数（当前存储 + 已消耗）
    pub fn total_processed_bytes(&self) -> u32 {
        self.current_size + self.consumed_bytes
    }
    
    // 检查是否已完成所有处理
    pub fn is_completed(&self) -> bool {
        self.total_processed_bytes() >= self.target_size
    }
}

