# RISC-V Vector Simulator Test Report

## Test 4: WAW

**Test Date:** November 13, 2025
**Simulator Version:** ruscv-vector-sim
**Configuration File:** config.toml
**Test Address Range:** 0x60 - 0x80

---

## 1. Executive Summary

| Metric                                 | Value                                                     |
| -------------------------------------- | --------------------------------------------------------- |
| **Total Cycles**                 | 22 cycles                                                 |
| **Total Instructions**           | 8                                                         |
| **Test Status**                  | Pass                                                      |
| **CPI (Cycles Per Instruction)** | 2.75                                                      |
| **Key Finding**                  | Multiple overwrites to v4 register, significant WAW delay |

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

| Unit Type            | Config Latency (cycles) |
| -------------------- | ----------------------- |
| Integer ALU          | 1                       |
| Integer Multiplier   | 3                       |
| Float ALU            | 3                       |
| Float Multiplier     | 6                       |
| Memory Access Unit   | 2                       |
| **Vector ALU** | **1**             |

**Note:** Actual execution latency includes data transfer and forwarding overhead

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
vle32.v v2, (a0)                    # Load vector v2

# PC=2  
vle32.v v3, (a1)                    # Load vector v3

# PC=3
vle32.v v4, (a2)                    # Load vector v4

# PC=4
vmv.v.v v5, v4                      # v5 = v4 (Vector move)

# PC=5
vmul.vv v4, v4, v3                  # v4 = v4 * v3 (Overwrite v4!)

# PC=6
vsub.vv v4, v4, v2                  # v4 = v4 - v2 (Overwrite v4 again!)

# PC=7
vse32.v v4, (a3)                    # Store v4
```

### 3.2 Key Dependencies

**WAW (Write After Write) Hazard - Consecutive overwrites to v4 register:**

```
PC=3: vle32.v v4, (a2)          → Write to v4 (1st time)
PC=5: vmul.vv v4, v4, v3        → Read v4, Write to v4 (2nd overwrite!)
PC=6: vsub.vv v4, v4, v2        → Read v4, Write to v4 (3rd overwrite!)
PC=7: vse32.v v4, (a3)          → Read v4
```

**RAW (Read After Write) Dependencies:**

```
PC=3: vle32.v v4, (a2)          → Write to v4
PC=4: vmv.v.v v5, v4            → Read v4

PC=3: vle32.v v4, (a2)          → Write to v4
PC=5: vmul.vv v4, v4, v3        → Read v4 (old value)

PC=2: vle32.v v3, (a1)          → Write to v3
PC=5: vmul.vv v4, v4, v3        → Read v3

PC=5: vmul.vv v4, v4, v3        → Write to v4 (new value)
PC=6: vsub.vv v4, v4, v2        → Read v4 (new value)

PC=1: vle32.v v2, (a0)          → Write to v2
PC=6: vsub.vv v4, v4, v2        → Read v2

