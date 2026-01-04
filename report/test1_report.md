# RISC-V Vector Simulator Test Report

## Test 1: Same Destination Register & WAR (Write After Read)

**Simulator Version:** ruscv-vector-sim
**Configuration File:** config.toml
**Test Address Range:** 0x0 - 0x28

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

| Unit Type          | Latency (cycles) |
| ------------------ | ---------------- |
| Integer ALU        | 1                |
| Integer Multiplier | 3                |
| Float ALU          | 3                |
| Float Multiplier   | 6                |
| Memory Access Unit | 2                |

### 2.3 Port Configuration

| Port Type                  | Count    |
| -------------------------- | -------- |
| Vector Register Read Port  | 2        |
| Vector Register Write Port | 2        |
| Memory Read Port           | 3        |
| Memory Write Port          | 2        |
| Maximum Forward Bytes      | 32 bytes |

---

## 3. Instruction Sequence Analysis

### 3.1 Complete Instruction List

```riscv
# PC=0
vsetivli x0, 8, e32, m1, ta, ma    # Set vector configuration

# PC=1
vle32.v v2, (a2)                    # Load vector v2

# PC=2  
vle32.v v3, (a3)                    # Load vector v3

# PC=3
vle32.v v4, (a4)                    # Load vector v4

# PC=4
vle32.v v5, (a5)                    # Load vector v5

# PC=5
vadd.vv v1, v2, v3                  # v1 = v2 + v3 (First write to v1)

# PC=6
vmul.vx v6, v1, a0                  # v6 = v1 * a0 (Read v1)

# PC=7
vadd.vv v1, v4, v5                  # v1 = v4 + v5 (Second write to v1)

# PC=8
vse32.v v6, (a6)                    # Store v6

# PC=9
vse32.v v1, (a7)                    # Store v1
```

### 3.2 Key Dependencies

**WAR (Write After Read) Hazard:**

```
PC=5: vadd.vv v1, v2, v3    → Write to v1
PC=6: vmul.vx v6, v1, a0    → Read v1 (must read value written by PC=5)
PC=7: vadd.vv v1, v4, v5    → Write to v1 again (WAR: cannot overwrite before PC=6 completes read)
```

**RAW (Read After Write) Dependencies:**

```
PC=1: vle32.v v2, (a2)      → Write to v2
PC=5: vadd.vv v1, v2, v3    → Read v2 (must wait for v2 load to complete)

PC=2: vle32.v v3, (a3)      → Write to v3  
PC=5: vadd.vv v1, v2, v3    → Read v3

PC=5: vadd.vv v1, v2, v3    → Write to v1
PC=6: vmul.vx v6, v1, a0    → Read v1
```

---

## 4. Cycle-by-Cycle Execution Analysis

### Cycle 0: Begin Execution

**Issued Instruction:** `vle32.v v2, (a2)` (PC=1)

**Operations:**

- ✓ Skip vsetivli (configuration instruction, does not occupy execution unit)
- ✓ Issue v2 load instruction to memory port 0
- ✓ Create write task for v2
- ✓ PC: 1 → 2

**Memory Port Status:**

- Read port 0: Occupied (v2, 0/128 bytes)
- Read ports 1-2: Idle
- Write ports 0-1: Idle

---

### Cycle 1: Continue Loading

**Issued Instruction:** `vle32.v v3, (a3)` (PC=2)

**Operations:**

- ✓ Memory port 0 processes v2: 0→32 bytes
- ✓ Issue v3 load instruction to memory port 1
- ✓ PC: 2 → 3

**Memory Port Status:**

- Read port 0: Occupied (v2, 32/128 bytes)
- Read port 1: Occupied (v3, 0/128 bytes)
- Read port 2: Idle

---

### Cycle 2: Parallel Loading

**Issued Instruction:** `vle32.v v4, (a4)` (PC=3)

**Operations:**

- ✓ v2 forwarding: consume 32 bytes (progress 32→64/128)
- ✓ Memory ports 0, 1, 2 working in parallel
- ✓ Issue v4 to memory port 2
- ✓ PC: 3 → 4

