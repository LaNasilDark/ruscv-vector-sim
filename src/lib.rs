use core::num;
use std::{cmp::Ordering, collections::BTreeSet, env::consts::FAMILY, iter::Once};

pub mod sim;
pub mod types;
pub mod inst;
pub mod config;
pub mod extract_file;
#[cfg(test)]
mod tests {
    use std::f32::consts::E;

    use crate::inst::{Destination, FuncInstruction, MemoryPlace, Resource};

    use simplelog::*;

    fn init() {
        CombinedLogger::init(
            vec![
                TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
                WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("sim.log").unwrap()),
            ]
        ).unwrap();
    }
    use anyhow::{Context, Result};
    use goblin::elf::{Elf, program_header::PT_LOAD};
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;
    use log::{info, warn, error};
    use crate::sim::vector_config::{Configuration, VectorConfig, HardwareConfig};
    use crate::sim::fetch::Fetch;
    use crate::sim::execute::Execute;
    fn read_file(elf_path : &str, start_line: usize, end_line : usize) -> anyhow::Result<()> {
        
        // 读取 ELF 文件内容
        let mut file = File::open(elf_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        // 解析 ELF 文件
        let elf = Elf::parse(&buffer)
            .context("解析 ELF 文件失败")?;
        
        // 打印 ELF 文件基本信息
        println!("ELF 文件信息:");
        println!("  架构: {}", elf.header.e_machine);
        println!("  入口点: 0x{:x}", elf.header.e_entry);
        println!("  程序头数量: {}", elf.program_headers.len());
        println!("  节头数量: {}", elf.section_headers.len());
        
        // 查找 .text 节
        let text_section = elf.section_headers.iter()
            .find(|sh| {
                elf.shdr_strtab.get_at(sh.sh_name)
                    .map(|name| name == ".text")
                    .unwrap_or(false)
            })
            .context("找不到 .text 节")?;
        
        // 获取 .text 节的内容
        let text_start = text_section.sh_offset as usize;
        let text_size = text_section.sh_size as usize;
        let text_data = &buffer[text_start..text_start + text_size];
        
        println!("\n.text 节信息:");
        println!("  地址: 0x{:x}", text_section.sh_addr);
        println!("  大小: {} 字节", text_size);
        println!("  从{start_line:x} 到 {end_line:x} 的输出是：");
        
        // 输出 .text 节的前32字节（以十六进制格式）
        text_data.iter().skip(start_line - text_start).take(end_line - start_line)
        .collect::<Vec<_>>().chunks(4)
        .for_each(|chunk| {
            println!("    {:02x} {:02x} {:02x} {:02x}",
                chunk[0], chunk[1], chunk[2], chunk[3]);
        });
        
        Ok(())
    }
    // fn jacobi_instruction_stream(vlen_bytes : usize) -> Vec<Instruction> {
    //     vec![
    //         Instruction::new(
    //             Destination::new(MemoryPlace::ScalarRegister(20), 8),
    //             vec![
    //                 Resource::new(MemoryPlace::Memory, 8)
    //             ],
    //             1,
    //             "ld	s4, 0(s0)"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::ScalarRegister(15), 8),
    //             vec![
    //                 Resource::new(MemoryPlace::Memory, 8)
    //             ],
    //             1,
    //             "ld	a5, -8(s0)"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::ScalarRegister(10), 8), 
    //             vec![
    //                 Resource::new(MemoryPlace::ScalarRegister(15), 8),
    //                 Resource::new(MemoryPlace::ScalarRegister(31), 8)
    //             ],
    //             1,
    //             "add a0, a5, t6"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::FloatingPointRegister(15), 8), 
    //             vec![
    //                 Resource::new(MemoryPlace::Memory, 8),
    //             ],
    //             1,
    //             "fld	fa5, 0(a0)"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::ScalarRegister(15), 8), 
    //             vec![
    //                 Resource::new(MemoryPlace::ScalarRegister(15), 8),
    //                 Resource::new(MemoryPlace::ScalarRegister(18), 8),
    //             ],
    //             1,
    //             "add a5, a5, s2"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::FloatingPointRegister(14), 8), 
    //             vec![
    //                 Resource::new(MemoryPlace::Memory, 8),
    //             ],
    //             1,
    //             "fld fa4, 0(a5)"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::ScalarRegister(20), 8), 
    //             vec![
    //                 Resource::new(MemoryPlace::ScalarRegister(20), 8), 
    //                 Resource::new(MemoryPlace::ScalarRegister(19), 8), 
    //             ],
    //             1,
    //             "add	s4, s4, s3"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::VectorRegister(11), vlen_bytes), 
    //             vec![
    //                 Resource::new(MemoryPlace::Memory, vlen_bytes)
    //             ],
    //             1,
    //             "vle64.v	v11, (s4)"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::VectorRegister(12), vlen_bytes), 
    //             vec![
    //                 Resource::new(MemoryPlace::VectorRegister(9), vlen_bytes),
    //             ],
    //             1,
    //             "vfslide1up.vf	v12, v9, fa5"
    //         ),

    //         Instruction::new(
    //             Destination::new(MemoryPlace::VectorRegister(13), vlen_bytes), 
    //             vec![
    //                 Resource::new(MemoryPlace::VectorRegister(9), vlen_bytes),
    //             ],
    //             1,
    //             "vfslide1down.vf	v13, v9, fa4"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::VectorRegister(12), vlen_bytes), 
    //             vec![
    //                 Resource::new(MemoryPlace::VectorRegister(12), vlen_bytes),
    //                 Resource::new(MemoryPlace::VectorRegister(9), vlen_bytes),
    //             ],
    //             3, 
    //             "vfadd.vv	v12, v9, v12"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::VectorRegister(12), vlen_bytes), 
    //             vec![
    //                 Resource::new(MemoryPlace::VectorRegister(12), vlen_bytes),
    //                 Resource::new(MemoryPlace::VectorRegister(13), vlen_bytes),
    //             ],
    //             3, 
    //             "vfadd.vv	v12, v12, v13"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::ScalarRegister(10), 8), 
    //             vec![
    //                 Resource::new(MemoryPlace::Memory, 8),
    //             ],
    //             1, 
    //             "ld	a0, 0(a4)"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::VectorRegister(10), vlen_bytes), 
    //             vec![
    //                 Resource::new(MemoryPlace::VectorRegister(11), vlen_bytes),
    //                 Resource::new(MemoryPlace::VectorRegister(10), vlen_bytes),
    //             ],
    //             3, 
    //             "vfadd.vv	v10, v11, v10"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::VectorRegister(10), vlen_bytes), 
    //             vec![
    //                 Resource::new(MemoryPlace::VectorRegister(12), vlen_bytes),
    //                 Resource::new(MemoryPlace::VectorRegister(10), vlen_bytes),
    //             ],
    //             3, 
    //             "vfadd.vv	v10, v12, v10"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::VectorRegister(10), vlen_bytes), 
    //             vec![
    //                 Resource::new(MemoryPlace::VectorRegister(10), vlen_bytes),
    //                 Resource::new(MemoryPlace::VectorRegister(8), vlen_bytes),
    //             ],
    //             4, 
    //             "vfmul.vv	v10, v12, v10"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::ScalarRegister(10), 8), 
    //             vec![
    //                 Resource::new(MemoryPlace::ScalarRegister(19), 8),
    //                 Resource::new(MemoryPlace::ScalarRegister(10), 8),
    //             ],
    //             1, 
    //             "add	a0, a0, s3"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::Memory, vlen_bytes), 
    //             vec![
    //                 Resource::new(MemoryPlace::VectorRegister(10), vlen_bytes)
    //             ],
    //             1, 
    //             "vse64.v	v10, (a0)"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::ScalarRegister(9), 8), 
    //             vec![
    //                 Resource::new(MemoryPlace::ScalarRegister(9), 8),
    //             ],
    //             1, 
    //             "addi	s1, s1, -1"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::ScalarRegister(8), 8), 
    //             vec![
    //                 Resource::new(MemoryPlace::ScalarRegister(8), 8),
    //             ],
    //             1, 
    //             "addi	s0, s0, 8"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::ScalarRegister(14), 8), 
    //             vec![
    //                 Resource::new(MemoryPlace::ScalarRegister(14), 8),
    //             ],
    //             1, 
    //             "addi	a4, a4, 8"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::VectorRegister(10), vlen_bytes), 
    //             vec![
    //                 Resource::new(MemoryPlace::VectorRegister(9), vlen_bytes)
    //             ],
    //             1, 
    //             "vmv1r.v	v10, v9"
    //         ),
    //         Instruction::new(
    //             Destination::new(MemoryPlace::VectorRegister(9), vlen_bytes), 
    //             vec![
    //                 Resource::new(MemoryPlace::VectorRegister(11), vlen_bytes)
    //             ],
    //             1, 
    //             "vmv.v.v	v9, v11"
    //         ),
    //     ]
    // }

    // fn test_jacobi_with_config(element_number : usize, sew : usize, lane_number : usize) {
    //     info!("This is a test for jacobi execution");
    //     let config = Configuration::new(
    //         VectorConfig::new(element_number, sew, 1),
    //         HardwareConfig::new(element_number * sew, lane_number)
    //     );

    //     let target_bytes = config.vector_config.total_length();
        
    //     let inst_memory = jacobi_instruction_stream(target_bytes);

    //     let mut fetch = Fetch::new();
    //     fetch.load(inst_memory);

    //     // 这里的16和32 都是临时设置的
    //     let mut execute = Execute::new(16, 32, config);
        
    //     // 模拟运行的部分
    //     let mut num_cycle : usize = 0;

    //     while !fetch.is_empty() || !execute.is_empty() {
    //         info!("Now simulate the cycle {}", num_cycle);
            

    //         execute.execute_serial();

    //         let inst = fetch.fetch();
    //         match inst {
    //             Some(inst) => {
    //                 info!("Fetch instruction: {:?}", inst);
    //                 match execute.push(inst) {
    //                     Ok(_) => {
    //                         info!("Push instruction to execute queue success");
    //                         fetch.next_pc();
    //                     },
    //                     Err(s) => {
    //                         info!("Push instruction to execute queue failed: {}", s);
    //                     }
    //                 }
    //             },
    //             None => {}
    //         }
    //         num_cycle += 1;
    //     }

    //     info!("The simulation is finished");
    //     info!("The number of cycles is {num_cycle}");
    // }
    // #[test]
    // fn test_jacobi() {
    //     init();
    //     // 目前写一个合适的前端还是有点复杂
    //     // read_file("./appendix/jacobi-2d_vector.exe", 0x10d16, 0x10d60);
    //     test_jacobi_with_config(4, 64, 4);
    //     test_jacobi_with_config(8, 64, 4);
    //     test_jacobi_with_config(16, 64, 4);
    //     test_jacobi_with_config(32, 64, 4);
    //     test_jacobi_with_config(64, 64, 4);
    // }
    

}

