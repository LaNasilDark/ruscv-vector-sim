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

    use crate::inst::{Destination, MemoryPlace, Resource};

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
    

}
