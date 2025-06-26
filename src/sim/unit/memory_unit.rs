use crate::{config::SimulatorConfig, sim::unit::buffer::{BufferEvent, BufferEventResult, BufferPair}};



pub struct LoadStoreUnit {
    latency: u32,
    max_access_width: u32,
    read_port_buffer : Vec<BufferPair>,
    write_port_buffer : Vec<BufferPair>
}

type PortNumberIdType = usize;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MemoryUnitKeyType {
    Load(PortNumberIdType),
    Store(PortNumberIdType)
}

impl LoadStoreUnit {
    pub fn new(latency: u32, max_access_width: u32) -> LoadStoreUnit {
        let read_port_count = SimulatorConfig::get_global_config().unwrap().get_memory_read_ports_limit();
        let write_port_count = SimulatorConfig::get_global_config().unwrap().get_memory_write_ports_limit();

        LoadStoreUnit {
            latency,
            max_access_width,
            read_port_buffer: Vec::with_capacity(read_port_count),
            write_port_buffer: Vec::with_capacity(write_port_count)
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
}