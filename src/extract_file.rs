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
        .context("解析 ELF 文件失败")?;
    
        // 打印 ELF 文件基本信息
        debug!("ELF 文件信息:");
        debug!("  架构: {}", elf.header.e_machine);
        debug!("  入口点: 0x{:x}", elf.header.e_entry);
        debug!("  程序头数量: {}", elf.program_headers.len());
        debug!("  节头数量: {}", elf.section_headers.len());
    
        // 查找 .text 节
        let text_section = elf.section_headers.iter()
        .find(|sh| {
            elf.shdr_strtab.get_at(sh.sh_name)
                .map(|name| name == ".text")
                .unwrap_or(false)
        })
        .context("找不到 .text 节")?;
    
        // 获取 .text 节的内容
        let text_start = text_section.sh_offset;
        let text_size = text_section.sh_size;
        let text_data = &buffer[(text_start as usize) ..(text_start as usize + text_size as usize)];
    
        debug!("\n.text 节信息:");
        debug!("  地址: 0x{:x}", text_section.sh_addr);
        debug!("  大小: {} 字节", text_size);
        debug!(" test的前32位字节是 {}", text_data.iter().take(32).map(|x| format!("0x{:02x} ", x)).collect::<String>());
        debug!("  从{start_addr:x} 到 {end_addr:x} 的输出是：");
        
        debug!("{}", text_data.iter().skip((start_addr - text_section.sh_addr) as usize).take((end_addr - start_addr) as usize)
        .map(|x| format!("{:02x} ", x)).collect::<String>());
        

        // 输出 .text 节的前32字节（以十六进制格式）
        Ok(text_data.iter().skip((start_addr - text_section.sh_addr) as usize).take((end_addr - start_addr) as usize).cloned()
        .collect::<Vec<_>>())
    
    }
}
