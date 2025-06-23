

use crate::types::*;
use crate::inst::{FuncInstruction, Inst};
pub struct Fetch {
    pc : usize,
    inst_memory : Vec<riscv_isa::Instruction>
}

impl Fetch {
    pub fn new() -> Fetch {
        Fetch {
            pc : 0,
            inst_memory : Vec::new()
        }
    }

    pub fn load(&mut self, inst_memory : Vec<riscv_isa::Instruction>) {
        self.inst_memory = inst_memory;
    }
    pub fn fetch(&self) -> Option<Inst>{
        if self.pc >= self.inst_memory.len() {
            return None;
        } else {

            Some(Inst::new(self.inst_memory[self.pc]))
        }


    }
    pub fn update_pc(&mut self, new_pc : usize) {
        self.pc = new_pc;
    }

    pub fn next_pc(&mut self) {
        self.update_pc(self.pc + 1);
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