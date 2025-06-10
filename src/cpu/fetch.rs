

use crate::types::*;
use crate::inst::Instruction;
pub struct Fetch {
    pc : Addr,
    inst_memory : Vec<Instruction>
}

impl Fetch {
    pub fn new() -> Fetch {
        Fetch {
            pc : 0,
            inst_memory : Vec::new()
        }
    }

    pub fn load(&mut self, inst_memory : Vec<Instruction>) {
        self.inst_memory = inst_memory;
    }
    pub fn fetch(&self) -> Option<Instruction>{
        if (self.pc / 4) as usize >= self.inst_memory.len() {
            return None;
        }
        let inst = self.inst_memory[(self.pc / 4) as usize].clone();
        Some(inst)
    }
    pub fn update_pc(&mut self, new_pc : Addr) {
        self.pc = new_pc;
    }

    pub fn next_pc(&mut self) {
        self.update_pc(self.pc + 4);
    }

    pub fn is_empty(&self) -> bool {
        self.fetch() == None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::inst::*;


}