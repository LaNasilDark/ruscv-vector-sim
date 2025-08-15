# Standard Operating Procedures (SOPs)

This document outlines standardized procedures for possible changes to the simulator.

## SOP-001: Add New Instruction

When adding a new instruction to the RISC-V Vector Simulator, the following code areas need to be modified:

### 1. Instruction Definition (vendor/riscv-isa/src/)

**IMPORTANT**: New instructions must first be defined in the riscv-isa library before they can be used in the simulator.

#### 1.1 Instruction Enum (vendor/riscv-isa/src/instruction.rs)
Add the new instruction variant to the `Instruction` enum with appropriate fields:
```rust
pub enum Instruction {
    // ... existing instructions ...
    NEW_INSTRUCTION { rd: u8, rs1: u8, rs2: u8 },
}
```

#### 1.2 Instruction Decoding (vendor/riscv-isa/src/decode/)
Implement decoding logic in the appropriate decoder file:
- `full.rs`: For standard 32-bit instructions
- `compressed.rs`: For compressed 16-bit instructions

Add the instruction's opcode, funct3, funct7, and other encoding fields to properly decode the binary instruction.

### 2. Instruction Classification (src/inst.rs)

In the `Inst::new()` function, add classification logic for the new instruction:
- **Memory instructions** (load/store): Route to `Inst::Mem(MemInst::new(riscv_isa))`
- **Function instructions** (arithmetic/logic): Route to `Inst::Func(FuncInst::new(riscv_isa))`
- **Vector configuration instructions** (vsetvl, etc.): Return `None` (ignored by simulator)

```rust
match riscv_isa {
    Instruction::LD {..} | Instruction::FLD{..} | Instruction::VLE{..} 
    | Instruction::VSE {..} | Instruction::SD{..} => Some(Inst::Mem(MemInst::new(riscv_isa))),
    Instruction::VSETVL { .. } | Instruction::VSETIVLI { .. } 
    | Instruction::VSETVLI { .. } => None,
    // Add new instruction here
    _ => Some(Inst::Func(FuncInst::new(riscv_isa)))
}
```

### 3. Function Instruction Implementation (src/inst/func.rs)

For computational instructions, add implementation in `FuncInst::new()`:
- Define **destination register** (`destination`)
- Define **source operands** (`resource`)
- Specify **function unit type** (`func_unit_key`)

```rust
match riscv_isa {
    // ... existing instructions ...
    Instruction::NEW_INSTRUCTION { rd, rs1, rs2 } => {
        (RegisterType::VectorRegister(rd),
         vec![RegisterType::VectorRegister(rs1), RegisterType::VectorRegister(rs2)],
         FunctionUnitKeyType::VectorAlu)
    },
}
```

**Supported Function Unit Types:**
- `IntegerAlu`: Integer arithmetic and logic operations
- `VectorAlu`: Vector arithmetic and logic operations
- `VectorMul`: Vector multiplication operations
- `VectorMacc`: Vector multiply-accumulate operations
- `VectorSlide`: Vector slide operations
- `FloatAlu`: Floating-point arithmetic operations
- `FloatMul`: Floating-point multiplication operations

### 4. Memory Instruction Implementation (src/inst/mem.rs)

For memory instructions, add implementation in `MemInst::new()`:
- Define **direction** (`Direction::Read` or `Direction::Write`)
- Define **memory address dependency** (`MemAddr`)
- Define **target/source register** (`RegisterType`)

```rust
match riscv_isa {
    // ... existing instructions ...
    riscv_isa::Instruction::NEW_LOAD { rd, rs1, offset } => {
        (Direction::Read, 
         MemAddr::new(RegisterType::ScalarRegister(rs1)), 
         RegisterType::VectorRegister(rd))
    },
}
```

### 5. Function Unit Configuration (config.toml)

If the new instruction requires a new function unit type, add latency configuration:
```toml
[function_units.new_unit_name]
latency = X  # Set appropriate latency cycles
```

### 6. Simulator Initialization (src/sim.rs)

If a new function unit type was added, initialize it in `Simulator::new()`:
```rust
function_units.insert(FunctionUnitKeyType::NewUnit, 
    FunctionUnitType::Common(CommonFunctionUnit::new(FunctionUnitKeyType::NewUnit)));
```

### 7. Register Type Support (src/sim/register.rs)

Ensure the instruction's register types are supported:
- `ScalarRegister`: Scalar registers (x0-x31)
- `VectorRegister`: Vector registers (v0-v31)
- `FloatRegister`: Floating-point registers (f0-f31)

### Implementation Checklist

When adding a new instruction, follow this checklist:

- [ ] **Step 1**: Define instruction in `vendor/riscv-isa/src/instruction.rs`
- [ ] **Step 2**: Implement decoding logic in `vendor/riscv-isa/src/decode/`
- [ ] **Step 3**: Add instruction classification in `src/inst.rs`
- [ ] **Step 4**: Implement instruction logic in `src/inst/func.rs` or `src/inst/mem.rs`
- [ ] **Step 5**: Update configuration file if needed
- [ ] **Step 6**: Update simulator initialization if new function unit added
- [ ] **Step 7**: Test the new instruction with appropriate test cases

### Important Notes

1. **Dependency Order**: The riscv-isa library must be updated first, as the simulator depends on it for instruction definitions
2. **Instruction Classification**: Correctly identifying instruction type (function vs memory) is crucial for proper routing
3. **Resource Dependencies**: Accurately define source operands and destination registers for dependency tracking
4. **Function Unit Mapping**: Choose appropriate function unit types based on instruction semantics
5. **Configuration Consistency**: Ensure config file settings match code implementation
6. **Testing**: Thoroughly test new instructions with various operand combinations

### Example: Adding Vector Floating-Point Addition (VFADD_VV)

1. **riscv-isa definition**:
```rust
// In vendor/riscv-isa/src/instruction.rs
VFADD_VV { vrd: u8, vrs1: u8, vrs2: u8 },
```

2. **Simulator classification** (already exists in current codebase):
```rust
// In src/inst/func.rs
Instruction::VFADD_VV { vrd, vrs1, vrs2 } => {
    (RegisterType::VectorRegister(vrd),
     vec![RegisterType::VectorRegister(vrs1), RegisterType::VectorRegister(vrs2)],
     FunctionUnitKeyType::VectorAlu)
}
```

3. **Configuration**:
```toml
# Ensure this exists in config.toml
[function_units.vector_alu]
latency = 3
```

This systematic approach ensures proper integration of new instructions into the simulator architecture.

