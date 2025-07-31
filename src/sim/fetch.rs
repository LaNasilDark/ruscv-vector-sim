

use crate::types::*;
use crate::inst::Inst;
use log::debug;
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
    pub fn fetch(&mut self) -> Option<Inst>{

        while self.pc < self.inst_memory.len() {
            if let Some(inst) = Inst::new(self.inst_memory[self.pc]) {
                return Some(inst);
            }
            self.pc += 1;
        }
        None
    }
    pub fn update_pc(&mut self, new_pc : usize) {
        self.pc = new_pc;
    }

    pub fn next_pc(&mut self) {
        debug!("Advancing PC from {} to {}", self.pc, self.pc + 1);
        self.update_pc(self.pc + 1);
    }

    pub fn is_empty(&self) -> bool {
        self.pc >= self.inst_memory.len()
    }

    pub fn get_pc(&self) -> usize {
        self.pc
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use crate::inst::*;


}