# RISC-V Vector Simulator Test Report

## Test 3: Dependency Chain & RAW

**Simulator Version:** ruscv-vector-sim
**Configuration File:** config.toml
**Test Address Range:** 0x44 - 0x60

---

## 1. Executive Summary

| Metric                                 | Value                                                          |
| -------------------------------------- | -------------------------------------------------------------- |
| **Total Cycles**                 | 19 cycles                                                      |
| **Total Instructions**           | 7                                                              |
| **Test Status**                  | Pass                                                           |
| **CPI (Cycles Per Instruction)** | 2.71                                                           |
| **Key Finding**                  | Long dependency chain, reduction operation has 9-cycle latency |

---

## 2. Test Environment Configuration

### 2.1 Hardware Configuration Parameters

```toml
[vector_config.software]
vl = 32          # Vector length (elements)
sew = 32         # Element width (bits)
lmul = 1         # Lane multiplier

[vector_config.hardware]
vlen = 4096      # Vector register length (bits)
lane_number = 8  # Number of lanes
```

**Derived Values:**

- Vector register size: 512 bytes
- Element size: 4 bytes
- Total vector operation size: 128 bytes (32 elements × 4 bytes)
- VL × SEW = 32 × 32 = 1024 bits ≤ 4096 bits ✓

### 2.2 Functional Unit Latency

| Unit Type          | Config Latency (cycles) |
| ------------------ | ----------------------- |
| Integer ALU        | 1                       |
| Integer Multiplier | 3                       |
| Float ALU          | 3                       |
| Float Multiplier   | 6                       |
| Memory Access Unit | 2                       |

**Note:**

- Vector ALU execution latency: Normal operations 6 cycles, reduction operations 9 cycles
- Actual execution latency includes data transfer and forwarding overhead

### 2.3 Port Configuration

| Port Type                  | Count    |
| -------------------------- | -------- |
| Vector Register Read Port  | 2        |
| Vector Register Write Port | 1        |
| Memory Read Port           | 3        |
| Memory Write Port          | 2        |
| Maximum Forward Bytes      | 32 bytes |

**Note:** This test is configured with `write_ports_limit = 1`

---

## 3. Instruction Sequence Analysis

### 3.1 Complete Instruction List

```riscv
# PC=0
vsetivli x0, 8, e32, m1, ta, ma    # Set vector configuration

# PC=1
vle32.v v0, (a0)                    # Load vector v0

# PC=2  
vle32.v v1, (a1)                    # Load vector v1

# PC=3
vmul.vv v2, v0, v1                  # v2 = v0 * v1 (Vector multiplication)

# PC=4
vredsum.vs v3, v2, v3               # v3 = sum(v2) + v3 (Vector reduction sum)

# PC=5
vadd.vx v4, v3, a2                  # v4 = v3 + a2 (Vector-scalar addition)

# PC=6
vse32.v v4, (a3)                    # Store v4
```

### 3.2 Key Dependencies

**RAW (Read After Write) Dependency Chain:**

```
PC=1: vle32.v v0, (a0)          → Write to v0
PC=3: vmul.vv v2, v0, v1        → Read v0

PC=2: vle32.v v1, (a1)          → Write to v1
PC=3: vmul.vv v2, v0, v1        → Read v1

PC=3: vmul.vv v2, v0, v1        → Write to v2
PC=4: vredsum.vs v3, v2, v3     → Read v2 

PC=4: vredsum.vs v3, v2, v3     → Write to v3
PC=5: vadd.vx v4, v3, a2        → Read v3 

PC=5: vadd.vx v4, v3, a2        → Write to v4
PC=6: vse32.v v4, (a3)          → Read v4
```

**Key Characteristic:** Forms a long dependency chain v0,v1 → v2 → v3 → v4, where each instruction depends on the result of the previous instruction

---

## 4. Cycle-by-Cycle Execution Analysis

### Cycle 0: Begin Execution

**Issued Instruction:** `vle32.v v0, (a0)` (PC=1)

**Operations:**

- ✓ Skip vsetivli (configuration instruction, does not occupy execution unit)
- ✓ Issue v0 load instruction to memory port 0
- ✓ Create write task for v0
- ✓ PC: 1 → 2