struct Solution;
impl Solution {
    pub fn longest_valid_parentheses(s: String) -> i32 {
        let prefix = std::iter::once(0).chain(
        s
        .chars()
        .into_iter()
        .map(|c| match c {
            '(' => 1,
            ')' => -1,
            _ => unreachable!()
        })
        .scan(0, |acc, x| {
            *acc = *acc + x;
            Some(*acc)
        }))
        .collect::<Vec<_>>();

        let mut sorted_with_index = prefix
        .into_iter()
        .enumerate()
        .collect::<Vec<_>>();
        sorted_with_index.sort_by(|(a,b),(c,d)| 
            match b.cmp(d) {
                Ordering::Less => Ordering::Less,
                Ordering::Equal => a.cmp(c),
                Ordering::Greater => Ordering::Greater 
            }
        );

        //println!("{:?}", sorted_with_index);
        let mut state = (0, sorted_with_index[0].1);
        let range_with_diff = sorted_with_index.iter()
        .cloned()
        .enumerate()
        .skip(1)
        .chain(std::iter::once((sorted_with_index.len(), (0, 0))))
        .map(|(id, (_, val))| {
            let (last_id, last_val) = state;
            //println!("last_id:{:?}, id:{:?} val:{:?}",last_id, id, val);
            if id == sorted_with_index.len() {
                return Some((last_id.clone(), id - 1));
            }
            
            if val != last_val {
                
                let res = (last_id.clone(), id - 1);
                state.0 = id;
                state.1 = val;
                Some(res)
            } else {
                None
            }
        })
        .flatten()
        .collect::<Vec<(usize, usize)>>();
        //println!("{:?}", range_with_diff);

        let mut s = BTreeSet::new();
        range_with_diff.into_iter()
        .map(| (l,r)| {
            
            let mut last_id : Option<usize> = None;
            let mut max_length : i32 = 0;
            sorted_with_index[l..=r].iter().for_each(
                |x| {
                    let (id, _) =  x;
                    match (last_id, s.range(..id).next_back()) {
                        (None, _) => {last_id = Some(*id);},
                        (Some(l_id), None) => {
                            max_length = max_length.max((id - l_id) as i32);
                        },
                        (Some(l_id), Some(&pred)) => {
                            if l_id <= pred {
                                last_id = Some(*id);
                            } else {
                                max_length = max_length.max((id - l_id) as i32);
                            }
                        }
                    }
                }
            );
            
            sorted_with_index[l..=r].iter().for_each(
                |(id,_)| {
                    s.insert(*id);
                }
            );
            Some(max_length)
        }
        ).flatten()
        .max()
        .unwrap_or(0)
    }
}

#[test]
fn test_solution() {
    Solution::longest_valid_parentheses("(()".to_string());
}