PC=6: vsub.vv v4, v4, v2        → Write to v4 (final value)
PC=7: vse32.v v4, (a3)          → Read v4 (final value)
```

**Key Characteristic:** v4 register undergoes 3 writes (VLE → VMUL → VSUB), forming a WAW hazard chain

---

## 4. Cycle-by-Cycle Execution Analysis

### Cycle 0: Begin Execution

**Issued Instruction:** `vle32.v v2, (a0)` (PC=1)

**Operations:**

- ✓ Skip vsetivli (configuration instruction, does not occupy execution unit)
- ✓ Issue v2 load instruction to memory port 0
- ✓ Create write task for v2
- ✓ PC: 1 → 2

**Memory Port Status:**

- Read port 0: Occupied (v2, 0/128 bytes)
- Read ports 1-2: Idle

---

### Cycle 1: Issue v3 Load

**Issued Instruction:** `vle32.v v3, (a1)` (PC=2)

**Operations:**

- ✓ v2 loading (0→32 bytes)
- ✓ Issue v3 load instruction to memory port 1
- ✓ Create write task for v3
- ✓ PC: 2 → 3

**Memory Port Status:**

- Read port 0: Occupied (v2, 32/128 bytes)
- Read port 1: Occupied (v3, 0/128 bytes)

---

### Cycle 2: Issue v4 Load

**Issued Instruction:** `vle32.v v4, (a2)` (PC=3)

**Operations:**

- ✓ v2 forwarding (32→64 bytes)
- ✓ v3 forwarding (0→32 bytes)
- ✓ Issue v4 load instruction to memory port 2
- ✓ Create write task for v4 (1st write task)
- ✓ PC: 3 → 4

**Memory Port Status:**

- Read port 0: Occupied (v2, 64/128 bytes)
- Read port 1: Occupied (v3, 32/128 bytes)
- Read port 2: Occupied (v4, 0/128 bytes)

**Key Point:** v4's 1st write task begins (VLE)

---

### Cycle 3: Issue vmv

**Issued Instruction:** `vmv.v.v v5, v4` (PC=4)

**Operations:**

- ✓ v2 forwarding (64→96 bytes)
- ✓ v3 forwarding (32→64 bytes)
- ✓ v4 forwarding (0→32 bytes)
- ✓ **Dependency check:** v4 data begins to be available through forwarding ✓
- ✓ **Issue to VectorAlu unit** (actual execution requires 6 cycles)
- ✓ Create write task for v5
- ✓ PC: 4 → 5

**Data Forwarding:** vmv reads v4's load data through forwarding mechanism

**Memory Port Status:**

- Read port 0: Occupied (v2, 96/128 bytes)
- Read port 1: Occupied (v3, 64/128 bytes)
- Read port 2: Occupied (v4, 32/128 bytes)

---

### Cycle 4: v2 Load Complete

**Issued Instruction:** None

**Operations:**

- ✓ **v2 load completed** (Cycle 0-4, total 5 cycles)
- ✓ v3 forwarding (64→96 bytes)
- ✓ v4 forwarding (32→64 bytes)
- ✓ vmv continues execution (Cycle 3-8)

---

### Cycle 5: v3 Load Complete

**Issued Instruction:** None

**Operations:**

- ✓ **v3 load completed** (Cycle 1-5, total 5 cycles)
- ✓ v4 forwarding (64→96 bytes)
- ✓ vmv continues execution (Cycle 3-8)

---

### Cycle 6: v4 Load Complete

**Issued Instruction:** None

**Operations:**

- ✓ **v4 load completed** (Cycle 2-6, total 5 cycles)
- ✓ v4's 1st write task (VLE) completed
- ✓ vmv continues execution (Cycle 3-8)

**Key Point:** v4's VLE write task completes, but v4 has subsequent write tasks (vmul, vsub)

---

### Cycle 7: Issue vmul (Cannot Issue Earlier!)

**Issued Instruction:** `vmul.vv v4, v4, v3` (PC=5)

**Operations:**

- ✓ vmv continues execution (Cycle 3-8)
- ✓ **Dependency check:** v3 completed ✓, v4 completed ✓
- ✓ **Check WAW hazard:** v4's write port is free ✓ (VLE completed)
- ✓ **Issue to VectorMul unit** (actual execution requires 8 cycles)
- ✓ Create 2nd write task for v4 (VMUL)
- ✓ PC: 5 → 6

**Key Point:** vmul must wait until Cycle 7 to issue because:

1. Need to read complete v4 data (VLE completes at Cycle 6)
2. v4's write port needs to be free to add new write task

---

### Cycle 8: vmv Complete

**Issued Instruction:** None

**Operations:**

- ✓ **vmv completed** (Cycle 3-8, total 6 cycles)
- ✓ v5 data available
- ✓ vmul continues execution (Cycle 7-14)

---

### Cycle 9-14: Wait for vmul to Complete

**Issued Instruction:** None

**Operations:**

- Cycle 9-14: vmul continues execution

---

### Cycle 15: Issue vsub & vmul Complete

**Issued Instruction:** `vsub.vv v4, v4, v2` (PC=6)

**Operations:**

- ✓ **vmul completed** (Cycle 7-14, total 8 cycles)
- ✓ v4's 2nd write task (VMUL) completed
- ✓ **Dependency check:** v2 completed ✓, v4 (vmul result) completed ✓
- ✓ **Check WAW hazard:** v4's write port is free ✓ (VMUL completed)
- ✓ **Issue to VectorAlu unit** (actual execution requires 6 cycles)
- ✓ Create 3rd write task for v4 (VSUB)
- ✓ PC: 6 → 7

**Key Point:** vsub must wait for vmul to complete (Cycle 14), then issue at Cycle 15

---

### Cycle 16: Issue vse

**Issued Instruction:** `vse32.v v4, (a3)` (PC=7)

**Operations:**

- ✓ vsub executing (Cycle 15-20)
- ✓ v4 data begins to be available through forwarding
- ✓ **Issue v4 store to memory write port 0** (actual execution requires 6 cycles)
- ✓ PC: 7 → 8 (all instructions issued)

**Data Forwarding:** vse reads v4 result being produced by vsub through forwarding mechanism

---

### Cycle 17-20: Wait for vsub to Complete

**Issued Instruction:** None

**Operations:**

- Cycle 17-20: vsub continues execution
- Cycle 17-21: vse continues execution

---

### Cycle 21: vsub and vse Complete

**Operations:**

- ✓ **vsub completed** (Cycle 15-20, total 6 cycles)
- ✓ v4's 3rd write task (VSUB) completed
- ✓ **vse completed** (Cycle 16-21, total 6 cycles)
- ✓ v4 final value store complete

---

### Cycle 22: Simulation Ends

**Status Check:**

- ✓ Fetch unit idle: true
- ✓ Functional units idle: true
- ✓ Memory unit idle: true
- ✓ All operations complete

---

## 5. Key Mechanism Verification

### 5.1 WAW (Write After Write) Hazard Handling 

**Test Scenario:**

```riscv
vle32.v v4, (a2)           # PC=3: 1st write to v4
vmul.vv v4, v4, v3         # PC=5: 2nd write to v4 (Overwrite!)
vsub.vv v4, v4, v2         # PC=6: 3rd write to v4 (Overwrite again!)
```

---

### 5.2 WAW Delay Analysis 

**Timing Analysis:**

| Instruction | Issue Cycle | Complete Cycle | Delay Reason                                      |
| ----------- | ----------- | -------------- | ------------------------------------------------- |
| VLE v4      | 2           | 6              | -                                                 |
| VMUL v4     | 7           | 14             | Wait for VLE complete (Cycle 6) + 1 cycle check   |
| VSUB v4     | 15          | 20             | Wait for VMUL complete (Cycle 14) + 1 cycle check |

**Delay Calculation:**

- VMUL theoretical earliest issue: Cycle 3 (v3 and v4 both start forwarding)
- VMUL actual issue: Cycle 7
- **VMUL WAW delay:** 4 cycles
- VSUB theoretical earliest issue: Cycle 8 (v2 completes at Cycle 4, vmul starts at Cycle 7)
- VSUB actual issue: Cycle 15
- **VSUB WAW delay:** 7 cycles

**Total WAW delay:** 4 + 7 = **11 cycles**

---

### 5.3 Data Forwarding 

**Observed Forwarding:**

1. **Load→Compute Forwarding:**

   - Cycle 3: vmv begins execution before v4 fully loads
   - Data provided to vmv through forwarding mechanism (32 bytes/cycle)
2. **Compute→Store Forwarding:**

   - Cycle 16: vse begins before vsub fully completes
   - v4 results provided to store unit through forwarding mechanism

**Configuration:**

```toml
[register]
maximum_forward_bytes = 32  # Maximum forward 32 bytes per cycle
```

---

### 5.4 Write Port Limitation Effect 

**Configuration:**

```toml
[vector_register.ports]
write_ports_limit = 1  # Each vector register can only have 1 write task at a time
```

**Observation:**

- v4 register's 3 write tasks must execute **strictly serially**
- Each new write task must wait for previous write task to complete
- Write port limitation prevents data inconsistency caused by WAW hazards

**Comparison with Tests 1-3:**

- Tests 1-2: `write_ports_limit = 2`, allows 2 concurrent write tasks to same register
- Tests 3-4: `write_ports_limit = 1`, forces serialization of all write tasks
- Test 4 particularly demonstrates strict control of WAW with single write port

---

## 6. Execution Timeline Overview

```
Timeline (Test 4):
Cycle 0  : VLE v2 issued
Cycle 1  : VLE v3 issued
Cycle 2  : VLE v4 issued (v4 1st write begins)
Cycle 3  : VMV v5 issued (reads v4)
Cycle 4  : v2 completes (0-4=5 cycles)
Cycle 5  : v3 completes (1-5=5 cycles)
Cycle 6  : v4 completes (2-6=5 cycles) ← v4 1st write completes
Cycle 7  : VMUL issued (v4 2nd write begins, waiting for WAW hazard resolution)
Cycle 8  : VMV completes (3-8=6 cycles)
Cycle 9-14: VMUL executing
Cycle 15 : VMUL completes (7-14=8 cycles) ← v4 2nd write completes
           VSUB issued (v4 3rd write begins, waiting for WAW hazard resolution)
