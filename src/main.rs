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

    let simulator_config = SimulatorConfig::load_from_file(config_path.as_str())?;

    debug!("Simulator config: {:?}", simulator_config);


    let target = Target::from_str("RV64IMFDAVZifencei_Zicsr_Zcd").unwrap();
    
    debug!("the start_addr is {:x}, end_addr is {:x}", args.start_addr, args.end_addr);
    let instructions = ExtractFile::extract_code_from_file(&args.input, args.start_addr, args.end_addr)?;

    let mut decoder = Decoder::from_le_bytes(target, &instructions[..]);
    let v = decoder.collect::<Vec<_>>();
    debug!("the instructions are {:?}", v);
    Ok(()) 

}