**Memory Port Status:**

- Read port 0: Occupied (v0, 0/128 bytes)
- Read ports 1-2: Idle

---

### Cycle 1: Issue v1 Load

**Issued Instruction:** `vle32.v v1, (a1)` (PC=2)

**Operations:**

- ✓ v0 loading (0→32 bytes)
- ✓ Issue v1 load instruction to memory port 1
- ✓ Create write task for v1
- ✓ PC: 2 → 3

**Memory Port Status:**

- Read port 0: Occupied (v0, 32/128 bytes)
- Read port 1: Occupied (v1, 0/128 bytes)

---

### Cycle 2: Issue vmul

**Issued Instruction:** `vmul.vv v2, v0, v1` (PC=3)

**Operations:**

- ✓ v0 forwarding (32→64 bytes)
- ✓ v1 forwarding (0→32 bytes)
- ✓ **Dependency check:** v0 and v1 data begin to be available through forwarding ✓
- ✓ **Issue to VectorMul unit** (actual execution requires 8 cycles)
- ✓ Create write task for v2
- ✓ PC: 3 → 4

**Data Forwarding:** vmul can read v0 and v1 through forwarding mechanism

**Memory Port Status:**

- Read port 0: Occupied (v0, 64/128 bytes)
- Read port 1: Occupied (v1, 32/128 bytes)

---

### Cycle 3: Issue vredsum

**Issued Instruction:** `vredsum.vs v3, v2, v3` (PC=4)

**Operations:**

- ✓ v0 forwarding (64→96 bytes)
- ✓ v1 forwarding (32→64 bytes)
- ✓ vmul executing (Cycle 2-9)
- ✓ **Dependency check:** v2 data begins to be available through forwarding ✓
- ✓ **Issue to VectorAlu unit** (actual execution requires 9 cycles, reduction operation)
- ✓ Create write task for v3
- ✓ PC: 4 → 5

**Key Point:** vredsum can read v2 being produced by vmul through forwarding mechanism

**Memory Port Status:**

- Read port 0: Occupied (v0, 96/128 bytes)
- Read port 1: Occupied (v1, 64/128 bytes)

---

### Cycle 4: v0 Load Complete

**Issued Instruction:** None

**Operations:**

- ✓ **v0 load completed** (Cycle 0-4, total 5 cycles)
- ✓ v1 forwarding (64→96 bytes)
- ✓ vmul continues execution (Cycle 2-9)
- ✓ vredsum continues execution (Cycle 3-11)

---

### Cycle 5: v1 Load Complete

**Issued Instruction:** None

**Operations:**

- ✓ **v1 load completed** (Cycle 1-5, total 5 cycles)
- ✓ vmul continues execution (Cycle 2-9)
- ✓ vredsum continues execution (Cycle 3-11)

---

### Cycle 6-9: Wait for vmul to Complete

**Issued Instruction:** None

**Operations:**

- Cycle 6-9: vmul continues execution
- Cycle 6-11: vredsum continues execution

---

### Cycle 10: vmul Complete

**Issued Instruction:** None

**Operations:**

- ✓ **vmul completed** (Cycle 2-9, total 8 cycles)
- ✓ v2 data fully available
- ✓ vredsum continues execution (Cycle 3-11)

---

### Cycle 11: vredsum Complete

**Issued Instruction:** None

**Operations:**

- ✓ **vredsum completed** (Cycle 3-11, total 9 cycles)
- ✓ v3 data available

---

### Cycle 12: Issue vadd

**Issued Instruction:** `vadd.vx v4, v3, a2` (PC=5)

**Operations:**

- ✓ **Dependency check:** v3 completed ✓
- ✓ **Issue to VectorAlu unit** (actual execution requires 6 cycles)
- ✓ Create write task for v4
- ✓ PC: 5 → 6

**Key Point:** vadd must wait for vredsum to complete before it can issue

---

### Cycle 13: Issue vse

**Issued Instruction:** `vse32.v v4, (a3)` (PC=6)

**Operations:**

