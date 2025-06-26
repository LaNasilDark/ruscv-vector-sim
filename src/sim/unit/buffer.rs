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
    pub buffer_size : u32, // The size restriction of input buffer is the longest resource can't exceed this size
    pub resource : Vec<Resource>
}

impl InputBuffer {
    
    // 添加使用全局配置的构造函数
    pub fn new_from_global() -> Self {
        let config = SimulatorConfig::get_global_config().unwrap();
        InputBuffer {
            buffer_size: config.buffer.input_maximum_size,
            resource: Vec::new()
        }
    }
    
    
    pub fn add_resource(&mut self, resource: Resource) -> Result<(), &'static str> {
        if resource.target_size <= self.buffer_size {
            self.resource.push(resource);
            Ok(())
        } else {
            Err("Resource target size exceeds buffer size")
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.resource.is_empty()
    }
    
    // 处理生产者事件
    pub fn handle_producer_event(&mut self, event: &ProducerEvent) -> Result<ProducerEventResult, &'static str> {
        if event.resource_index >= self.resource.len() {
            return Err("Resource index out of bounds");
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
    pub buffer_size : u32,
    pub destination : Option<Resource>
}

impl ResultBuffer {
    
    // 添加使用全局配置的构造函数
    pub fn new_from_global() -> Self {
        let config = SimulatorConfig::get_global_config().unwrap();
        ResultBuffer {
            buffer_size: config.buffer.result_maximum_size,
            destination: None
        }
    }
    
    pub fn set_destination(&mut self, destination: Resource) -> Result<(), &'static str> {
        self.destination = Some(destination);
        Ok(())
    }
    
    // 处理消费者事件
    pub fn handle_consumer_event(&mut self, event: &ConsumerEvent) -> Result<ConsumerEventResult, &'static str> {
        if let Some(ref mut destination) = self.destination {
            let consume_length = std::cmp::min(event.maximum_consume_length as u32, destination.current_size);
            let consumed_bytes = destination.consume_data(consume_length);
            let remaining_bytes = destination.current_size;
            
            Ok(ConsumerEventResult {
                consumed_bytes,
                remaining_bytes,
            })
        } else {
            Err("No destination resource set in ResultBuffer")
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
    pub result_buffer : ResultBuffer
}

impl BufferPair {
    pub fn new() -> Self {
        BufferPair {
            input_buffer: InputBuffer::new_from_global(),
            result_buffer: ResultBuffer::new_from_global()
        }
    }

    pub fn handle_buffer_event(&mut self, event: BufferEvent) -> Result<BufferEventResult, &'static str> {
        match event {
            BufferEvent::Producer(producer_event) => {
                Ok(BufferEventResult::Producer(self.input_buffer.handle_producer_event(&producer_event)?))
            },
            BufferEvent::Consumer(consumer_event) => {
                Ok(BufferEventResult::Consumer(self.result_buffer.handle_consumer_event(&consumer_event)?))
            }
        }
    }
}

