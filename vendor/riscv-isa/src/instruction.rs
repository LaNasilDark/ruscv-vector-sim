// Copyright James Wainwright
//
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

/// RISC-V instruction.
///
/// Contains all supported canonical RISC-V instructions. Does not
/// contain pseudo or compressed instructions.
///
/// Instruction arguments are all in their decoded forms, meaning correctly
/// scaled and sign extended. Their names are as they appear in specifications
/// (for the most part), and include:
///
/// * `rd`, `rs1`, `rs2`: destination and source registers.
/// * `frd`, `frs1`, `frs2`, `frs3`: destination and source floating point registers.
/// * `offset`, `imm`, `shamt`: numerical offsets, immediates, and shift amounts.
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
    // Unknown:
    UNIMP,
    // RV32I base instruction set:
    LUI { rd: u32, imm: u32 },
    AUIPC { rd: u32, imm: u32 },
    JAL { rd: u32, offset: i32 },
    JALR { rd: u32, rs1: u32, offset: i32 },
    BEQ { rs1: u32, rs2: u32, offset: i32 },
    BNE { rs1: u32, rs2: u32, offset: i32 },
    BLT { rs1: u32, rs2: u32, offset: i32 },
    BGE { rs1: u32, rs2: u32, offset: i32 },
    BLTU { rs1: u32, rs2: u32, offset: i32 },
    BGEU { rs1: u32, rs2: u32, offset: i32 },
    LB { rd: u32, rs1: u32, offset: i32 },
    LH { rd: u32, rs1: u32, offset: i32 },
    LW { rd: u32, rs1: u32, offset: i32 },
    LBU { rd: u32, rs1: u32, offset: i32 },
    LHU { rd: u32, rs1: u32, offset: i32 },
    SB { rs1: u32, rs2: u32, offset: i32 },
    SH { rs1: u32, rs2: u32, offset: i32 },
    SW { rs1: u32, rs2: u32, offset: i32 },
    ADDI { rd: u32, rs1: u32, imm: i32 },
    SLTI { rd: u32, rs1: u32, imm: i32 },
    SLTIU { rd: u32, rs1: u32, imm: i32 },
    XORI { rd: u32, rs1: u32, imm: i32 },
    ORI { rd: u32, rs1: u32, imm: i32 },
    ANDI { rd: u32, rs1: u32, imm: i32 },
    SLLI { rd: u32, rs1: u32, shamt: u32 },
    SRLI { rd: u32, rs1: u32, shamt: u32 },
    SRAI { rd: u32, rs1: u32, shamt: u32 },
    ADD { rd: u32, rs1: u32, rs2: u32 },
    SUB { rd: u32, rs1: u32, rs2: u32 },
    SLL { rd: u32, rs1: u32, rs2: u32 },
    SLT { rd: u32, rs1: u32, rs2: u32 },
    SLTU { rd: u32, rs1: u32, rs2: u32 },
    XOR { rd: u32, rs1: u32, rs2: u32 },
    SRL { rd: u32, rs1: u32, rs2: u32 },
    SRA { rd: u32, rs1: u32, rs2: u32 },
    OR { rd: u32, rs1: u32, rs2: u32 },
    AND { rd: u32, rs1: u32, rs2: u32 },
    FENCE { pred: Iorw, succ: Iorw },
    ECALL,
    EBREAK,
    // S-mode:
    SRET,
    SFENCE_VMA { rs1: u32, rs2: u32 },
    // Privileged:
    MRET,
    WFI,
    // RV64I base instruction set:
    LWU { rd: u32, rs1: u32, offset: i32 },
    LD { rd: u32, rs1: u32, offset: i32 },
    SD { rs1: u32, rs2: u32, offset: i32 },
    ADDIW { rd: u32, rs1: u32, imm: i32 },
    SLLIW { rd: u32, rs1: u32, shamt: u32 },
    SRLIW { rd: u32, rs1: u32, shamt: u32 },
    SRAIW { rd: u32, rs1: u32, shamt: u32 },
    ADDW { rd: u32, rs1: u32, rs2: u32 },
    SUBW { rd: u32, rs1: u32, rs2: u32 },
    SLLW { rd: u32, rs1: u32, rs2: u32 },
    SRLW { rd: u32, rs1: u32, rs2: u32 },
    SRAW { rd: u32, rs1: u32, rs2: u32 },
    // RV32/RV64 Zifencei:
    FENCE_I,
    // RV32/RV64 Zicsr extension:
    CSRRW { rd: u32, csr: u32, rs1: u32 },
    CSRRS { rd: u32, csr: u32, rs1: u32 },
    CSRRC { rd: u32, csr: u32, rs1: u32 },
    CSRRWI { rd: u32, csr: u32, uimm: u32 },
    CSRRSI { rd: u32, csr: u32, uimm: u32 },
    CSRRCI { rd: u32, csr: u32, uimm: u32 },
    // RV32M extension:
    MUL { rd: u32, rs1: u32, rs2: u32 },
    MULH { rd: u32, rs1: u32, rs2: u32 },
    MULHSU { rd: u32, rs1: u32, rs2: u32 },
    MULHU { rd: u32, rs1: u32, rs2: u32 },
    DIV { rd: u32, rs1: u32, rs2: u32 },
    DIVU { rd: u32, rs1: u32, rs2: u32 },
    REM { rd: u32, rs1: u32, rs2: u32 },
    REMU { rd: u32, rs1: u32, rs2: u32 },
    // RV64M extension:
    MULW { rd: u32, rs1: u32, rs2: u32 },
    DIVW { rd: u32, rs1: u32, rs2: u32 },
    DIVUW { rd: u32, rs1: u32, rs2: u32 },
    REMW { rd: u32, rs1: u32, rs2: u32 },
    REMUW { rd: u32, rs1: u32, rs2: u32 },
    // RV32A extension:
    LR_W { rd: u32, rs1: u32, rl: u32, aq: u32 },
    SC_W { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOSWAP_W { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOADD_W { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOXOR_W { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOAND_W { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOOR_W { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOMIN_W { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOMAX_W { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOMINU_W { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOMAXU_W { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    // RV64A extension:
    LR_D { rd: u32, rs1: u32, rl: u32, aq: u32 },
    SC_D { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOSWAP_D { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOADD_D { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOXOR_D { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOAND_D { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOOR_D { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOMIN_D { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOMAX_D { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOMINU_D { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    AMOMAXU_D { rd: u32, rs1: u32, rs2: u32, rl: u32, aq: u32 },
    // RV32F extension:
    FLW { frd: u32, rs1: u32, offset: i32 },
    FSW { rs1: u32, frs2: u32, offset: i32 },
    FMADD_S { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FMSUB_S { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FNMSUB_S { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FNMADD_S { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FADD_S { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FSUB_S { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FMUL_S { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FDIV_S { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FSQRT_S { frd: u32, rm: u32, frs1: u32 },
    FSGNJ_S { frd: u32, frs1: u32, frs2: u32 },
    FSGNJN_S { frd: u32, frs1: u32, frs2: u32 },
    FSGNJX_S { frd: u32, frs1: u32, frs2: u32 },
    FMIN_S { frd: u32, frs1: u32, frs2: u32 },
    FMAX_S { frd: u32, frs1: u32, frs2: u32 },
    FCVT_W_S { rd: u32, rm: u32, frs1: u32 },
    FCVT_WU_S { rd: u32, rm: u32, frs1: u32 },
    FMV_X_W { rd: u32, frs1: u32 },
    FEQ_S { rd: u32, frs1: u32, frs2: u32 },
    FLT_S { rd: u32, frs1: u32, frs2: u32 },
    FLE_S { rd: u32, frs1: u32, frs2: u32 },
    FCLASS_S { rd: u32, frs1: u32 },
    FCVT_S_W { frd: u32, rm: u32, rs1: u32 },
    FCVT_S_WU { frd: u32, rm: u32, rs1: u32 },
    FMV_W_X { frd: u32, rs1: u32 },
    // RV64F extension:
    FCVT_L_S { rd: u32, rm: u32, frs1: u32 },
    FCVT_LU_S { rd: u32, rm: u32, frs1: u32 },
    FCVT_S_L { frd: u32, rm: u32, rs1: u32 },
    FCVT_S_LU { frd: u32, rm: u32, rs1: u32 },
    // RV32D extension:
    FLD { frd: u32, rs1: u32, offset: i32 },
    FSD { rs1: u32, frs2: u32, offset: i32 },
    FMADD_D { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FMSUB_D { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FNMSUB_D { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FNMADD_D { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FADD_D { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FSUB_D { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FMUL_D { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FDIV_D { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FSQRT_D { frd: u32, rm: u32, frs1: u32 },
    FSGNJ_D { frd: u32, frs1: u32, frs2: u32 },
    FSGNJN_D { frd: u32, frs1: u32, frs2: u32 },
    FSGNJX_D { frd: u32, frs1: u32, frs2: u32 },
    FMIN_D { frd: u32, frs1: u32, frs2: u32 },
    FMAX_D { frd: u32, frs1: u32, frs2: u32 },
    FCVT_S_D { frd: u32, rm: u32, frs1: u32 },
    FCVT_D_S { frd: u32, rm: u32, frs1: u32 },
    FEQ_D { rd: u32, frs1: u32, frs2: u32 },
    FLT_D { rd: u32, frs1: u32, frs2: u32 },
    FLE_D { rd: u32, frs1: u32, frs2: u32 },
    FCLASS_D { rd: u32, frs1: u32 },
    FCVT_W_D { rd: u32, rm: u32, frs1: u32 },
    FCVT_WU_D { rd: u32, rm: u32, frs1: u32 },
    FCVT_D_W { frd: u32, rm: u32, rs1: u32 },
    FCVT_D_WU { frd: u32, rm: u32, rs1: u32 },
    // RV64D extension:
    FCVT_L_D { rd: u32, rm: u32, frs1: u32 },
    FCVT_LU_D { rd: u32, rm: u32, frs1: u32 },
    FMV_X_D { rd: u32, frs1: u32 },
    FCVT_D_L { frd: u32, rm: u32, rs1: u32 },
    FCVT_D_LU { frd: u32, rm: u32, rs1: u32 },
    FMV_D_X { frd: u32, rs1: u32 },
    // RV32Q extension:
    FLQ { frd: u32, rs1: u32, offset: i32 },
    FSQ { rs1: u32, frs2: u32, offset: i32 },
    FMADD_Q { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FMSUB_Q { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FNMSUB_Q { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FNMADD_Q { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FADD_Q { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FSUB_Q { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FMUL_Q { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FDIV_Q { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FSQRT_Q { frd: u32, rm: u32, frs1: u32 },
    FSGNJ_Q { frd: u32, frs1: u32, frs2: u32 },
    FSGNJN_Q { frd: u32, frs1: u32, frs2: u32 },
    FSGNJX_Q { frd: u32, frs1: u32, frs2: u32 },
    FMIN_Q { frd: u32, frs1: u32, frs2: u32 },
    FMAX_Q { frd: u32, frs1: u32, frs2: u32 },
    FCVT_S_Q { frd: u32, rm: u32, frs1: u32 },
    FCVT_Q_S { frd: u32, rm: u32, frs1: u32 },
    FCVT_D_Q { frd: u32, rm: u32, frs1: u32 },
    FCVT_Q_D { frd: u32, rm: u32, frs1: u32 },
    FEQ_Q { rd: u32, frs1: u32, frs2: u32 },
    FLT_Q { rd: u32, frs1: u32, frs2: u32 },
    FLE_Q { rd: u32, frs1: u32, frs2: u32 },
    FCLASS_Q { rd: u32, frs1: u32 },
    FCVT_W_Q { rd: u32, rm: u32, frs1: u32 },
    FCVT_WU_Q { rd: u32, rm: u32, frs1: u32 },
    FCVT_Q_W { frd: u32, rm: u32, rs1: u32 },
    FCVT_Q_WU { frd: u32, rm: u32, rs1: u32 },
    // RV64Q extension:
    FCVT_L_Q { rd: u32, rm: u32, frs1: u32 },
    FCVT_LU_Q { rd: u32, rm: u32, frs1: u32 },
    FCVT_Q_L { frd: u32, rm: u32, rs1: u32 },
    FCVT_Q_LU { frd: u32, rm: u32, rs1: u32 },
    // RV32Zfh extension:
    FLH { frd: u32, rs1: u32, offset: i32 },
    FSH { rs1: u32, frs2: u32, offset: i32 },
    FMADD_H { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FMSUB_H { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FNMSUB_H { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FNMADD_H { frd: u32, rm: u32, frs1: u32, frs2: u32, frs3: u32 },
    FADD_H { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FSUB_H { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FMUL_H { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FDIV_H { frd: u32, rm: u32, frs1: u32, frs2: u32 },
    FSQRT_H { frd: u32, rm: u32, frs1: u32 },
    FSGNJ_H { frd: u32, frs1: u32, frs2: u32 },
    FSGNJN_H { frd: u32, frs1: u32, frs2: u32 },
    FSGNJX_H { frd: u32, frs1: u32, frs2: u32 },
    FMIN_H { frd: u32, frs1: u32, frs2: u32 },
    FMAX_H { frd: u32, frs1: u32, frs2: u32 },
    FCVT_S_H { frd: u32, rm: u32, frs1: u32 },
    FCVT_H_S { frd: u32, rm: u32, frs1: u32 },
    FCVT_D_H { frd: u32, rm: u32, frs1: u32 },
    FCVT_H_D { frd: u32, rm: u32, frs1: u32 },
    FCVT_Q_H { frd: u32, rm: u32, frs1: u32 },
    FCVT_H_Q { frd: u32, rm: u32, frs1: u32 },
    FEQ_H { rd: u32, frs1: u32, frs2: u32 },
    FLT_H { rd: u32, frs1: u32, frs2: u32 },
    FLE_H { rd: u32, frs1: u32, frs2: u32 },
    FCLASS_H { rd: u32, frs1: u32 },
    FCVT_W_H { rd: u32, rm: u32, frs1: u32 },
    FCVT_WU_H { rd: u32, rm: u32, frs1: u32 },
    FMV_X_H { frd: u32, rs1: u32 },
    FCVT_H_W { frd: u32, rm: u32, rs1: u32 },
    FCVT_H_WU { frd: u32, rm: u32, rs1: u32 },
    FMV_H_X { frd: u32, rs1: u32 },
    // RV64Zfh extension:
    FCVT_L_H { rd: u32, rm: u32, frs1: u32 },
    FCVT_LU_H { rd: u32, rm: u32, frs1: u32 },
    FCVT_H_L { frd: u32, rm: u32, rs1: u32 },
    FCVT_H_LU { frd: u32, rm: u32, rs1: u32 },
    // Zawrs extension:
    WRS_NTO,
    WRS_STO,
    // RV32Zba extension:
    SH1ADD { rd: u32, rs1: u32, rs2: u32 },
    SH2ADD { rd: u32, rs1: u32, rs2: u32 },
    SH3ADD { rd: u32, rs1: u32, rs2: u32 },
    // RV64Zba extension:
    ADD_UW { rd: u32, rs1: u32, rs2: u32 },
    SH1ADD_UW { rd: u32, rs1: u32, rs2: u32 },
    SH2ADD_UW { rd: u32, rs1: u32, rs2: u32 },
    SH3ADD_UW { rd: u32, rs1: u32, rs2: u32 },
    SLLI_UW { rd: u32, rs1: u32, shamt: u32 },
    // RV32Zbb extension:
    ANDN { rd: u32, rs1: u32, rs2: u32 },
    ORN { rd: u32, rs1: u32, rs2: u32 },
    XNOR { rd: u32, rs1: u32, rs2: u32 },
    CLZ { rd: u32, rs1: u32 },
    CTZ { rd: u32, rs1: u32 },
    CPOP { rd: u32, rs1: u32 },
    MAX { rd: u32, rs1: u32, rs2: u32 },
    MAXU { rd: u32, rs1: u32, rs2: u32 },
    MIN { rd: u32, rs1: u32, rs2: u32 },
    MINU { rd: u32, rs1: u32, rs2: u32 },
    SEXT_B { rd: u32, rs1: u32 },
    SEXT_H { rd: u32, rs1: u32 },
    ZEXT_H { rd: u32, rs1: u32 },
    // RV64Zbb extension:
    CLZW { rd: u32, rs1: u32 },
    CTZW { rd: u32, rs1: u32 },
    CPOPW { rd: u32, rs1: u32 },
    // Bitwise rotations (RV32Zbb AND RV32Zbkb extensions):
    ROL { rd: u32, rs1: u32, rs2: u32 },
    ROR { rd: u32, rs1: u32, rs2: u32 },
    RORI { rd: u32, rs1: u32, shamt: u32 },
    ORC_B { rd: u32, rs1: u32 },
    REV8 { rd: u32, rs1: u32 },
    // Bitwise rotations (RV64Zbb AND RV64Zbkb extensions):
    ROLW { rd: u32, rs1: u32, rs2: u32 },
    RORIW { rd: u32, rs1: u32, shamt: u32 },
    RORW { rd: u32, rs1: u32, rs2: u32 },
    // RV32Zbkb extension:
    PACK { rd: u32, rs1: u32, rs2: u32 },
    PACKH { rd: u32, rs1: u32, rs2: u32 },
    BREV8 { rd: u32, rs1: u32 },
    ZIP { rd: u32, rs1: u32 },
    UNZIP { rd: u32, rs1: u32 },
    // RV64Zbkb extension:
    PACKW { rd: u32, rs1: u32, rs2: u32 },
    // Zbc extension:
    CLMUL { rd: u32, rs1: u32, rs2: u32 },
    CLMULH { rd: u32, rs1: u32, rs2: u32 },
    CLMULR { rd: u32, rs1: u32, rs2: u32 },
    // Zbs extension:
    BCLR { rd: u32, rs1: u32, rs2: u32 },
    BCLRI { rd: u32, rs1: u32, shamt: u32 },
    BEXT { rd: u32, rs1: u32, rs2: u32 },
    BEXTI { rd: u32, rs1: u32, shamt: u32 },
    BINV { rd: u32, rs1: u32, rs2: u32 },
    BINVI { rd: u32, rs1: u32, shamt: u32 },
    BSET { rd: u32, rs1: u32, rs2: u32 },
    BSETI { rd: u32, rs1: u32, shamt: u32 },
    // RVV extension:
    VLE { vrd: u32, rs1: u32, width: u32},
    VSE { vrd: u32, rs1: u32, width: u32},
    
    VADD_VV {vrd : u32, vrs1: u32, vrs2: u32},
    VMUL_VX {vrd : u32, rs1: u32, vrs2: u32},
    VMUL_VV {vrd : u32, vrs1: u32, vrs2: u32},
    VFADD_VV {vrd : u32, vrs1: u32, vrs2: u32},
    VFSUB_VV {vrd : u32, vrs1: u32, vrs2: u32},
    VFMUL_VV {vrd : u32, vrs1: u32, vrs2: u32},
    VFMACC_VV {vrd : u32, vrs1: u32, vrs2: u32},
    
    VFSLIDE1UP_VF {vrd : u32, frs1: u32, vrs2: u32},
    VFSLIDE1DOWN_VF {vrd : u32, frs1: u32, vrs2: u32},
    
    VMVnR_V {vrd : u32, vrs1: u32},
    VMV_V_V {vrd : u32, vrs1: u32},

    VSETVLI { rd: u32, rs1: u32, sew : u32, lmul : LMUL, tail : VectorOption, mask : VectorOption},
    VSETIVLI { rd: u32, uimm : u32, sew : u32, lmul : LMUL, tail : VectorOption, mask : VectorOption},
    VSETVL {rd : u32, rs1: u32, rs2: u32}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VectorOption {
    Agnostic,
    Undisturbed
}
impl VectorOption {
    pub fn new(value: u32) -> Self {
        match value {
            1 => VectorOption::Agnostic,
            0 => VectorOption::Undisturbed,
            _ => panic!("Invalid vector option")
        }
    }
}
impl fmt::Display for VectorOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VectorOption::Agnostic => write!(f, "a"),
            VectorOption::Undisturbed => write!(f, "u")
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LMUL {
    vlmul : u32
}

impl fmt::Display for LMUL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.vlmul {
            0b101 => write!(f, "mf8"),
            0b110 => write!(f, "mf4"),
            0b111 => write!(f, "mf2"),
            x => write!(f, "m{}", 1 << x)
        }
    }
}

impl LMUL {
    pub fn new(vlmul : u32) -> LMUL {
        LMUL { vlmul }
    }
}
impl Instruction {
    /// Check whether an instruction could branch or jump.
    pub fn branch(&self) -> bool {
        matches!(
            self,
            Instruction::JAL { .. }
                | Instruction::JALR { .. }
                | Instruction::BEQ { .. }
                | Instruction::BNE { .. }
                | Instruction::BLT { .. }
                | Instruction::BGE { .. }
                | Instruction::BLTU { .. }
                | Instruction::BGEU { .. }
        )
    }

    /// Check whether an instruction could load from memory.
    pub fn load(&self) -> bool {
        matches!(
            self,
            Instruction::LB { .. }
                | Instruction::LH { .. }
                | Instruction::LW { .. }
                | Instruction::LBU { .. }
                | Instruction::LHU { .. }
                | Instruction::LWU { .. }
                | Instruction::LD { .. }
                | Instruction::LR_W { .. }
                | Instruction::LR_D { .. }
                | Instruction::AMOSWAP_W { .. }
                | Instruction::AMOADD_W { .. }
                | Instruction::AMOXOR_W { .. }
                | Instruction::AMOAND_W { .. }
                | Instruction::AMOOR_W { .. }
                | Instruction::AMOMIN_W { .. }
                | Instruction::AMOMAX_W { .. }
                | Instruction::AMOMINU_W { .. }
                | Instruction::AMOMAXU_W { .. }
                | Instruction::AMOSWAP_D { .. }
                | Instruction::AMOADD_D { .. }
                | Instruction::AMOXOR_D { .. }
                | Instruction::AMOAND_D { .. }
                | Instruction::AMOOR_D { .. }
                | Instruction::AMOMIN_D { .. }
                | Instruction::AMOMAX_D { .. }
                | Instruction::AMOMINU_D { .. }
                | Instruction::AMOMAXU_D { .. }
                | Instruction::FLW { .. }
                | Instruction::FLD { .. }
                | Instruction::FLQ { .. }
                | Instruction::FLH { .. }
        )
    }

    /// Check whether an instruction could store to memory.
    pub fn store(&self) -> bool {
        matches!(
            self,
            Instruction::SB { .. }
                | Instruction::SH { .. }
                | Instruction::SW { .. }
                | Instruction::SD { .. }
                | Instruction::SC_W { .. }
                | Instruction::SC_D { .. }
                | Instruction::AMOSWAP_W { .. }
                | Instruction::AMOADD_W { .. }
                | Instruction::AMOXOR_W { .. }
                | Instruction::AMOAND_W { .. }
                | Instruction::AMOOR_W { .. }
                | Instruction::AMOMIN_W { .. }
                | Instruction::AMOMAX_W { .. }
                | Instruction::AMOMINU_W { .. }
                | Instruction::AMOMAXU_W { .. }
                | Instruction::AMOSWAP_D { .. }
                | Instruction::AMOADD_D { .. }
                | Instruction::AMOXOR_D { .. }
                | Instruction::AMOAND_D { .. }
                | Instruction::AMOOR_D { .. }
                | Instruction::AMOMIN_D { .. }
                | Instruction::AMOMAX_D { .. }
                | Instruction::AMOMINU_D { .. }
                | Instruction::AMOMAXU_D { .. }
                | Instruction::FSW { .. }
                | Instruction::FSD { .. }
                | Instruction::FSQ { .. }
                | Instruction::FSH { .. }
        )
    }
}

/// Compressed RISC-V instruction.
///
/// Compressed instructions can be decompressed using [`Instruction::from`].
///
/// Instruction arguments are all in their decoded forms, meaning correctly
/// scaled and sign extended. Registers are normalised. Their names are as they
/// appear in specifications (for the most part), and include:
///
/// * `rd`, `rs1`, `rs2``: destination and source registers.
/// * `frd`, `frs2`: destination and source floating point registers.
/// * `offset`, `imm`, `shamt`: numerical offsets, immediates, and shift amounts.
/// * `pred`, `succ`: predecessor and successor IORW (input, output, read, write) flags.
/// * `rm`: floating point rounding mode.
/// * `rl`, `aq`: atomic release and acquire flags.
///
/// Assumed and redundant flags are not repeated, i.e. `rd` is not stored for
/// `c.swsp` as it's always `sp` (`x2`) and only `rd` is stored for `c.addi` as
/// `rs1` is always the same.
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compressed {
    // Unknown:
    UNIMP,
    // Zca extension:
    // Stack-pointer based loads and stores:
    C_LWSP { rd: u32, offset: i32 },
    C_LDSP { rd: u32, offset: i32 },
    C_LQSP { rd: u32, offset: i32 },
    C_SWSP { rs2: u32, offset: i32 },
    C_SDSP { rs2: u32, offset: i32 },
    C_SQSP { rs2: u32, offset: i32 },
    // Register based loads and stores:
    C_LW { rd: u32, rs1: u32, offset: i32 },
    C_LD { rd: u32, rs1: u32, offset: i32 },
    C_LQ { rd: u32, rs1: u32, offset: i32 },
    C_SW { rs1: u32, rs2: u32, offset: i32 },
    C_SD { rs1: u32, rs2: u32, offset: i32 },
    C_SQ { rs1: u32, rs2: u32, offset: i32 },
    // Control transfer instructions:
    C_J { offset: i32 },
    C_JAL { offset: i32 },
    C_JR { rs1: u32 },
    C_JALR { rs1: u32 },
    C_BEQZ { rs1: u32, offset: i32 },
    C_BNEZ { rs1: u32, offset: i32 },
    // Integer computation instructions:
    C_LI { rd: u32, imm: i32 },
    C_LUI { rd: u32, imm: u32 },
    C_ADDI { rd: u32, imm: i32 },
    C_ADDIW { rd: u32, imm: i32 },
    C_ADDI16SP { imm: i32 },
    C_ADDI4SPN { rd: u32, imm: i32 },
    C_SLLI { rd: u32, shamt: u32 },
    C_SRLI { rd: u32, shamt: u32 },
    C_SRAI { rd: u32, shamt: u32 },
    C_ANDI { rd: u32, imm: i32 },
    C_MV { rd: u32, rs2: u32 },
    C_ADD { rd: u32, rs2: u32 },
    C_AND { rd: u32, rs2: u32 },
    C_OR { rd: u32, rs2: u32 },
    C_XOR { rd: u32, rs2: u32 },
    C_SUB { rd: u32, rs2: u32 },
    C_ADDW { rd: u32, rs2: u32 },
    C_SUBW { rd: u32, rs2: u32 },
    // Misc and system instructions:
    C_NOP,
    C_EBREAK,
    // RV32Zcf extension:
    C_FLW { frd: u32, rs1: u32, offset: i32 },
    C_FLWSP { frd: u32, offset: i32 },
    C_FSW { rs1: u32, frs2: u32, offset: i32 },
    C_FSWSP { frs2: u32, offset: i32 },
    // Zcd extension:
    C_FLD { frd: u32, rs1: u32, offset: i32 },
    C_FLDSP { frd: u32, offset: i32 },
    C_FSD { rs1: u32, frs2: u32, offset: i32 },
    C_FSDSP { frs2: u32, offset: i32 },
}

impl From<Compressed> for Instruction {
    fn from(compressed: Compressed) -> Self {
        match compressed {
            Compressed::UNIMP => Instruction::UNIMP,
            // Zca extension:
            Compressed::C_LWSP { rd, offset } => Instruction::LW { rd, rs1: 2, offset },
            Compressed::C_LDSP { rd, offset } => Instruction::LD { rd, rs1: 2, offset },
            Compressed::C_LQSP { .. } => unimplemented!("rv128"),
            Compressed::C_SWSP { rs2, offset } => Instruction::SW { rs1: 2, rs2, offset },
            Compressed::C_SDSP { rs2, offset } => Instruction::SD { rs1: 2, rs2, offset },
            Compressed::C_SQSP { .. } => unimplemented!("rv128"),
            Compressed::C_LW { rd, rs1, offset } => Instruction::LW { rd, rs1, offset },
            Compressed::C_LD { rd, rs1, offset } => Instruction::LD { rd, rs1, offset },
            Compressed::C_LQ { .. } => unimplemented!("rv128"),
            Compressed::C_SW { rs1, rs2, offset } => Instruction::SW { rs1, rs2, offset },
            Compressed::C_SD { rs1, rs2, offset } => Instruction::SD { rs1, rs2, offset },
            Compressed::C_SQ { .. } => unimplemented!("rv128"),
            Compressed::C_J { offset } => Instruction::JAL { rd: 0, offset },
            Compressed::C_JAL { offset } => Instruction::JAL { rd: 1, offset },
            Compressed::C_JR { rs1 } => Instruction::JALR { rd: 0, rs1, offset: 0 },
            Compressed::C_JALR { rs1 } => Instruction::JALR { rd: 1, rs1, offset: 0 },
            Compressed::C_BEQZ { rs1, offset } => Instruction::BEQ { rs1, rs2: 0, offset },
            Compressed::C_BNEZ { rs1, offset } => Instruction::BNE { rs1, rs2: 0, offset },
            Compressed::C_LI { rd, imm } => Instruction::ADDI { rd, rs1: 0, imm },
            Compressed::C_LUI { rd, imm } => Instruction::LUI { rd, imm },
            Compressed::C_ADDI { rd, imm } => Instruction::ADDI { rd, rs1: rd, imm },
            Compressed::C_ADDIW { rd, imm } => Instruction::ADDIW { rd, rs1: rd, imm },
            Compressed::C_ADDI16SP { imm } => Instruction::ADDI { rd: 2, rs1: 2, imm },
            Compressed::C_ADDI4SPN { rd, imm } => Instruction::ADDI { rd, rs1: 2, imm },
            Compressed::C_SLLI { rd, shamt } => Instruction::SLLI { rd, rs1: rd, shamt },
            Compressed::C_SRLI { rd, shamt } => Instruction::SRLI { rd, rs1: rd, shamt },
            Compressed::C_SRAI { rd, shamt } => Instruction::SRAI { rd, rs1: rd, shamt },
            Compressed::C_ANDI { rd, imm } => Instruction::ANDI { rd, rs1: rd, imm },
            Compressed::C_MV { rd, rs2 } => Instruction::ADD { rd, rs1: 0, rs2 },
            Compressed::C_ADD { rd, rs2 } => Instruction::ADD { rd, rs1: rd, rs2 },
            Compressed::C_AND { rd, rs2 } => Instruction::AND { rd, rs1: rd, rs2 },
            Compressed::C_OR { rd, rs2 } => Instruction::OR { rd, rs1: rd, rs2 },
            Compressed::C_XOR { rd, rs2 } => Instruction::XOR { rd, rs1: rd, rs2 },
            Compressed::C_SUB { rd, rs2 } => Instruction::SUB { rd, rs1: rd, rs2 },
            Compressed::C_ADDW { rd, rs2 } => Instruction::ADDW { rd, rs1: rd, rs2 },
            Compressed::C_SUBW { rd, rs2 } => Instruction::SUBW { rd, rs1: rd, rs2 },
            Compressed::C_NOP => Instruction::ADDI { rd: 0, rs1: 0, imm: 0 },
            Compressed::C_EBREAK => Instruction::EBREAK,
            // Zcf extension:
            Compressed::C_FLW { frd, rs1, offset } => Instruction::FLW { frd, rs1, offset },
            Compressed::C_FLWSP { frd, offset } => Instruction::FLW { frd, rs1: 2, offset },
            Compressed::C_FSW { rs1, frs2, offset } => Instruction::FSW { rs1, frs2, offset },
            Compressed::C_FSWSP { frs2, offset } => Instruction::FSW { rs1: 2, frs2, offset },
            // Zcd extension:
            Compressed::C_FLD { frd, rs1, offset } => Instruction::FLD { frd, rs1, offset },
            Compressed::C_FLDSP { frd, offset } => Instruction::FLD { frd, rs1: 2, offset },
            Compressed::C_FSD { rs1, frs2, offset } => Instruction::FSD { rs1, frs2, offset },
            Compressed::C_FSDSP { frs2, offset } => Instruction::FSD { rs1: 2, frs2, offset },
        }
    }
}

impl Compressed {
    /// Check whether an instruction could branch or jump.
    pub fn branch(&self) -> bool {
        matches!(
            self,
            Compressed::C_J { .. }
                | Compressed::C_JAL { .. }
                | Compressed::C_JR { .. }
                | Compressed::C_JALR { .. }
                | Compressed::C_BEQZ { .. }
                | Compressed::C_BNEZ { .. }
        )
    }
}

/// Bitfield for input/output/read/write fields, e.g. for `FENCE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Iorw(pub(crate) u8);