**Data Forwarding:**

- v2: Can forward 32 bytes (total progress 32/128)

**Memory Port Status:**

- Read port 0: Occupied (v2, 64/128 bytes)
- Read port 1: Occupied (v3, 32/128 bytes)
- Read port 2: Occupied (v4, 0/128 bytes)

---

### Cycle 3: Parallel Loading Continues

**Issued Instruction:** None (waiting for previous instructions to complete)

**Operations:**

- ✓ v2, v3, v4 continue loading and forwarding
- ✓ All 3 memory ports working
- **PC stays at 4, waiting for opportunity to issue v5**

---

### Cycle 4: Continue Waiting

**Issued Instruction:** None (waiting for previous instructions to complete)

**Operations:**

- ✓ v2, v3, v4 continue loading and forwarding
- ✓ PC: Stays at 4 (waiting for any of v2, v3, v4 to complete)

---

### Cycle 5: v2 Completes, Issue v5

**Issued Instruction:** `vle32.v v5, (a5)` (PC=4)

**Operations:**

- ✓ **v2 load completed** (after 5 cycles: Cycle 0-4)
- ✓ Memory port 0 released
- ✓ Issue v5 to port 0
- ✓ PC: 4 → 5

**Memory Port Status:**

- Read port 0: Occupied (v5, 0/128 bytes) ← Newly allocated
- Read port 1: Occupied (v3, continues execution)
- Read port 2: Occupied (v4, continues execution)

---

### Cycle 6: v3 Completes, Issue First vadd

**Issued Instruction:** `vadd.vv v1, v2, v3` (PC=5)

**Operations:**

- ✓ **v3 load completed** (after 5 cycles: Cycle 1-5)
- ✓ **Dependency check:** v2 completed, v3 just completed ✓
- ✓ **Issue to VectorAlu unit** (actual execution requires 6 cycles)
- ✓ Create write task for v1
- ✓ v4, v5 continue loading
- ✓ PC: 5 → 6

**Key Point:**

- This is the **first write** to v1

---

### Cycle 7: v4 Completes, Issue vmul

**Issued Instruction:** `vmul.vx v6, v1, a0` (PC=6)

**Operations:**

- ✓ **v4 load completed** (after 5 cycles: Cycle 2-6)
- ✓ vadd (PC=5) **still executing** (Cycle 6-11)
- ✓ v1 data begins to be available through forwarding
- ✓ **Issue vmul to VectorMul unit** (actual execution requires 9 cycles)
- ✓ v5 continues loading
- ✓ PC: 6 → 7

**Data Forwarding Key:** vmul can read v1 being written by vadd (through forwarding mechanism)

---

### Cycle 8-11: Wait for vadd and vmul to Complete

**Issued Instruction:** None

**Operations:**

- Cycle 8-11: vadd (PC=5) continues execution (Cycle 6-11, total 6 cycles)
- Cycle 8-15: vmul (PC=6) continues execution (Cycle 7-15, total 9 cycles)
- ✓ **v5 load completed** (at Cycle 10: Cycle 5-9, total 5 cycles)

---

### Cycle 12: vadd Completes, Issue Second vadd

**Issued Instruction:** `vadd.vv v1, v4, v5` (PC=7)

**Operations:**

- ✓ **First vadd (PC=5) completed** (Cycle 6-11, total 6 cycles)
- ✓ **Dependency check:** v4 completed, v5 completed ✓
- ✓ **Check WAR hazard:** v1 is being read by vmul!
  - v1 task queue: [Task 0: Completed, Task 1: Being read by vmul]
  - **Simulator allows new write task to enqueue**
- ✓ **Issue second vadd to VectorAlu unit** (actual execution requires 6 cycles)
- ✓ Create second write task for v1
- ✓ PC: 7 → 8

**Key Mechanism:** Write tasks queue up, will not overwrite data being read by vmul

---

### Cycle 13: Issue v6 Store

**Issued Instruction:** `vse32.v v6, (a6)` (PC=8)

**Operations:**

