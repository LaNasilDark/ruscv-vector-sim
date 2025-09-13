# Supported Instructions (by now) are listed below:

| Instruction | Format         | Function                      | Destination Register | Source Operands    | Notes / Operation                                |
| :---------- | :------------- | :---------------------------- | :------------------- | :----------------- | :----------------------------------------------- |
| `ADD`     | rd, rs1, rs2   | Add                           | Scalar Register      | 2 Scalar Registers | 64-bit addition                                  |
| `ADDW`    | rd, rs1, rs2   | Add Word (32-bit)             | Scalar Register      | 2 Scalar Registers | 32-bit addition, result sign-extended to 64-bits |
| `SUB`     | rd, rs1, rs2   | Subtract                      | Scalar Register      | 2 Scalar Registers | 64-bit subtraction                               |
| `ADDI`    | rd, rs1, imm   | Add Immediate                 | Scalar Register      | 1 Scalar Register  | Add immediate value                              |
| `ADDIW`   | rd, rs1, imm   | Add Immediate Word            | Scalar Register      | 1 Scalar Register  | 32-bit add immediate, result sign-extended       |
| `XORI`    | rd, rs1, imm   | Exclusive-OR Immediate        | Scalar Register      | 1 Scalar Register  | Bitwise XOR with immediate                       |
| `SLLI`    | rd, rs1, shamt | Shift Left Logical Immediate  | Scalar Register      | 1 Scalar Register  | Logical left shift                               |
| `SRLI`    | rd, rs1, shamt | Shift Right Logical Immediate | Scalar Register      | 1 Scalar Register  | Logical right shift                              |

| Instruction         | Format          | Function                      | Functional Unit | Destination Register | Source Operands                   | Notes / Operation                           |
| :------------------ | :-------------- | :---------------------------- | :-------------- | :------------------- | :-------------------------------- | :------------------------------------------ |
| `VFADD.VV`        | vrd, vrs1, vrs2 | Vector FP Add                 | VectorAlu       | Vector Register      | 2 Vector Registers                | Vector-vector floating-point addition       |
| `VFMUL.VV`        | vrd, vrs1, vrs2 | Vector FP Multiply            | VectorMul       | Vector Register      | 2 Vector Registers                | Vector-vector floating-point multiplication |
| `VFMACC.VV`       | vrd, vrs1, vrs2 | Vector FP Multiply-Accumulate | VectorMacc      | Vector Register      | 3 Vector Registers (incl. dest.)  | vrd = vrd + (vrs1 * vrs2)                   |
| `VFSLIDE1DOWN.VF` | vrd, frs1, vrs2 | Vector Slide 1 Down           | VectorSlide     | Vector Register      | FP Register + 2 Vector Registers* | Shift vector elements down by one           |
| `VFSLIDE1UP.VF`   | vrd, frs1, vrs2 | Vector Slide 1 Up             | VectorSlide     | Vector Register      | FP Register + 2 Vector Registers* | Shift vector elements up by one             |

| Instruction | Format            | Function                   | Access Direction | Destination/Source Register   | Address Register | Notes                        |
| :---------- | :---------------- | :------------------------- | :--------------- | :---------------------------- | :--------------- | :--------------------------- |
| `LD`      | rd, offset(rs1)   | Load Doubleword (64-bit)   | Read             | Scalar Register (Destination) | Scalar Register  | Load 64-bit data from memory |
| `SD`      | rs2, offset(rs1)  | Store Doubleword (64-bit)  | Write            | Scalar Register (Source)      | Scalar Register  | Store 64-bit data to memory  |
| `FLD`     | frd, offset(rs1)  | Floating-point Load Double | Read             | FP Register (Destination)     | Scalar Register  | Load floating-point data     |
| `VLE`     | vrd, (rs1), width | Vector Load Element        | Read             | Vector Register (Destination) | Scalar Register  | Load vector data from memory |
| `VSE`     | vrs, (rs1), width | Vector Store Element       | Write            | Vector Register (Source)      | Scalar Register  | Store vector data to memory  |

| Inst         | Note that these instructions are ignored by simulator |
| ------------ | ----------------------------------------------------- |
| `VSETVL`   | Ignored                                               |
| `VSETIVLI` | Ignored                                               |
| `VSETVLI`  | Ignored                                               |
