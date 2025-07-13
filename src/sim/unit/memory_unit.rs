use std::fmt::Write;

use crate::{config::SimulatorConfig, inst::mem::{Direction, MemInst}, sim::unit::buffer::{BufferEvent, BufferEventResult, BufferPair, ResourceType}};
use log::debug;


pub struct LoadStoreUnit {
    latency: u32,
    max_access_width: u32,
    read_port_buffer : Vec<BufferPair>,
    write_port_buffer : Vec<BufferPair>,
    read_port_used : Vec<Option<MemoryPortEventGenerator>>,
    write_port_used : Vec<Option<MemoryPortEventGenerator>>
}

type PortNumberIdType = usize;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MemoryUnitKeyType {
    Load(PortNumberIdType),
    Store(PortNumberIdType)
}

pub struct MemoryPortEventGenerator {
    index : usize,
    bytes_per_cycle : u32,
    raw_inst : MemInst,
    total_bytes : u32,
    current_pos : u32,
}

impl MemoryPortEventGenerator {
    pub fn new(index : usize, raw_inst : MemInst) -> Self {
        let config = SimulatorConfig::get_global_config().unwrap();
        Self {
            index,
            bytes_per_cycle: config.get_max_access_width(),
            raw_inst,
            total_bytes: raw_inst.get_total_bytes(),
            current_pos : 0,
        }
    }
}

impl LoadStoreUnit {


    // 新增：判断特定端口是否可以接受新指令的函数
    pub fn can_port_accept_new_instruction(&self, dir: Direction, port_index: usize) -> bool {
        match dir {
            Direction::Read => {
                // 检查端口是否被占用
                if self.read_port_used[port_index].is_some() {
                    return false;
                }
                
                // 检查ResultBuffer是否为空
                if let Some(ref destination) = self.read_port_buffer[port_index].result_buffer.destination {
                    if destination.current_size > 0 {
                        return false;
                    }
                }
                
                true
            },
            Direction::Write => {
                // 检查端口是否被占用
                if self.write_port_used[port_index].is_some() {
                    return false;
                }
                
                // 检查ResultBuffer是否为空
                if let Some(ref destination) = self.write_port_buffer[port_index].result_buffer.destination {
                    if destination.current_size > 0 {
                        return false;
                    }
                }
                
                true
            }
        }
    }

    // 新增：判断是否有可用端口可以接受新指令的函数
    pub fn can_accept_new_instruction(&self, dir: Direction) -> bool {
        match dir {
            Direction::Read => {
                // 查找可用的读端口
                for i in 0..self.read_port_used.len() {
                    if self.can_port_accept_new_instruction(Direction::Read, i) {
                        return true;
                    }
                }
                false
            },
            Direction::Write => {
                // 查找可用的写端口
                for i in 0..self.write_port_used.len() {
                    if self.can_port_accept_new_instruction(Direction::Write, i) {
                        return true;
                    }
                }
                false
            }
        }
    }
    pub fn new(latency: u32, max_access_width: u32) -> LoadStoreUnit {
        let read_port_count = SimulatorConfig::get_global_config().unwrap().get_memory_read_ports_limit();
        let write_port_count = SimulatorConfig::get_global_config().unwrap().get_memory_write_ports_limit();
    
        use crate::sim::unit::buffer::BufferOwnerType;
        
        // 创建并初始化缓冲区和端口使用状态
        let mut read_port_buffer = Vec::with_capacity(read_port_count);
        let mut write_port_buffer = Vec::with_capacity(write_port_count);
        let mut read_port_used = Vec::with_capacity(read_port_count);
        let mut write_port_used = Vec::with_capacity(write_port_count);
        
        // 初始化缓冲区和端口使用状态
        for i in 0..read_port_count {
            read_port_buffer.push(BufferPair::new_with_owner(BufferOwnerType::MemoryUnit(MemoryUnitKeyType::Load(i))));
            read_port_used.push(None);
        }
        
        for i in 0..write_port_count {
            write_port_buffer.push(BufferPair::new_with_owner(BufferOwnerType::MemoryUnit(MemoryUnitKeyType::Store(i))));
            write_port_used.push(None);
        }
    
        LoadStoreUnit {
            latency,
            max_access_width,
            read_port_buffer,
            write_port_buffer,
            read_port_used,
            write_port_used
        }
    }

    pub fn new_from_config(config: &crate::config::LoadStoreUnit) -> LoadStoreUnit {
        LoadStoreUnit::new(config.latency, config.max_access_width)
    }

