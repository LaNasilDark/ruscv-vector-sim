use clap::Parser;
use log::debug;
use ruscv_vector_sim::config::SimulatorConfig;
use simplelog::*;
use std::fs::File;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the RISC-V binary
    #[arg(short, long)]
    input: String,

    /// Path to Configuration file
    #[arg(short, long)]
    config: String,

    /// Start address of the region to simulate (hex or decimal)
    #[arg(short, long)]
    start_addr: Option<u64>,

    /// End address of the region to simulate (hex or decimal)
    #[arg(short, long)]
    end_addr: Option<u64>,

    /// Simulate a specific function by name
    #[arg(short, long)]
    function: Option<String>,

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

    Ok(()) 

}