Cycle 16 : VSE v4 issued (reads vsub result through forwarding)
Cycle 17-20: VSUB executing
Cycle 21 : VSUB completes (15-20=6 cycles) ← v4 3rd write completes
           VSE completes (16-21=6 cycles)
Cycle 22 : Simulation ends
```

**Stalls Caused by WAW Hazards:**

- Cycle 3-6: VMUL waits for v4's VLE to complete (4 cycle stall)
- Cycle 8-14: VSUB waits for v4's VMUL to complete (7 cycle stall)

---

## 7. Detailed Cycle Table

| Cycle | Issued Instruction | Active Units  | Completed Operations                                                    | Key Events                       | Notes                                 |
| ----- | ------------------ | ------------- | ----------------------------------------------------------------------- | -------------------------------- | ------------------------------------- |
| 0     | VLE v2             | Load0         | -                                                                       | v2 load begins                   | Start                                 |
| 1     | VLE v3             | Load0,1       | -                                                                       | v3 load begins                   |                                       |
| 2     | VLE v4             | Load0,1,2     | -                                                                       | v4 load begins (write task #1)   | v4 1st write                          |
| 3     | VMV v5             | Load0,1,2,Alu | -                                                                       | vmv begins (needs 6 cycles)      | v4 forwarded to vmv                   |
| 4     | -                  | Load1,2,Alu   | **v2 completes** (5 cycles)                                       | v2 load complete                 |                                       |
| 5     | -                  | Load2,Alu     | **v3 completes** (5 cycles)                                       | v3 load complete                 |                                       |
| 6     | -                  | Alu           | **v4 completes** (5 cycles)                                       | v4 load complete (task #1 done)  | v4 1st write complete                 |
| 7     | VMUL v4            | Alu,Mul       | -                                                                       | vmul begins (write task #2)      | WAW resolved, v4 2nd write            |
| 8     | -                  | Mul           | **vmv completes** (6 cycles)                                      | v5 ready                         |                                       |
| 9     | -                  | Mul           | -                                                                       | vmul executing                   |                                       |
| 10    | -                  | Mul           | -                                                                       | vmul executing                   |                                       |
| 11    | -                  | Mul           | -                                                                       | vmul executing                   |                                       |
| 12    | -                  | Mul           | -                                                                       | vmul executing                   |                                       |
| 13    | -                  | Mul           | -                                                                       | vmul executing                   |                                       |
| 14    | -                  | Mul           | -                                                                       | vmul executing                   |                                       |
| 15    | VSUB v4            | Mul,Alu       | **vmul completes** (8 cycles)                                     | vmul done, vsub begins (#3)      | WAW resolved, v4 3rd write            |
| 16    | VSE v4             | Alu,Store0    | -                                                                       | v4 store begins (needs 6 cycles) | All instructions issued, v4 forwarded |
| 17    | -                  | Alu,Store0    | -                                                                       | vsub and store parallel          |                                       |
| 18    | -                  | Alu,Store0    | -                                                                       | vsub and store parallel          |                                       |
| 19    | -                  | Alu,Store0    | -                                                                       | vsub and store parallel          |                                       |
| 20    | -                  | Alu,Store0    | -                                                                       | vsub and store parallel          |                                       |
| 21    | -                  | Store0        | **vsub completes** (6 cycles), **vse completes** (6 cycles) | v4 store complete (task #3 done) | v4 3rd write complete                 |
| 22    | -                  | -             | -                                                                       | -                                | **Simulation ends**             |

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
write_ports_limit = 1  # Key configuration: single write port

[buffer]
input_maximum_size = 64
result_maximum_size = 64

[register]
maximum_forward_bytes = 32
```

