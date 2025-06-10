use std::{fmt, sync::{Arc, Mutex}};

#[derive(Debug,Clone, Copy, PartialEq, Eq)]
pub enum MemoryPlace {
    VectorRegister(usize),
    ScalarRegister(usize),
    FloatingPointRegister(usize),
    Memory
}
#[derive(Debug,Clone, Copy, PartialEq)]
pub struct Resource {
    pub source : MemoryPlace,
    pub target_bytes : usize
}


#[derive(Debug,Clone, PartialEq)]
pub struct Destination {
    pub target : MemoryPlace,
    pub target_bytes : usize,

}

#[derive(Clone, PartialEq)]
pub struct Instruction {
    pub destination : Destination,
    pub resource : Vec<Resource>,
    pub operation_cycle : usize,
    raw_string : String
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.raw_string)
    }
}

impl Resource {
    pub fn new(source : MemoryPlace, target_bytes : usize) -> Resource {
        Resource {
            source,
            target_bytes
        }
    }
}

impl Destination {
    pub fn new(target : MemoryPlace, target_bytes : usize) -> Destination {
        Destination {
            target,
            target_bytes
        }
    }
}
impl Instruction {
    pub fn new(destination : Destination, resource : Vec<Resource>, operation_cycle : usize, raw_string : &str) -> Instruction {
        Instruction {
            resource,
            destination,
            operation_cycle,
            raw_string : raw_string.to_string() 
        }
    }
}

trait Inst {
    fn try_get_forwarding(&self, forwarding_source : Vec<MemoryPlace>);
    fn try_execute(&self);
    fn try_write_back(&self);
}