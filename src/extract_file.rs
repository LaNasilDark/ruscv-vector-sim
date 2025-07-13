use std::fs::File;
use anyhow::{anyhow, Context, Result};
use goblin::elf::{Elf, program_header::PT_LOAD};
use log::debug;
use std::io::Read;

pub struct ExtractFile;

impl ExtractFile {
    pub fn extract_code_from_file(file_path : &str, start_addr : u64, end_addr : u64) -> Result<Vec<u8>>{
        let mut file = File::open(file_path).expect("Failed to open file");
        debug!("Opening file: {}", file_path);
    
        if start_addr > end_addr {
            return Err(anyhow!("Start address cannot be greater than end address"));
        }
    
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
    
        // 解析 ELF 文件
        let elf = Elf::parse(&buffer)
        .context("Failed to parse ELF file")?;
    
        // 打印 ELF 文件基本信息
        debug!("ELF file information:");
        debug!("  Architecture: {}", elf.header.e_machine);
        debug!("  Entry point: 0x{:x}", elf.header.e_entry);
        debug!("  Program headers count: {}", elf.program_headers.len());
        debug!("  Section headers count: {}", elf.section_headers.len());
    
        // 查找 .text 节
        let text_section = elf.section_headers.iter()
        .find(|sh| {
            elf.shdr_strtab.get_at(sh.sh_name)
                .map(|name| name == ".text")
                .unwrap_or(false)
        })
        .context("Could not find .text section")?;
    
        // 获取 .text 节的内容
        let text_start = text_section.sh_offset;
        let text_size = text_section.sh_size;
        let text_data = &buffer[(text_start as usize) ..(text_start as usize + text_size as usize)];
    
        debug!("\n.text section information:");
        debug!("  Address: 0x{:x}", text_section.sh_addr);
        debug!("  Size: {} bytes", text_size);
        debug!(" First 32 bytes of test are {}", text_data.iter().take(32).map(|x| format!("0x{:02x} ", x)).collect::<String>());
        debug!("  Output from {start_addr:x} to {end_addr:x}:");
        
        debug!("{}", text_data.iter().skip((start_addr - text_section.sh_addr) as usize).take((end_addr - start_addr) as usize)
        .map(|x| format!("{:02x} ", x)).collect::<String>());
        

        // 输出 .text 节的前32字节（以十六进制格式）
        Ok(text_data.iter().skip((start_addr - text_section.sh_addr) as usize).take((end_addr - start_addr) as usize).cloned()
        .collect::<Vec<_>>())
    
    }
}
