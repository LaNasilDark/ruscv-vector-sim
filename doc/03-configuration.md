# RISC-V Vector Simulator Configuration Documentation

This document provides a comprehensive guide to all configuration parameters available in the RISC-V Vector Simulator. The parameters are organized by functional categories to help you understand their relationships and usage.

## Configuration File Structure

The simulator uses a TOML configuration file [config.toml](../config.toml) to define various hardware and software parameters. The configuration is structured into several main sections:

## 1. Function Unit Latency Configuration

Function units are the computational components that execute different types of instructions. Each function unit has a configurable latency that determines how many cycles it takes to complete an operation.

### Integer Function Units

```toml
[function_units.interger_alu]
latency = 1  # Arithmetic Logic Unit latency in cycles

[function_units.interger_multiplier]
latency = 3  # Integer multiplication latency in cycles

[function_units.interger_divider]
latency = 6  # Integer division latency in cycles
```

### Floating-Point Function Units

```toml
[function_units.float_alu]
latency = 3  # Floating-point ALU latency in cycles

[function_units.float_multiplier]
latency = 6  # Floating-point multiplication latency in cycles

[function_units.float_divider]
latency = 10  # Floating-point division latency in cycles
```

### Branch Unit

```toml
[function_units.branch_unit]
latency = 1  # Branch instruction latency in cycles
```

**Notes:**
- Processing of branch-related instructions has not been implemented yet.
- These latencies affect both scalar and vector operations of the corresponding types
- Vector operations use the same latency as their scalar counterparts (e.g., vector integer ALU uses `interger_alu.latency`)

## 2. Memory Unit Configuration

The memory unit handles all load and store operations, including vector memory operations.

```toml
[memory_units.load_store_unit]
latency = 2                # Memory access latency in cycles
max_access_width = 32      # Maximum memory access width in bytes per cycle
read_ports_limit = 3       # Number of simultaneous read operations
write_ports_limit = 2      # Number of simultaneous write operations
```

**Parameter Details:**
- **latency**: Base latency for memory operations
- **max_access_width**: Determines memory bandwidth - larger values allow more data transfer per cycle
- **read_ports_limit**: Controls memory read parallelism
- **write_ports_limit**: Controls memory write parallelism

## 3. Vector Configuration

Vector configuration is divided into software and hardware settings that work together to define the vector processing capabilities.

### Software Vector Configuration

```toml
[vector_config.software]
vl = 32    # Vector Length - number of elements in vector operations
sew = 32   # Scalar Element Width in bits (supports 8/16/32/64)
lmul = 1   # Lane Multiplier - vector register group width multiplier
```

**Parameter Details:**
- **vl (Vector Length)**: Determines how many elements are processed in each vector operation
- **sew (Scalar Element Width)**: Size of each vector element in bits
- **lmul (Lane Multiplier)**: Multiplier for vector register grouping (affects register usage) (It has not been implemented yet)

### Hardware Vector Configuration

```toml
[vector_config.hardware]
vlen = 4096      # Vector register length in bits
lane_number = 4  # Number of parallel vector processing lanes
```

**Parameter Details:**
- **vlen**: Physical size of each vector register in bits
- **lane_number**: Number of parallel processing lanes for vector operations

**Important Constraint:**
The configuration must satisfy: `vl × sew ≤ vlen`

**Derived Values:**
- Vector register size in bytes: `vlen ÷ 8`
- Active vector data size: `(vl × sew) ÷ 8` bytes
- Element size: `sew ÷ 8` bytes

## 4. Vector Register Port Configuration

Controls the number of simultaneous read and write operations on vector registers.

```toml
[vector_register.ports]
read_ports_limit = 1   # Simultaneous read ports per vector register
write_ports_limit = 1  # Simultaneous write ports per vector register
```

**Usage Notes:**
- Higher port limits allow more parallel operations but increase hardware complexity
- For unlimited ports, set these values to a large number (e.g., 64)
- Write port limit of 1 is typically sufficient with proper register allocation

## 5. Buffer Configuration

Buffers manage the flow of data between different simulator components and enable instruction chaining.

```toml
[buffer]
input_maximum_size = 64   # Input buffer size in bytes
result_maximum_size = 64  # Result buffer size in bytes
```

**Parameter Details:**
- **input_maximum_size**: Maximum size of input buffers for staging instruction operands
- **result_maximum_size**: Maximum size of result buffers for instruction outputs

## 6. Register Configuration

Controls register file behavior and data forwarding capabilities.

```toml
[register]
maximum_forward_bytes = 32  # Maximum bytes for register forwarding
```

**Parameter Details:**
- **maximum_forward_bytes**: Controls the amount of data that can be forwarded between instructions to reduce pipeline stalls

## Configuration Validation

The simulator automatically validates the configuration when loaded:

1. **Vector Configuration Validation**: Ensures `vl × sew ≤ vlen`
2. **Parameter Range Validation**: Checks that all parameters are within acceptable ranges
3. **Consistency Validation**: Verifies that related parameters are compatible

## Example Complete Configuration

```toml
# Function unit latencies
[function_units.interger_alu]
latency = 1

[function_units.interger_multiplier]
latency = 3

[function_units.float_alu]
latency = 3

[function_units.float_multiplier]
latency = 6

[function_units.interger_divider]
latency = 6

[function_units.float_divider]
latency = 10

[function_units.branch_unit]
latency = 1

# Memory configuration
[memory_units.load_store_unit]
latency = 2
max_access_width = 32
read_ports_limit = 3
write_ports_limit = 2

# Vector software configuration
[vector_config.software]
vl = 32
sew = 32
lmul = 1

# Vector hardware configuration
[vector_config.hardware]
vlen = 4096
lane_number = 4

# Vector register ports
[vector_register.ports]
read_ports_limit = 1
write_ports_limit = 1

# Buffer configuration
[buffer]
input_maximum_size = 64
result_maximum_size = 64

# Register configuration
[register]
maximum_forward_bytes = 32
```