- ✓ vadd executing (Cycle 12-17)
- ✓ v4 data begins to be available through forwarding
- ✓ **Issue v4 store to memory write port 0** (actual execution requires 6 cycles)
- ✓ PC: 6 → 7 (all instructions issued)

---

### Cycle 14-17: Wait for vadd to Complete

**Issued Instruction:** None

**Operations:**

- Cycle 14-17: vadd continues execution
- Cycle 14-18: vse continues execution

---

### Cycle 18: vadd and vse Complete

**Operations:**

- ✓ **vadd completed** (Cycle 12-17, total 6 cycles)
- ✓ **vse completed** (Cycle 13-18, total 6 cycles)
- ✓ v4 store complete

---

### Cycle 19: Simulation Ends

**Status Check:**

- ✓ Fetch unit idle: true
- ✓ Functional units idle: true
- ✓ Memory unit idle: true
- ✓ All operations complete

---

## 5. Key Mechanism Verification

### 5.1 RAW (Read After Write) Dependency Chain Handling 

**Test Scenario:**

```riscv
vmul.vv v2, v0, v1         # PC=3: Write to v2
vredsum.vs v3, v2, v3      # PC=4: Read v2 (depends on v2)
vadd.vx v4, v3, a2         # PC=5: Read v3 (depends on v3)
```

### 5.2 Data Forwarding 

**Observed Forwarding:**

1. **Load→Compute Forwarding:**

   - Cycle 2: vmul begins execution before v0 and v1 fully load
   - Data provided to vmul through forwarding mechanism (32 bytes/cycle)
2. **Compute→Compute Forwarding:**

   - Cycle 3: vredsum begins execution before vmul fully completes
   - Partial results of v2 provided to vredsum through forwarding mechanism
3. **Compute→Store Forwarding:**

   - Cycle 13: vse begins before vadd fully completes
   - Results of v4 provided to store unit through forwarding mechanism

**Configuration:**

```toml
[register]
maximum_forward_bytes = 32  # Maximum forward 32 bytes per cycle
```

---

### 5.3 Reduction Operation Characteristics 

**Observation:**

