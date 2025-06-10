use std::{cmp::{min,max}, collections::VecDeque, sync::{Arc, Mutex}};
use crate::inst::{Instruction, MemoryPlace};

use super::vector_config::{self, Configuration, VectorConfig};
use log::{info, debug, error, warn};


pub struct ExecuteInst {
    inst : Instruction,
    forwarding : Vec<Option<Arc<Mutex<usize>>>>,
    current_length : Vec<Arc<Mutex<usize>>>,
    issue_pos : usize,
    event_queue : VecDeque<ChangeEvent>,
    target : Arc<Mutex<usize>> // target data (unit is byte)
}

pub struct Execute {
    execute_queue : VecDeque<ExecuteInst>,
    max_queue_size : usize,
    forwarding_bytes : usize, // memory bandwidth in bytes 
    config : Configuration  
}

#[derive(Debug)]
struct ChangeEvent {
    remaining_cycles : usize,
    new_pos : usize
}


impl ChangeEvent {
    pub fn new(remaining_cycles : usize, new_pos : usize) -> ChangeEvent {
        ChangeEvent {
            remaining_cycles,
            new_pos
        }
    }
}
impl Execute {
    pub fn new(max_queue_size : usize, forwarding_bytes : usize, config : Configuration) -> Execute {
        Execute {
            execute_queue : VecDeque::new(),
            max_queue_size,
            forwarding_bytes,
            config
        }
    }

    // 让我想想指令需要什么
    // 需要知道自己的源都在哪
    pub fn push(&mut self, inst : Instruction) -> Result<(), String> {
        if self.execute_queue.len() < self.max_queue_size {
            let mut v = vec![];
            for r in inst.resource.iter() {
                let mut flag = false;
                if r.source == MemoryPlace::Memory {
                    v.push(None);
                    flag = true;
                }
                for e in self.execute_queue.iter().rev() {
                    // Find the neareast instruction whose target is the resource
                    if e.inst.destination.target == r.source {
                        v.push(Some(e.target.clone()));
                        flag = true;
                        break;
                    } 
                }
                if flag == false {
                    v.push(None);
                }
            }
            let l = inst.resource.len();
            self.execute_queue.push_back(ExecuteInst {
                inst,
                forwarding : v,
                current_length : (0..l).map(|_| Arc::new(Mutex::new(0))).collect(),
                issue_pos : 0,
                event_queue : VecDeque::new(),
                target : Arc::new(Mutex::new(0)),
            });
            Ok(())
        } else {
            Err("Execute queue is full".to_owned())
        }
    }

    pub fn is_empty(&self) -> bool {
        self.execute_queue.len() == 0
    }

    pub(crate) fn print_instruction_state(&self) {
        debug!("Print the instructions in the execution queue");
        for (i, e) in self.execute_queue.iter().enumerate() {
            debug!("The {i}th instruction {:?}", e.inst);
            debug!("The Resource State of Instruction {:?}", e.inst);
            for (j, r) in e.current_length.iter().enumerate() {
                let current_length = r.lock().unwrap().clone();
                let target_length = e.inst.resource[j].target_bytes;
                debug!("For the {j}th resource, current length: {current_length}, target_length: {target_length}")
            }

            debug!("The issue position is {} bytes", e.issue_pos);
            debug!("The target data is {} bytes", e.target.lock().unwrap().clone());
        }
        debug!("The Print of the execution queue ends");
    }
    pub fn execute_serial(&mut self) {
        // 最开始：把所有的执行完毕的指令都弹出
        while self.execute_queue.len() > 0 {
            debug!("Execute queue has {} instructions", self.execute_queue.len());
            let current_target = self.execute_queue[0].target.lock().unwrap().clone();
            let des_target = self.execute_queue[0].inst.destination.target_bytes;
            if current_target == des_target {
                debug!("Instruction {:?} is executed", self.execute_queue[0].inst);
                self.execute_queue.pop_front();
            } else {
                // 只弹出最靠前的指令
                break;
            }
        }

        self.print_instruction_state();
        // 顺序执行的时候第一步先能转发的都转发
        debug!("Start Forwarding Process");
        for e in self.execute_queue.iter_mut() {
            debug!("Handle  Instruction {:?}", e.inst);
            for (i, f) in e.forwarding.iter().enumerate() {
                
                match f {
                    Some(f) => {
                        let forward_source = f.lock().unwrap().clone();
                        let current_length = e.current_length[i].lock().unwrap().clone(); 
                        debug!("The {i}th resource has forwarding source with {forward_source} bytes, current length is {current_length} bytes");
                        if forward_source > current_length {
                            let step = min(
                                self.forwarding_bytes,
                                e.inst.resource[i].target_bytes - current_length
                            );
                            if step > 0 {
                                // 进行一次转发
                                *e.current_length[i].lock().unwrap() += step;
                            }
                            
                        }
                    },
                    None => {
                        //  把这个forwarding_bytes当做线宽，如果是None的话就证明每次可以增加这么多

                        
                        let current_length = e.current_length[i].lock().unwrap().clone();

                        debug!("The {i}th resource has no need to be forwarded");
                        debug!("current_length: {}, target_bytes: {}", current_length, e.inst.resource[i].target_bytes);
                        let step = min(
                            self.forwarding_bytes,
                            e.inst.resource[i].target_bytes - current_length
                        );



                        if step > 0 {
                            debug!("step: {step}, change current_length to: {}", current_length + step);
                            *e.current_length[i].lock().unwrap() += step;
                        }
                    }
                }
            }
        }

        // 第二步是把在流水线里所有的event都减少一个cycle
        debug!("Start Executing Process");
        for e in self.execute_queue.iter_mut() {
            for event in e.event_queue.iter_mut() {
                event.remaining_cycles -= 1;
            }
            // 如果有执行完成的event，那么更新一次target
            if e.event_queue.len() > 0 {
                if e.event_queue[0].remaining_cycles == 0 {
                    *e.target.lock().unwrap() = e.event_queue[0].new_pos;
                    e.event_queue.pop_front();
                }
            }
        }
            
        

        // 第三步开始往流水线塞新的可执行指令
        debug!("Issue new events");
        for e in self.execute_queue.iter_mut() {
            let min_pos = e
            .current_length.iter()
            .map(|x| x.lock().unwrap().clone())
            .min()
            .unwrap();

            if e.issue_pos + self.config.vector_config.bytes_per_element() <= min_pos  {
                // 这里的逻辑是如果可以issue就立刻issue掉，没有额外的设置，比如每次必须攒足2个或者4个lane一起issue
                let element_issue = min(
                    (min_pos - e.issue_pos) / self.config.vector_config.bytes_per_element(), 
                    self.config.hardware_config.lane_number);

                let step_length = element_issue * self.config.vector_config.bytes_per_element();
                e.event_queue.push_back(
                    ChangeEvent::new(
                        e.inst.operation_cycle, // 这里默认所有操作都是3个cycle完成，之后可能要根据指令的type更改一下
                        e.issue_pos + step_length
                    )
                );
                e.issue_pos += step_length
            }

        }

    } 

}