    pub fn handle_buffer_event(&mut self, key : MemoryUnitKeyType, event : BufferEvent) -> BufferEventResult {
        let res = match key {
            MemoryUnitKeyType::Load(i) => {
                self.read_port_buffer[i].handle_buffer_event(event)
            },
            MemoryUnitKeyType::Store(i) => {
                self.write_port_buffer[i].handle_buffer_event(event)
            }
        };

        match res {
            Ok(r) => r,
            Err(err) => panic!("Buffer event handling error: {}", err)
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        // 检查所有读端口是否都空闲
        let read_ports_empty = self.read_port_used.iter().all(|port| port.is_none());
        // 检查所有写端口是否都空闲
        let write_ports_empty = self.write_port_used.iter().all(|port| port.is_none());
        
        // 只有当所有端口都空闲时，内存单元才被视为空闲
        read_ports_empty && write_ports_empty
    }
    pub fn handle_event_queue(&mut self) -> anyhow::Result<()> {
        // 第一步： 读口和写口分别传递数据
        let mut finish_read_port : Vec<usize> = Vec::new();
        for i in 0..self.read_port_used.len() {
            if let Some(ref mut port) = self.read_port_used[i] {
                let update_bytes = (self.read_port_buffer[i].get_memory_input_current_bytes()? - port.current_pos).min(port.bytes_per_cycle);
                if update_bytes > 0 {
                    port.current_pos += update_bytes;
                    self.read_port_buffer[i].increase_result(update_bytes)?;
                }
                // 判断读端口任务是否完成的条件
                if port.current_pos == port.total_bytes && self.read_port_buffer[i].is_result_completed() {
                    // DEBUG: 读端口完成条件满足 - 当前位置等于总字节数且结果缓冲区已完成
                    debug!("Read port {} task completed: current_pos={}/{} bytes, result buffer completed={}", 
                           i, port.current_pos, port.total_bytes, self.read_port_buffer[i].is_result_completed());
                    finish_read_port.push(i);
                } else {
                    // DEBUG: 读端口任务未完成 - 当前进度和完成条件
                    debug!("Read port {} task not completed: current_pos={}/{} bytes, result buffer completed={}", 
                           i, port.current_pos, port.total_bytes, self.read_port_buffer[i].is_result_completed());
                }
            }
        }

        finish_read_port.into_iter().for_each(|i| {
            // DEBUG: 清空读端口 - 释放资源
            debug!("Clearing read port {}: releasing resources", i);
            self.read_port_used[i] = None;
            self.read_port_buffer[i].clear();
        });
    
        let mut finish_write_port : Vec<usize> = Vec::new();
        for i in 0..self.write_port_used.len() {
            if let Some(ref mut port) = self.write_port_used[i] {
                let update_bytes = (self.write_port_buffer[i].get_register_input_current_bytes()? - port.current_pos).min(port.bytes_per_cycle);
                if update_bytes > 0 {
                    port.current_pos += update_bytes;
                    self.write_port_buffer[i].increase_result(update_bytes)?;
                }
                // 判断写端口任务是否完成的条件
                if port.current_pos == port.total_bytes && self.write_port_buffer[i].is_result_completed() {
                    // DEBUG: 写端口完成条件满足 - 当前位置等于总字节数且结果缓冲区已完成
                    debug!("Write port {} task completed: current_pos={}/{} bytes, result buffer completed={}", 
                           i, port.current_pos, port.total_bytes, self.write_port_buffer[i].is_result_completed());
                    finish_write_port.push(i);
                } else {
                    // DEBUG: 写端口任务未完成 - 当前进度和完成条件
                    debug!("Write port {} task not completed: current_pos={}/{} bytes, result buffer completed={}", 
                           i, port.current_pos, port.total_bytes, self.write_port_buffer[i].is_result_completed());
                }
            }
        }
        finish_write_port.into_iter().for_each(|i| {
            // DEBUG: 清空写端口 - 释放资源
            debug!("Clearing write port {}: releasing resources", i);
            self.write_port_used[i] = None;
            // 清除当前指令信息
            self.write_port_buffer[i].clear();
        });
        
    
        Ok(())
    }

    pub fn has_free_port(&self, dir : Direction) -> bool {
        match dir {
            Direction::Read => {
                self.read_port_used.iter().position(|used| used.is_none()).is_some()
            },
            Direction::Write => {
                self.write_port_used.iter().position(|used| used.is_none()).is_some()
            }
        }
    }

    fn set_port_event(&mut self, mem_inst : MemInst) -> anyhow::Result<usize> {
        match mem_inst.dir {
            Direction::Read => {
                let port_index = self.read_port_used.iter().position(|used| used.is_none()).unwrap();
                self.read_port_used[port_index] = Some(MemoryPortEventGenerator::new(port_index, mem_inst));
                Ok(port_index)
            },
            Direction::Write => {
                let port_index = self.write_port_used.iter().position(|used| used.is_none()).unwrap();
                self.write_port_used[port_index] = Some(MemoryPortEventGenerator::new(port_index, mem_inst));
                Ok(port_index)
            }
        }
    }
    pub fn issue(&mut self, mem_inst : MemInst) -> anyhow::Result<usize> {
        let index = self.set_port_event(mem_inst.clone())?;
        use crate::sim::unit::buffer::Resource;
        use crate::sim::unit::buffer::EnhancedResource;
        match mem_inst.dir {
        
            Direction::Read => {
                self.read_port_buffer[index].set_input(vec![Resource::new(ResourceType::Register(mem_inst.mem_addr.dependency), mem_inst.mem_addr.dependency.get_bytes()), Resource::new(crate::sim::unit::buffer::ResourceType::Memory, mem_inst.get_total_bytes())])?;

                self.read_port_buffer[index].set_output(EnhancedResource::new(crate::sim::unit::buffer::ResourceType::Register(mem_inst.reg), mem_inst.get_total_bytes()));
                
                // 记录当前正在处理的指令信息
                self.read_port_buffer[index].set_current_instruction(crate::inst::Inst::Mem(mem_inst.clone()));
            },
            Direction::Write => {
                self.write_port_buffer[index].set_input(vec![Resource::new(ResourceType::Register(mem_inst.mem_addr.dependency), mem_inst.mem_addr.dependency.get_bytes()),Resource::new(crate::sim::unit::buffer::ResourceType::Register(mem_inst.reg), mem_inst.get_total_bytes())])?;
                self.write_port_buffer[index].set_output(EnhancedResource::new(crate::sim::unit::buffer::ResourceType::Memory, mem_inst.get_total_bytes()));
                
                // 记录当前正在处理的指令信息
                self.write_port_buffer[index].set_current_instruction(crate::inst::Inst::Mem(mem_inst.clone()));
            }
        }

        Ok(index)
    }

        // 添加一个新函数，用于自动增加内存数据
    pub fn auto_increase_memory_data(&mut self) -> anyhow::Result<()> {
        // 处理读端口
        for i in 0..self.read_port_used.len() {
            if let Some(ref port) = self.read_port_used[i] {
                // 获取每周期可以从内存读取的字节数
                let memory_bytes_per_cycle = port.bytes_per_cycle;
                
                // 首先检查该端口的 input_buffer 中所有 Register 类型资源是否都已满
                let all_registers_full = !self.read_port_buffer[i].input_buffer.resource.iter()
                    .any(|r| matches!(r.resource_type, ResourceType::Register(_)) && !r.is_full());
                
                // 只有当所有 Register 类型资源都已满时，才处理 Memory 类型资源
                if all_registers_full {
                    // 遍历 input_buffer 中的资源
                    for resource in &mut self.read_port_buffer[i].input_buffer.resource {
                        // 只处理 Memory 类型的资源
                        if let ResourceType::Memory = resource.resource_type {
                            // 计算可以增加的字节数（不超过目标大小）
                            let bytes_to_add = memory_bytes_per_cycle.min(resource.target_size - resource.current_size);
                            if bytes_to_add > 0 {
                                // 增加当前字节数
                                resource.current_size += bytes_to_add;
                                debug!("Auto-increasing memory data: read port {}, adding {} bytes", i, bytes_to_add);
                            }
                        }
                    }
                } else {
                    debug!("Not increasing memory data for read port {} because not all register resources are full", i);
                }
            }
        }
        
        Ok(())
    }
    // 添加在LoadStoreUnit实现中的其他函数之后
    pub fn debug_port_status(&self) {
        debug!("Memory unit port status:");
        debug!("Read ports total: {}, Write ports total: {}", self.read_port_used.len(), self.write_port_used.len());
        
        // 显示读端口状态
        for (i, port) in self.read_port_used.iter().enumerate() {
            if let Some(ref port_gen) = port {
                debug!("Read port {}: Occupied - Instruction: {:?}, Processed: {}/{} bytes", 
                      i, port_gen.raw_inst.raw, port_gen.current_pos, port_gen.total_bytes);
            } else {
                debug!("Read port {}: Idle", i);
            }
        }
        
        // 显示写端口状态
        for (i, port) in self.write_port_used.iter().enumerate() {
            if let Some(ref port_gen) = port {
                debug!("Write port {}: Occupied - Instruction: {:?}, Processed: {}/{} bytes", 
                      i, port_gen.raw_inst.raw, port_gen.current_pos, port_gen.total_bytes);
            } else {
                debug!("Write port {}: Idle", i);
            }
        }
    }
}
