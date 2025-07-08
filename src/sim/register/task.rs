use crate::{config::SimulatorConfig, sim::unit::{buffer::{BufferEvent, BufferEventResult, ConsumerEvent, ProducerEvent, ConsumerEventResult, ProducerEventResult}, *}};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterTask {
    pub current_place : u32,
    pub resource_index : usize, // index for the corresponding input buffer
    pub behavior : UnitBehavior,
    pub unit_key : UnitKeyType
}

impl RegisterTask {
    pub fn new(resource_index : usize, behavior : UnitBehavior, unit_key : UnitKeyType) -> Self {
        Self {
            current_place: 0,
            resource_index,
            behavior,
            unit_key
        }
    }

    pub fn generate_event(&self, update_bytes : u32) -> BufferEvent {
        match self.behavior {
            UnitBehavior::Read => BufferEvent::Consumer(
                ConsumerEvent::new(update_bytes)
            ),
            UnitBehavior::Write => BufferEvent::Producer(
                ProducerEvent::new(self.resource_index, update_bytes)
            )
        }
    }

    pub fn handle_result(&mut self, result : BufferEventResult) {
        match result {
            BufferEventResult::Consumer(ConsumerEventResult { consumed_bytes, remaining_bytes }) => {
                self.current_place = self.current_place + consumed_bytes;
            },
            BufferEventResult::Producer(ProducerEventResult { resource_index, accepted_length, remaining_bytes }) => {
                self.current_place = self.current_place + accepted_length;
            }
        }
    }
}