- ✓ Second vadd begins execution (Cycle 12-17)
- ✓ vmul continues execution (Cycle 7-15)
- ✓ **Issue v6 store to memory write port 0**
- ✓ PC: 8 → 9

---

### Cycle 14: Issue v1 Store

**Issued Instruction:** `vse32.v v1, (a7)` (PC=9)

**Operations:**

- ✓ **Issue v1 store to memory write port 1**
- ✓ PC: 9 → 10 (all instructions issued)
- ✓ v6 storing (Cycle 13-17)

---

### Cycle 15-17: Store and Compute in Parallel

**Operations:**

- Cycle 15: vmul completes (Cycle 7-15, total 9 cycles)
- Cycle 15-17: Second vadd continues (Cycle 12-17)
- Cycle 15-17: v6 storing
- Cycle 15-18: v1 storing

---

### Cycle 18: v6 Store Completes, vadd Completes

**Operations:**

- ✓ v6 store completed (Cycle 13-17, total 5 cycles)
- ✓ Second vadd completed (Cycle 12-17, total 6 cycles)
- ✓ v1 store continues (Cycle 14-18)

---

### Cycle 19: Store Complete

**Operations:**

- ✓ v1 store completed (128/128 bytes)
- ✓ Memory write port 1 released
- ✓ **All units idle**

---

### Cycle 20: Simulation Ends

**Status Check:**

- ✓ Fetch unit idle: true
- ✓ Functional units idle: true
- ✓ Memory unit idle: true

---

## 5. Key Mechanism Verification

### 5.1 WAR (Write After Read) Hazard Handling

**Test Scenario:**

```riscv
vadd.vv v1, v2, v3    # PC=5: Write to v1
vmul.vx v6, v1, a0    # PC=6: Read v1
vadd.vv v1, v4, v5    # PC=7: Write to v1 again (WAR!)
```

**Verification Results:**

- vmul (PC=6) successfully read the v1 value written by first vadd (PC=5)
- Second vadd (PC=7) does not overwrite v1 data being read by vmul
- v1's write tasks correctly queued: [Task 0: PC=5, Task 1: PC=7]
- After vmul completes reading v1, second vadd's result takes effect

**Conclusion:** Simulator correctly handles WAR hazards through **task queue mechanism**

---

### 5.2 Data Forwarding

**Observed Forwarding:**

1. **Load→Load Forwarding:**

   - v2, v3, v4, v5 continuously forward during loading (32 bytes/cycle)
   - Allows subsequent instructions early access to partial data
2. **Compute→Compute Forwarding:**

   - Cycle 7: vadd completes writing v1, vmul can immediately read v1
   - No need to wait for v1 to be fully written to register file

**Configuration:**

```toml
[register]
maximum_forward_bytes = 32  # Maximum forward 32 bytes per cycle
```

**Verification:**

- Forward 32 bytes per cycle (1/4 vector: 128/4=32)
- Complete 128-byte transfer in 4 cycles
- Forwarding greatly reduces RAW stalls

---

## 6. Execution Timeline Overview

```
Timeline:
Cycle 0  : VLE v2 issued
Cycle 1  : VLE v3 issued
Cycle 2  : VLE v4 issued
Cycle 3-4: Waiting for v2 to complete
Cycle 5  : VLE v5 issued, v2 completes (0-4=5 cycles)
Cycle 6  : VADD#1 issued (v1←v2+v3), v3 completes (1-5=5 cycles)
Cycle 7  : VMUL issued (v6←v1*a0), v4 completes (2-6=5 cycles)
Cycle 8-11: VADD#1 executing
Cycle 10 : v5 completes (5-9=5 cycles)
Cycle 12 : VADD#1 completes (6-11=6 cycles), VADD#2 issued (v1←v4+v5)
Cycle 13 : VSE v6 issued
Cycle 14 : VSE v1 issued
Cycle 15 : VMUL completes (7-15=9 cycles)
Cycle 17 : VADD#2 completes (12-17=6 cycles), VSE v6 completes (13-17=5 cycles)
Cycle 18 : VSE v1 completes (14-18=5 cycles)
Cycle 19 : Cleanup
Cycle 20 : Simulation ends
```

---

