# Quick Start Guide

This guide will help you get the RISC-V Vector Simulator running quickly with minimal setup.

## Prerequisites

- Rust toolchain (latest stable version recommended)
- RISC-V binary files to simulate
- Basic understanding of RISC-V architecture

## Installation

1. Clone the repository (if not already done):
   ```bash
   git clone <repository-url>
   cd ruscv-vector-sim
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

## Basic Usage

### Running Your First Simulation

The simplest way to run a simulation:

```bash
cargo run -- -i appendix/_chaintest/bin/chaintest -c ./config.toml -s 0x1023c -e 0x10250
```

**Command breakdown:**
- `-i`: Input RISC-V binary file
- `-c`: Configuration file
- `-s`: Start address (hexadecimal)
- `-e`: End address (hexadecimal)

### Parameter Adjustment

Please see [Configuration Reference](./03-configuration.md). 

However, you can mainly focus on "vl" and "sew" under [vector_config.software], as well as "lane_number" under [vector_config.hardware]. It is recommended that "vlen" be set equal to vl * sew. In the current situation, "vlen" has no practical use. If you want to know the ratio of the "dl" and "vl" you set, it is **lane_number * sew : vl * sew**.

For example, if you set "vl" to 32, "sew" to 32, and "lane_number" to 4, then the ratio is 4 * 4 : 32 * 4 = 1:8.

### Understanding the Output

The simulator will output:
- Cycle-by-cycle execution information
- Register state changes
- Memory access operations
- Total cycle statistics

### Quick Examples

1. **Chain Test Example** (Basic functionality):
   ```bash
   cargo run -- -i appendix/_chaintest/bin/chaintest -c ./config.toml -s 0x1023c -e 0x10250
   ```

2. **Matrix Multiplication Example**:
   ```bash
   cargo run -- -i appendix/_matmul/bin/matmul_vector.exe -c ./config.toml -s 0x1021a -e 0x1023a
   ```

3. **Jacobi 2D Example** (Complex vector operations):
   ```bash
   cargo run -- -i appendix/jacobi-2d_vector.exe -c ./config.toml -s 0x10cb2 -e 0x10d10
   ```
