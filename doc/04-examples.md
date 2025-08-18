
# Example Analysis

## Matrix Multiply

### Code Sequence

```sh
   1021a: 33 8f c7 41  	sub	t5, a5, t3
   1021e: 57 7f 8f 0d  	vsetvli	t5, t5, e64, m1, ta, ma
   10222: 93 1f 3e 00  	slli	t6, t3, 0x3
   10226: 33 04 f3 01  	add	s0, t1, t6
   1022a: 07 75 04 02  	vle64.v	v10, (s0)
   1022e: f6 9f        	add	t6, t6, t4
   10230: 87 f5 0f 02  	vle64.v	v11, (t6)
   10234: 7a 9e        	add	t3, t3, t5
   10236: d7 94 a5 b2  	vfmacc.vv	v9, v11, v10
   1023a: e3 60 fe fe  	bltu	t3, a5, 0x1021a <mat
```
### Running Command

```sh
cargo run -- -i appendix/_matmul/bin/matmul_vector.exe -c ./config.toml -s 0x1021a -e 0x1023a
```

### Configure

```toml
[function_units.interger_alu]
latency = 1 # The arithmetic logic unit usually has low latency.

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

[memory_units.load_store_unit]
latency = 2 
max_access_width = 32 #bytes
read_ports_limit = 3 # This value indicates how many values can be read in parallel
write_ports_limit = 2 # This value indicates how many values can be write in parallel

[vector_config.software]
vl = 32 # Vector Length, vector length register value that determines the number of elements for current vector operations
sew = 32   # Scalar Element Width, scalar element width (in bits), supports 8/16/32/64 bits
lmul = 1   # Lane Multiplier, lane multiplier that defines the width multiplier of vector register groups

[vector_config.hardware]
vlen = 4096 # (in bits) vlen must be greater than or equal sew * vl
lane_number = 4

[vector_register.ports]
read_ports_limit = 2 # if you want unlimited ports number, please change this number large enough, like 64 or some number else. Number of simultaneous read ports for one vector register.
write_ports_limit = 2 # I believe that a write port is absolutely sufficient when the compiler allocates registers reasonably. However, if some odd instruction streams are constructed, increasing this value can also be considered. Number of simultaneous write ports for one vector register


[buffer]
input_maximum_size = 64 # in bytes
result_maximum_size = 64 # in bytes

[register]
maximum_forward_bytes = 32 # in bytes
```



### Running Result

```sh
[INFO] Main simulation loop ended, total cycles: 26
```

### Result Analysis

#### Theoretical Analysis: Why 26 Cycles?

The simulation resulted in 26 cycles due to the combination of scalar instruction execution, vector loading operations, and vector computation with hardware limitations. Here's the detailed breakdown:

##### Key Configuration Parameters
- **Integer ALU latency**: 1 cycle
- **Memory load latency**: 2 cycles (due to buffer system)
- **Vector register ports**: 2 read, 2 write per register
- **Vector length (VL)**: 32 elements
- **Element width (SEW)**: 64 bits (8 bytes per element)
- **Vector register size**: 128 bytes (can hold 16 elements of 64-bit data)
- **Maximum forward bytes**: 32 bytes (4 elements)

##### Execution Phase Analysis

**Phase 1: Scalar Instructions and Vector Loading (Cycles 0-9)**
1. **SUB instruction** (cycle 0): Calculates remaining elements, completes in 1 cycle
2. **VSETVLI instruction** Ignored
3. **SLLI instruction** (cycle 1): Shifts for address calculation, completes in 1 cycle
4. **ADD instruction** (cycle 3): Pending in cycle 2 because operand has unfinished writing. Calculates first load address, completes in 1 cycle
5. **First VLE64.V** (cycles 5): Loads vector v10, pending in cycle 4 because the address register is not ready
6. **ADD instruction** (cycle 6): Updates address for second load, completes in 1 cycle
7. **Second VLE64.V** (cycles 8): Loads vector v11, pending in cycle 7 because the address register is not ready
8. **ADD instruction** (cycle 9): Updates loop counter, completes in 1 cycle
9. **VFMACC.VV** (cycle 10): Issue in cycle 10

**Phase 2: Vector Multiply-Accumulate Operation (Cycles 10-26)**

Here, since the bandwidth of the memory is greater than dl (lane_number * sew), the loaded data is always sufficient. The ratio of dl to vl is 1:8, so a total of 8 computing events will be generated.

In the cycle 11, the first event is generated. And the last event is generated in cycle 18.

In the config file, the latency of "float_multiplier" is 6. The last event needs 6 cycles to complete. So the total cycle of calculation is 18 + 6 = 24.

In the cycle 25, the newly calculated result is written to the register. The simulation ends.

Therefore, the simulator simulates a total of 26 cycles (here, counting starts from 0, and there are 26 cycles from cycle 0 to cycle 25).


## Read Ports Conflict

### Code Sequence

```sh
   1023c: 07 74 05 02  	vle64.v	v8, (a0)
   10240: 87 f4 05 02  	vle64.v	v9, (a1)
   10244: d7 94 84 92  	vfmul.vv	v9, v8, v9
   10248: 57 94 84 02  	vfadd.vv	v8, v8, v9
   1024c: 27 f4 06 02  	vse64.v	v8, (a3)
```

### Running Command

```sh
cargo run -- -i appendix/_conflict_port/bin/conflict_port.exe -c ./config.toml -s 0x1023c -e 0x10250
```

### Configure

Same as the matmul example. But change the "read_ports_limit" under [vector_register.ports] to 1

```toml
[vector_register.ports]
read_ports_limit = 1
```

### Result

09:46:39 [INFO] Main simulation loop ended, total cycles: 23

### Result Analysis

#### Issue Stage

**vle64.v	v8, (a0)** instruction is issued in cycle 0
**vle64.v	v9, (a1)** instruction is issued in cycle 1
**vfmul.vv	v9, v8, v9** instruction is issued in cycle 2

But **vfadd.vv	v8, v8, v9** instruction is issued in cycle 6. Because the read ports of v8 and v9 are both occupied by the previous instruction. We need 4 cycles(forwarding bytes from register to input buffer is 32 bytes. 8 elements in total) to read vector to the input buffer

**vse64.v	v8, (a3)** instruction is issued in cycle 7

#### Execution Stage

The first event of **vfmul.vv	v9, v8, v9** is generated in cycle 3. The last event of **vfmul.vv	v9, v8, v9** is generated in cycle 10. And the last event ends in cycle 16.

The last event of **vfadd.vv	v8, v8, v9** is generated in cycle 17, which needs 3 cycles to execute. And the result is written to result buffer in cycle 20.

In the cycle 21, the last part of v8 is forwarding to the input buffer of the memory ports. And the memory ports write the data to the result buffer.

In the cycle 22. The last part of result buffer of memory write port is consumed. And the simulation comes to the end.