## 7. Detailed Cycle Table

| Cycle | Issued Instruction | Active Units     | Completed Operations                                                           | Key Events                       | Notes                   |
| ----- | ------------------ | ---------------- | ------------------------------------------------------------------------------ | -------------------------------- | ----------------------- |
| 0     | VLE v2             | Load0            | -                                                                              | v2 load begins                   | Start                   |
| 1     | VLE v3             | Load0,1          | -                                                                              | v3 load begins                   |                         |
| 2     | VLE v4             | Load0,1,2        | -                                                                              | v4 load begins                   | 3 ports fully loaded    |
| 3     | -                  | Load0,1,2        | -                                                                              | v2,v3,v4 executing               | Wait for v2 completion  |
| 4     | -                  | Load0,1,2        | **v2 completes** (5 cycles)                                              | v2 load completed                | Port 0 about to release |
| 5     | VLE v5             | Load0,1,2        | -                                                                              | v5 load begins                   | Port 0 reallocated      |
| 6     | VADD#1             | Load0,2,Alu      | **v3 completes** (5 cycles)                                              | vadd#1 begins (needs 6 cycles)   | v2,v3 ready             |
| 7     | VMUL               | Load0,2,Alu,Mul  | **v4 completes** (5 cycles)                                              | vmul begins (needs 9 cycles)     | v1 forwarded to vmul    |
| 8     | -                  | Load0,Alu,Mul    | -                                                                              | vadd#1,vmul executing            |                         |
| 9     | -                  | Load0,Alu,Mul    | **v5 completes** (5 cycles)                                              | v5 load completed                |                         |
| 10    | -                  | Alu,Mul          | -                                                                              | vadd#1,vmul executing            |                         |
| 11    | -                  | Alu,Mul          | **vadd#1 completes** (6 cycles)                                          | v1 first write completes         |                         |
| 12    | VADD#2             | Alu,Mul          | -                                                                              | vadd#2 begins (needs 6 cycles)   | v1 WAR handling         |
| 13    | VSE v6             | Alu,Mul,Store0   | -                                                                              | v6 store begins (needs 5 cycles) |                         |
| 14    | VSE v1             | Alu,Mul,Store0,1 | -                                                                              | v1 store begins (needs 5 cycles) | All instructions issued |
| 15    | -                  | Alu,Store0,1     | **vmul completes** (9 cycles)                                            | vmul completed                   |                         |
| 16    | -                  | Alu,Store0,1     | -                                                                              | vadd#2, stores executing         |                         |
| 17    | -                  | Store1           | **vadd#2 completes** (6 cycles), **v6 store completes** (5 cycles) | v1 second write completes        |                         |
| 18    | -                  | Store1           | **v1 store completes** (5 cycles)                                        | All operations complete          |                         |
| 19    | -                  | -                | -                                                                              | -                                | Cleanup                 |
| 20    | -                  | -                | -                                                                              | -                                | **End**           |

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
write_ports_limit = 2

[buffer]
input_maximum_size = 64
result_maximum_size = 64

[register]
maximum_forward_bytes = 32
```

---

### 8.2 Key Log Snippets

```
02:08:36 [INFO] Main simulation loop ended, total cycles: 20

02:08:36 [DEBUG] Vector configuration is valid: vl * sew <= vlen (32 * 32 <= 4096)

02:08:36 [DEBUG] (1) ruscv_vector_sim::sim: Function unit VectorAlu issued instruction: VADD_VV { vrd: 1, vrs1: 3, vrs2: 2 } at cycle 6

02:08:36 [DEBUG] Memory unit port status:
  Read port 0: Occupied - Instruction: VLE { vrd: 2, rs1: 12, width: 32 }, Processed: 32/128 bytes
  Read port 1: Occupied - Instruction: VLE { vrd: 3, rs1: 13, width: 32 }, Processed: 0/128 bytes
  Read port 2: Idle

02:08:36 [DEBUG] [FORWARD-INFO] Forward bytes: 32 bytes (max allowed: 32)
```

### 8.3 Gantt Chart

![1763003747497](image/test1_detailed_report/1763003747497.png)