---

### 8.2 Key Log Snippets

```
06:56:26 [INFO] Main simulation loop ended, total cycles: 22

06:56:26 [DEBUG] Vector configuration is valid: vl * sew <= vlen (32 * 32 <= 4096)

06:56:26 [DEBUG] Memory unit issued instruction: VLE { vrd: 2, rs1: 10, width: 32 } at cycle 0

06:56:26 [DEBUG] Memory unit issued instruction: VLE { vrd: 3, rs1: 11, width: 32 } at cycle 1

06:56:26 [DEBUG] Memory unit issued instruction: VLE { vrd: 4, rs1: 12, width: 32 } at cycle 2
  (v4 write task #1 starts)

06:56:26 [DEBUG] Function unit VectorAlu issued instruction: VMV_V_V { vrd: 5, vrs1: 4 } at cycle 3

06:56:26 [DEBUG] [WAW-DELAY] Cannot issue VMUL_VV { vrd: 4, vrs1: 3, vrs2: 4 } at cycles 3-6
  (Waiting for v4 write task #1 to complete)

06:56:26 [DEBUG] Function unit VectorMul issued instruction: VMUL_VV { vrd: 4, vrs1: 3, vrs2: 4 } at cycle 7
  (v4 write task #1 completed, write task #2 starts)

06:56:26 [DEBUG] [WAW-DELAY] Cannot issue VSUB_VV { vrd: 4, vrs1: 2, vrs2: 4 } at cycles 8-14
  (Waiting for v4 write task #2 to complete)

06:56:26 [DEBUG] Function unit VectorAlu issued instruction: VSUB_VV { vrd: 4, vrs1: 2, vrs2: 4 } at cycle 15
  (v4 write task #2 completed, write task #3 starts)

06:56:26 [DEBUG] Memory unit issued instruction: VSE { vrd: 4, rs1: 13, width: 32 } at cycle 16
  (Reading from v4 write task #3 via forwarding)

06:56:26 [DEBUG] [FORWARD-INFO] Forward bytes: 32 bytes (max allowed: 32)
```

---

### 8.3 Gantt Chart

![1763020354705](image/test4_detailed_report/1763020354705.png)
