use clap::Parser;
use log::debug;
use ruscv_vector_sim::config::SimulatorConfig;
use ruscv_vector_sim::extract_file::ExtractFile;
use simplelog::*;
use std::fs::File;
use riscv_isa::{Decoder, Instruction, Target};
use std::str::FromStr;


use std::num::ParseIntError;

// 自定义解析器：将十六进制字符串转换为 u64
fn parse_hex(s: &str) -> Result<u64, ParseIntError> {
    // 检查是否有 0x 前缀，有则去掉
    let s = if s.starts_with("0x") {
        &s[2..]
    } else {
        s
    };
    // 从十六进制字符串解析为 u64
    u64::from_str_radix(s, 16)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the RISC-V binary
    #[arg(short, long)]
    input: String,

    /// Path to Configuration file
    #[arg(short, long)]
    config: String,

    /// Start address of the region to simulate (hex)
    #[arg(short, long, value_parser = parse_hex)]
    start_addr: u64,

    /// End address of the region to simulate (hex )
    #[arg(short, long, value_parser = parse_hex)]
    end_addr: u64,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn init_logger() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("sim.log").unwrap()),
        ]
    ).unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

    let args = Args::parse();

    let config_path = args.config;

    // 初始化全局配置
    SimulatorConfig::init_global_config(config_path.as_str())?;

    if let Some(config) = SimulatorConfig::get_global_config() {
        debug!("Simulator config: {:?}", config);
        
        // 输出向量配置信息
        debug!("Vector configuration:");
        debug!("  Software settings:");
        debug!("    Vector Length (vl): {}", config.vector_config.software.vl);
        debug!("    Scalar Element Width (sew): {} bits", config.vector_config.software.sew);
        debug!("    Lane Multiplier (lmul): {}", config.vector_config.software.lmul);
        debug!("  Hardware settings:");
        debug!("    Vector Register Length (vlen): {} bits", config.vector_config.hardware.vlen);
        debug!("    Vector Lane Number: {}", config.vector_config.hardware.lane_number);
        debug!("  Derived values:");
        debug!("    Vector Register Size: {} bytes", config.get_vector_register_bytes());
        debug!("    Element Size: {} bytes", config.get_element_bytes());
        debug!("    Total Vector Operation Size: {} bytes", config.get_total_vector_bytes());
        
        // 验证配置是否有效
        if config.vector_config.is_valid() {
            debug!("Vector configuration is valid: vl * sew <= vlen ({} * {} <= {})", 
                config.vector_config.software.vl,
                config.vector_config.software.sew,
                config.vector_config.hardware.vlen);
        } else {
            debug!("WARNING: Vector configuration is INVALID: vl * sew > vlen ({} * {} > {})", 
                config.vector_config.software.vl,
                config.vector_config.software.sew,
                config.vector_config.hardware.vlen);
        }
    }


    let target = Target::from_str("RV64IMFDAVZifencei_Zicsr_Zcd").unwrap();
    
    debug!("the start_addr is {:x}, end_addr is {:x}", args.start_addr, args.end_addr);
    let instructions = ExtractFile::extract_code_from_file(&args.input, args.start_addr, args.end_addr)?;

    let mut decoder = Decoder::from_le_bytes(target, &instructions[..]);
    let v = decoder.collect::<Vec<_>>();
    debug!("the instructions are {:?}", v);
    Ok(()) 

}
