# RISC-V Vector Simulator Test Report

## Test 2: Register Overwrite & WAR (Write After Read)

**Simulator Version:** ruscv-vector-sim
**Configuration File:** config.toml
**Test Address Range:** 0x28 - 0x44

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

**Note:** Actual execution latency includes data transfer and forwarding overhead.

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
vle32.v v2, (a1)                    # Load vector v2

# PC=2  
vle32.v v1, (a0)                    # Load vector v1

# PC=3
vadd.vv v3, v1, v2                  # v3 = v1 + v2 (Read v1)

# PC=4
vle32.v v1, (a2)                    # Reload v1 (Overwrite! WAR hazard)

# PC=5
vmul.vv v4, v1, v3                  # v4 = v1 * v3 (Read new v1)

# PC=6
vse32.v v4, (a3)                    # Store v4
```

### 3.2 Key Dependencies

**WAR (Write After Read) Hazard:**

```
PC=2: vle32.v v1, (a0)      → First write to v1
PC=3: vadd.vv v3, v1, v2    → Read v1 (must read first loaded value)
PC=4: vle32.v v1, (a2)      → Write to v1 again (WAR: cannot overwrite before PC=3 completes read)
PC=5: vmul.vv v4, v1, v3    → Read v1 (must read second loaded value)
```

**RAW (Read After Write) Dependencies:**

```
PC=1: vle32.v v2, (a1)      → Write to v2
PC=3: vadd.vv v3, v1, v2    → Read v2

PC=2: vle32.v v1, (a0)      → Write to v1  
PC=3: vadd.vv v3, v1, v2    → Read v1

PC=3: vadd.vv v3, v1, v2    → Write to v3
PC=5: vmul.vv v4, v1, v3    → Read v3