- `vredsum.vs` reduction operation execution latency is **9 cycles** (longer than normal vadd's 6 cycles)
- Reduction needs to accumulate 32 elements into 1 scalar, requiring more computation

**Impact:**

- vredsum becomes the bottleneck in the dependency chain
- vadd must wait for vredsum to complete, causing Cycles 5-11 to be in a waiting state

---

## 6. Execution Timeline Overview

```
Timeline (Test 3):
Cycle 0  : VLE v0 issued
Cycle 1  : VLE v1 issued
Cycle 2  : VMUL issued (v2←v0*v1)
Cycle 3  : VREDSUM issued (v3←sum(v2)+v3)
Cycle 4  : v0 completes (0-4=5 cycles)
Cycle 5  : v1 completes (1-5=5 cycles)
Cycle 6-9: VMUL and VREDSUM executing in parallel
Cycle 10 : VMUL completes (2-9=8 cycles)
Cycle 11 : VREDSUM completes (3-11=9 cycles)
Cycle 12 : VADD issued (v4←v3+a2)
Cycle 13 : VSE v4 issued
Cycle 14-17: VADD and VSE executing in parallel
Cycle 18 : VADD completes (12-17=6 cycles), VSE completes (13-18=6 cycles)
Cycle 19 : Simulation ends
```

---

## 7. Detailed Cycle Table

| Cycle | Issued Instruction | Active Units    | Completed Operations                                                    | Key Events                       | Notes                        |
| ----- | ------------------ | --------------- | ----------------------------------------------------------------------- | -------------------------------- | ---------------------------- |
| 0     | VLE v0             | Load0           | -                                                                       | v0 load begins                   | Start                        |
| 1     | VLE v1             | Load0,1         | -                                                                       | v1 load begins                   |                              |
| 2     | VMUL               | Load0,1,Mul     | -                                                                       | vmul begins (needs 8 cycles)     | v0,v1 forwarded to vmul      |
| 3     | VREDSUM            | Load0,1,Mul,Alu | -                                                                       | vredsum begins (needs 9 cycles)  | v2 forwarded to vredsum      |
| 4     | -                  | Load1,Mul,Alu   | **v0 completes** (5 cycles)                                       | v0 load complete                 |                              |
| 5     | -                  | Mul,Alu         | **v1 completes** (5 cycles)                                       | v1 load complete                 |                              |
| 6     | -                  | Mul,Alu         | -                                                                       | vmul and vredsum parallel        |                              |
| 7     | -                  | Mul,Alu         | -                                                                       | vmul and vredsum parallel        |                              |
| 8     | -                  | Mul,Alu         | -                                                                       | vmul and vredsum parallel        |                              |
| 9     | -                  | Mul,Alu         | -                                                                       | vmul and vredsum parallel        |                              |
| 10    | -                  | Alu             | **vmul completes** (8 cycles)                                     | v2 fully available               |                              |
| 11    | -                  | -               | **vredsum completes** (9 cycles)                                  | v3 ready                         | Dependency chain bottleneck  |
| 12    | VADD               | Alu             | -                                                                       | vadd begins (needs 6 cycles)     | Waiting for vredsum complete |
| 13    | VSE v4             | Alu,Store0      | -                                                                       | v4 store begins (needs 6 cycles) | All instructions issued      |
| 14    | -                  | Alu,Store0      | -                                                                       | vadd and store parallel          |                              |
| 15    | -                  | Alu,Store0      | -                                                                       | vadd and store parallel          |                              |
| 16    | -                  | Alu,Store0      | -                                                                       | vadd and store parallel          |                              |
| 17    | -                  | Alu,Store0      | -                                                                       | vadd and store parallel          |                              |
| 18    | -                  | Store0          | **vadd completes** (6 cycles), **vse completes** (6 cycles) | v4 store complete                |                              |
| 19    | -                  | -               | -                                                                       | -                                | **Simulation ends**    |

---

## 8. Appendix

### 8.1 Complete Configuration

```toml
[function_units.interger_alu]
latency = 1

[function_units.interger_multiplier]
latency = 3

[function_units.float_alu]
latency = 3

[function_units.float_multiplier]
latency = 6

[memory_units.load_store_unit]
latency = 2
max_access_width = 32
read_ports_limit = 3
write_ports_limit = 2

[vector_config.software]
vl = 32
sew = 32
lmul = 1

[vector_config.hardware]
vlen = 4096
lane_number = 8

[vector_register.ports]
read_ports_limit = 2
write_ports_limit = 1

[buffer]
input_maximum_size = 64
result_maximum_size = 64

[register]
maximum_forward_bytes = 32
```

---

### 8.2 Key Log Snippets

```
06:27:53 [INFO] Main simulation loop ended, total cycles: 19

06:27:53 [DEBUG] Vector configuration is valid: vl * sew <= vlen (32 * 32 <= 4096)

06:27:53 [DEBUG] Memory unit issued instruction: VLE { vrd: 0, rs1: 10, width: 32 } at cycle 0

06:27:53 [DEBUG] Memory unit issued instruction: VLE { vrd: 1, rs1: 11, width: 32 } at cycle 1

06:27:53 [DEBUG] Function unit VectorMul issued instruction: VMUL_VV { vrd: 2, vrs1: 1, vrs2: 0 } at cycle 2

06:27:53 [DEBUG] Function unit VectorAlu issued instruction: VREDSUM_VS { vrd: 3, vrs1: 3, vrs2: 2 } at cycle 3
  (Reduction operation: 9 cycles latency)

06:27:53 [DEBUG] Function unit VectorAlu issued instruction: VADD_VX { vrd: 4, rs1: 12, vrs2: 3 } at cycle 12
  (Waiting for vredsum to complete)

06:27:53 [DEBUG] Memory unit issued instruction: VSE { vrd: 4, rs1: 13, width: 32 } at cycle 13

06:27:53 [DEBUG] [FORWARD-INFO] Forward bytes: 32 bytes (max allowed: 32)
```

### 8.3 Gantt Chart

![1763016874338](image/test3_detailed_report/1763016874338.png)