PC=4: vle32.v v1, (a2)      → Write to v1
PC=5: vmul.vv v4, v1, v3    → Read v1
```

---

## 4. Cycle-by-Cycle Execution Analysis

### Cycle 0: Begin Execution

**Issued Instruction:** `vle32.v v2, (a1)` (PC=1)

**Operations:**

- ✓ Skip vsetivli (configuration instruction, does not occupy execution unit)
- ✓ Issue v2 load instruction to memory port 0
- ✓ Create write task for v2
- ✓ PC: 1 → 2

**Memory Port Status:**

- Read port 0: Occupied (v2, 0/128 bytes)
- Read ports 1-2: Idle

---

### Cycle 1: Issue v1 First Load

**Issued Instruction:** `vle32.v v1, (a0)` (PC=2)

**Operations:**

- ✓ v2 loading (0→32 bytes)
- ✓ Issue v1 load instruction to memory port 1
- ✓ Create first write task for v1
- ✓ PC: 2 → 3

**Memory Port Status:**

- Read port 0: Occupied (v2, 32/128 bytes)
- Read port 1: Occupied (v1, 0/128 bytes)

---

### Cycle 2: Issue vadd

**Issued Instruction:** `vadd.vv v3, v1, v2` (PC=3)

**Operations:**

- ✓ v2 forwarding (32→64 bytes)
- ✓ v1 forwarding (0→32 bytes)
- ✓ **Dependency check:** v1 and v2 data begin to be available through forwarding ✓
- ✓ **Issue to VectorAlu unit** (actual execution requires 6 cycles)
- ✓ Create write task for v3
- ✓ PC: 3 → 4

**Data Forwarding:** vadd can read v1 and v2 through forwarding mechanism

**Memory Port Status:**

- Read port 0: Occupied (v2, 64/128 bytes)
- Read port 1: Occupied (v1, 32/128 bytes)

---

### Cycle 3: Issue v1 Second Load

**Issued Instruction:** `vle32.v v1, (a2)` (PC=4)

**Operations:**

- ✓ v2 forwarding (64→96 bytes)
- ✓ v1 forwarding (32→64 bytes)
- ✓ vadd executing (Cycle 2-7)
- ✓ **Check WAR hazard:** v1 is being read by vadd!
  - v1 task queue: [Task 0: Being read by vadd]
  - **Simulator allows new load task to enqueue**
- ✓ Issue v1 second load to memory port 2
- ✓ Create second write task for v1
- ✓ PC: 4 → 5

**Key Mechanism:** Second v1 load task queues up, will not overwrite data being read; `vector_register.ports` defines 2 write ports.

**Memory Port Status:**

- Read port 0: Occupied (v2, 96/128 bytes)
- Read port 1: Occupied (v1_1st, 64/128 bytes)
- Read port 2: Occupied (v1_2nd, 0/128 bytes)

---

### Cycle 4: Issue vmul

**Issued Instruction:** `vmul.vv v4, v1, v3` (PC=5)

**Operations:**

- ✓ v2 load completed (Cycle 0-4, total 5 cycles)
- ✓ v1 first load continues
- ✓ v1 second load continues
- ✓ vadd continues execution (Cycle 2-7)
- ✓ **Dependency check:** v1 and v3 data available through forwarding ✓
- ✓ **Issue vmul to VectorMul unit** (actual execution requires 8 cycles)
- ✓ Create write task for v4
- ✓ PC: 5 → 6

**Data Forwarding Key:** vmul reads v1 (second load) and v3 (vadd result) through forwarding

---

### Cycle 5: Issue v4 Store

**Issued Instruction:** `vse32.v v4, (a3)` (PC=6)

**Operations:**

- ✓ **v1 first load completed** (Cycle 1-5, total 5 cycles)
- ✓ v1 second load continues
- ✓ vadd continues execution (Cycle 2-7)
- ✓ vmul executing (Cycle 4-11)
- ✓ **Issue v4 store to memory write port 0** (actual execution requires 8 cycles)
- ✓ PC: 6 → 7 (all instructions issued)

---

### Cycle 6-7: Wait for vadd and v1 Second Load to Complete

**Issued Instruction:** None

**Operations:**

- Cycle 6-7: vadd continues execution
- Cycle 6-7: v1 second load continues
- Cycle 6-11: vmul continues execution
- Cycle 6-12: v4 store continues
- ✓ **v1 second load completed** (Cycle 3-7, total 5 cycles)
- ✓ **vadd completed** (Cycle 2-7, total 6 cycles)

---

### Cycle 8-11: Wait for vmul to Complete

**Issued Instruction:** None

**Operations:**

- Cycle 8-11: vmul continues execution (Cycle 4-11, total 8 cycles)
- Cycle 8-12: v4 store continues

---

### Cycle 12: vmul and Store Complete

**Operations:**

- ✓ **vmul completed** (Cycle 4-11, total 8 cycles)
- ✓ **v4 store completed** (Cycle 5-12, total 8 cycles)

---

### Cycle 13: Cleanup

**Operations:**

- ✓ All instructions executed
- ✓ All units idle

---

### Cycle 14: Simulation Ends

**Status Check:**

- ✓ Fetch unit idle: true
- ✓ Functional units idle: true
- ✓ Memory unit idle: true
- ✓ All operations complete

---

## 5. Key Mechanism Verification

### 5.1 WAR (Write After Read) Hazard Handling

**Test Scenario:**

```riscv
vle32.v v1, (a0)       # PC=2: First write to v1
vadd.vv v3, v1, v2     # PC=3: Read v1 (first value)
vle32.v v1, (a2)       # PC=4: Write to v1 again (WAR!)
vmul.vv v4, v1, v3     # PC=5: Read v1 (second value)
```

**Verification Results:**

- vadd (PC=3, issued at Cycle 2) reads first loaded v1 value (from a0)
- Second v1 load (PC=4, issued at Cycle 3) does not overwrite data being read by vadd
- v1's write tasks correctly queued: [Task 0: Load from a0, Task 1: Load from a2]
- vmul (PC=5, issued at Cycle 4) reads second loaded v1 value (from a2) through forwarding
- vmul can issue at Cycle 4 because it accesses v1#2 and v3 simultaneously through forwarding

**Conclusion:** Simulator correctly handles register overwrite WAR hazards through **task queue mechanism and data forwarding**

---

### 5.2 Data Forwarding

**Observed Forwarding:**

1. **Load→Compute Forwarding:**

   - Cycle 2: vadd begins execution before v1 and v2 fully load
   - Data provided to vadd through forwarding mechanism (32 bytes/cycle)
2. **Compute→Store Forwarding:**

   - Cycle 5: v4 store begins before vmul fully completes (vmul issued at Cycle 4)
   - vmul results provided to store unit through forwarding mechanism
3. **Cross-Task Forwarding (Key):**

   - Cycle 4: vmul can simultaneously read v1#2 (still loading) and v3 (vadd still computing) through forwarding
   - This demonstrates forwarding mechanism allows multi-level pipeline overlap

**Configuration:**

```toml
[register]
maximum_forward_bytes = 32  # Maximum forward 32 bytes per cycle
```

**Verification:**

- Forward 32 bytes per cycle (1/4 vector: 128/4=32)
- Forwarding mechanism allows compute and load to overlap
- Forwarding greatly reduces RAW stalls

---

## 6. Execution Timeline Overview

```
Timeline (Test 2 - Corrected Version):
Cycle 0  : VLE v2 issued (from a1)
Cycle 1  : VLE v1 issued (from a0, 1st time)
Cycle 2  : VADD issued (v3←v1+v2)
Cycle 3  : VLE v1 issued (from a2, 2nd time) - WAR hazard!
Cycle 4  : VMUL issued (v4←v1*v3), v2 completes (0-4=5 cycles)
Cycle 5  : VSE v4 issued, v1 1st completes (1-5=5 cycles)
Cycle 6-7: VADD, VMUL, VSE executing in parallel
Cycle 7  : VADD completes (2-7=6 cycles), v1 2nd completes (3-7=5 cycles)
Cycle 8-11: VMUL and VSE continue execution
Cycle 11 : VMUL completes (4-11=8 cycles)
Cycle 12 : VSE completes (5-12=8 cycles)
Cycle 13 : Cleanup
Cycle 14 : Simulation ends
```

---

## 7. Detailed Cycle Table

| Cycle | Issued Instruction | Active Units      | Completed Operations                                                     | Key Events                       | Notes                     |
| ----- | ------------------ | ----------------- | ------------------------------------------------------------------------ | -------------------------------- | ------------------------- |
| 0     | VLE v2             | Load0             | -                                                                        | v2 load begins (from a1)         | Start                     |
| 1     | VLE v1(#1)         | Load0,1           | -                                                                        | v1 1st load (from a0)            |                           |
| 2     | VADD               | Load0,1,Alu       | -                                                                        | vadd begins (needs 6 cycles)     | v1,v2 forwarded to vadd   |
| 3     | VLE v1(#2)         | Load0,1,2,Alu     | -                                                                        | v1 2nd load (from a2)            | v1 WAR handling           |
| 4     | VMUL               | Load1,2,Alu,Mul   | **v2 completes** (5 cycles)                                        | vmul begins (needs 8 cycles)     | v1,v3 forwarded to vmul   |
| 5     | VSE v4             | Load2,Alu,Mul,St0 | **v1#1 completes** (5 cycles)                                      | v4 store begins (needs 8 cycles) | All instructions issued   |
| 6     | -                  | Load2,Alu,Mul,St0 | -                                                                        | Multiple units parallel          |                           |
| 7     | -                  | Mul,St0           | **vadd completes** (6 cycles), **v1#2 completes** (5 cycles) | v3 and v1#2 ready                |                           |
| 8     | -                  | Mul,St0           | -                                                                        | vmul and store parallel          |                           |
| 9     | -                  | Mul,St0           | -                                                                        | vmul and store parallel          |                           |
| 10    | -                  | Mul,St0           | -                                                                        | vmul and store parallel          |                           |
| 11    | -                  | Mul,St0           | **vmul completes** (8 cycles)                                      | v4 ready                         |                           |
| 12    | -                  | St0               | **VSE completes** (8 cycles)                                       | v4 store complete                |                           |
| 13    | -                  | -                 | -                                                                        | Cleanup                          |                           |
| 14    | -                  | -                 | -                                                                        | -                                | **Simulation ends** |

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
04:27:24 [INFO] Main simulation loop ended, total cycles: 14

04:27:24 [DEBUG] Vector configuration is valid: vl * sew <= vlen (32 * 32 <= 4096)

04:27:24 [DEBUG] Memory unit issued instruction: VLE { vrd: 2, rs1: 11, width: 32 } at cycle 0

04:27:24 [DEBUG] Memory unit issued instruction: VLE { vrd: 1, rs1: 10, width: 32 } at cycle 1

04:27:24 [DEBUG] Function unit VectorAlu issued instruction: VADD_VV { vrd: 3, vrs1: 2, vrs2: 1 } at cycle 2

04:27:24 [DEBUG] Memory unit issued instruction: VLE { vrd: 1, rs1: 12, width: 32 } at cycle 3
  (WAR conflict: v1 task queue manages first and second write)

04:27:24 [DEBUG] Function unit VectorMul issued instruction: VMUL_VV { vrd: 4, vrs1: 3, vrs2: 1 } at cycle 4

04:27:24 [DEBUG] Memory unit issued instruction: VSE { vrd: 4, rs1: 13, width: 32 } at cycle 5

04:27:24 [DEBUG] [FORWARD-INFO] Forward bytes: 32 bytes (max allowed: 32)
```

### 8.3 Gantt Chart

![1763009755854](image/test2_detailed_report/1763009755854.png)

If we change writeport number to 1:
![1763021170935](image/test2_detailed_report/1763021170935.png)
