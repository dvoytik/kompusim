// RV64i 32-bit instruction decoder

use crate::alu::{I12, I13, I21};
use crate::bits::BitOps;

pub enum Opcode {
    System {
        csr: u16,
        rs1: u8,
        funct3: u8,
        rd: u8,
    },
    Branch {
        off13: I13,
        rs2: u8,
        rs1: u8,
        funct3: u8,
    },
    Auipc {
        uimm20: u64,
        rd: u8,
    },
    OpImm {
        imm12: I12,
        rs1: u8,
        funct3: u8,
        rd: u8,
    },
    ADDIW {
        imm12: I12,
        rs1: u8,
        rd: u8,
    },
    SLLIW {
        /// shift amount
        shamt: u8,
        rs1: u8,
        rd: u8,
    },
    Op {
        funct7: u8,
        rs2: u8,
        rs1: u8,
        funct3: u8,
        rd: u8,
    },
    Jal {
        imm21: I21,
        rd: u8,
    },
    Jalr {
        imm12: I12,
        rs1: u8,
        rd: u8,
    },
    LUI {
        uimm20: u32,
        rd: u8,
    },
    Load {
        imm12: I12,
        rs1: u8,
        funct3: u8,
        rd: u8,
    },
    Store {
        imm12: I12,
        rs2: u8,
        rs1: u8,
        funct3: u8,
    },
    /// Atomic Memory Operations
    Amo {
        funct5: u8,
        rs2: u8,
        rs1: u8,
        funct3: u8,
        rd: u8,
        aq: bool,
        rl: bool,
    },
    Fence {
        imm12: I12,
        funct3: u8,
        // rs1 and rd fields are reserved
    },
    Uknown,
}

// TODO:
// #[repr(u8)]
// enum Opcodes {
//
// }

// Use mod to disable auto-formatting for all definitions
/// RV32/RV64 instruction opcodes (inst[6:0])
#[rustfmt::skip]
mod opc {
pub const OPC_SYSTEM: u8 =   0b_11_100_11;
pub const OPC_BRANCH: u8 =   0b_11_000_11;
pub const OPC_AUIPC:  u8 =   0b_00_101_11;
pub const OPC_OP_IMM: u8 =   0b_00_100_11;
pub const OPC_OP_IMM32: u8 = 0b_00_110_11;
pub const OPC_OP:     u8 =   0b_01_100_11;
pub const OPC_JALR:   u8 =   0b_11_001_11;
pub const OPC_FENCE:  u8 =   0b_00_011_11;
pub const OPC_AMO:    u8 =   0b_01_011_11;
pub const OPC_JAL:    u8 =   0b_11_011_11;
pub const OPC_LUI:    u8 =   0b_01_101_11;
pub const OPC_LOAD:   u8 =   0b_00_000_11;
pub const OPC_STORE:  u8 =   0b_01_000_11;

pub const F3_BRANCH_BEQ: u8  = 0b000; // Branch EQual
pub const F3_BRANCH_BNE: u8  = 0b001; // Branch Not Equal
pub const F3_BRANCH_BLT: u8  = 0b100; // Branch Less Than (Signed)
pub const F3_BRANCH_BLTU: u8 = 0b110; // Branch Less Than Unsigned
pub const F3_BRANCH_BGE: u8  = 0b101; // Branch Greater or Equal (Signed)
pub const F3_BRANCH_BGEU: u8 = 0b111; // Branch Greater or Equal (Unsigned)

pub const F3_SYSTEM_WFI: u8    = 0b000; // Wait For Interrupt
pub const F3_SYSTEM_CSRRW: u8  = 0b001; // atomic CSR read, write
pub const F3_SYSTEM_CSRRS: u8  = 0b010; // atomic CSR read, set bits
pub const F3_SYSTEM_CSRRWI: u8 = 0b101; // atomic CSR read, write immidiate

pub const F3_OP_IMM_ADDI: u8 = 0b000;
pub const F3_OP_IMM_SLLI: u8 = 0b001;
pub const F3_OP_IMM_SRLI: u8 = 0b101;

pub const F3_OP_IMM32_ADDIW: u8 = 0b000;
pub const F3_OP_IMM32_SLLIW: u8 = 0b001; // Shift Left Logical Immediate Word

pub const F3_OP_ADD_SUB: u8 = 0b_000;

pub const F3_OP_LOAD_LB:  u8 = 0b000;
pub const F3_OP_LOAD_LBU: u8 = 0b100;
pub const F3_OP_LOAD_LW:  u8 = 0b010;
pub const F3_OP_LOAD_LD:  u8 = 0b011;

pub const F3_OP_STORE_SB: u8 = 0b000;
pub const F3_OP_STORE_SW: u8 = 0b010;
pub const F3_OP_STORE_SD: u8 = 0b011;

// funct7 field of R-type instruction
pub const F7_OP_ADD: u8 = 0b_000_0000;
pub const F7_OP_SUB: u8 = 0b_010_0000;

// func5 field of AMO instructions
pub const F5_OP_AMO_ADD: u8   = 0b_00000;
pub const F5_OP_AMO_SWAP: u8  = 0b_00001;
pub const F5_OP_AMO_LRW: u8   = 0b_00010;

pub const F3_OP_AMO_WORD: u8  = 0b_010;
pub const F3_OP_AMO_DWORD: u8 = 0b_011;

pub const F3_OP_FENCE: u8   = 0b_000;
pub const F3_OP_FENCE_I: u8 = 0b_001;
}
pub use opc::*;

#[inline(always)]
pub fn i_opcode(ins: u32) -> u8 {
    ins.bits(6, 0) as u8
}

#[inline(always)]
pub fn i_funct3(ins: u32) -> u8 {
    ins.bits(14, 12) as u8
}

#[inline(always)]
pub fn i_rd(ins: u32) -> u8 {
    ins.bits(11, 7) as u8
}

#[inline(always)]
pub fn i_rs1(ins: u32) -> u8 {
    ins.bits(19, 15) as u8
}

#[inline(always)]
pub fn i_rs2(ins: u32) -> u8 {
    ins.bits(24, 20) as u8
}

#[inline(always)]
fn i_csr(ins: u32) -> u16 {
    ins.bits(31, 20) as u16
}

// Decode 13-bit signed offset from a B-type instruction
#[inline(always)]
pub fn i_b_off13(ins: u32) -> I13 {
    let off_4_1 = ins.bits(11, 8) as u16;
    let off_11 = ins.bits(7, 7) as u16;
    let off_10_5 = ins.bits(30, 25) as u16;
    let off_12 = ins.bits(31, 31) as u16;
    I13::from(off_12 << 12 | off_11 << 11 | off_10_5 << 5 | off_4_1 << 1)
}

// extract upper 20-bit for LUI, AUIPC instructions
#[inline(always)]
pub fn i_u_uimm20(ins: u32) -> u32 {
    ins & 0xffff_f000
}

// Decode signed 12-bit immidiate from I-type instruction
#[inline(always)]
pub fn i_i_type_imm12(ins: u32) -> I12 {
    I12::from(ins.bits(31, 20) as u16)
}

// Decode funct7 field (inst[31:25]) from the R-type (integer register-register) instruction
#[inline(always)]
pub fn i_r_type_funct7(ins: u32) -> u8 {
    ins.bits(31, 25) as u8
}

// Decode signed 12-bit immidiate from S-type instruction
#[inline(always)]
pub fn i_s_type_imm12(ins: u32) -> I12 {
    let imm11_5 = ins.bits(31, 25) as u16;
    let imm4_0 = ins.bits(11, 7) as u16;
    I12::from(imm11_5 << 5 | imm4_0)
}

/// Decodes Zics SYSTEM opcodes: CSRRS, WFI, ...
pub fn dec_opc_system(ins: u32) -> Opcode {
    // I-type instruction
    let rd = i_rd(ins);
    let funct3 = i_funct3(ins);
    let rs1 = i_rs1(ins);
    let csr = i_csr(ins);
    Opcode::System {
        csr,
        rs1,
        funct3,
        rd,
    }
}

/// Decodes BRANCH opcodes: BEQ, BNE, BLT, ...
pub fn dec_opc_branch(instr: u32) -> Opcode {
    // B-type instructions
    let off13 = i_b_off13(instr);
    let rs2 = i_rs2(instr);
    let rs1 = i_rs1(instr);
    let funct3 = i_funct3(instr);
    Opcode::Branch {
        off13,
        rs2,
        rs1,
        funct3,
    }
}

pub fn dec_opc_op_imm(instr: u32) -> Opcode {
    // I-type instructions
    let imm12 = i_i_type_imm12(instr);
    let rs1 = i_rs1(instr);
    let funct3 = i_funct3(instr);
    let rd = i_rd(instr);
    Opcode::OpImm {
        imm12,
        rs1,
        funct3,
        rd,
    }
}

// R-type instructions: ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND
pub fn dec_opc_op(instr: u32) -> Opcode {
    Opcode::Op {
        funct7: i_r_type_funct7(instr),
        rs2: i_rs2(instr),
        rs1: i_rs1(instr),
        funct3: i_funct3(instr),
        rd: i_rd(instr),
    }
}

pub fn dec_opc_jal(instr: u32) -> Opcode {
    let imm21 = instr.bits(31, 31) << 20
        | instr.bits(19, 12) << 12
        | instr.bits(20, 20) << 11
        | instr.bits(30, 21) << 1;
    let imm21 = I21::from(imm21);
    let rd = i_rd(instr);
    Opcode::Jal { imm21, rd }
}

pub fn dec_opc_jalr(instr: u32) -> Opcode {
    let imm12 = i_i_type_imm12(instr);
    let rs1 = i_rs1(instr);
    let rd = i_rd(instr);
    Opcode::Jalr { imm12, rs1, rd }
}

pub fn dec_opc_load(instr: u32) -> Opcode {
    let imm12 = i_i_type_imm12(instr);
    let rs1 = i_rs1(instr);
    let funct3 = i_funct3(instr);
    let rd = i_rd(instr);
    Opcode::Load {
        imm12,
        rs1,
        funct3,
        rd,
    }
}

pub fn dec_opc_store(instr: u32) -> Opcode {
    let imm12 = i_s_type_imm12(instr);
    let rs2 = i_rs2(instr);
    let rs1 = i_rs1(instr);
    let funct3 = i_funct3(instr);
    Opcode::Store {
        imm12,
        rs2,
        rs1,
        funct3,
    }
}

pub fn decode_instr(instr: u32) -> Opcode {
    match i_opcode(instr) {
        OPC_LUI => Opcode::LUI {
            uimm20: i_u_uimm20(instr),
            rd: i_rd(instr),
        },
        OPC_AUIPC => Opcode::Auipc {
            uimm20: i_u_uimm20(instr) as u64,
            rd: i_rd(instr),
        },
        OPC_JAL => dec_opc_jal(instr),
        OPC_JALR => dec_opc_jalr(instr),
        OPC_FENCE => Opcode::Fence {
            imm12: i_i_type_imm12(instr),
            funct3: i_funct3(instr),
        },
        OPC_BRANCH => dec_opc_branch(instr),
        OPC_LOAD => dec_opc_load(instr),
        OPC_STORE => dec_opc_store(instr),
        OPC_OP_IMM => dec_opc_op_imm(instr),
        OPC_OP_IMM32 => {
            let funct3 = i_funct3(instr);
            let rs1 = i_rs1(instr);
            let rd = i_rd(instr);
            if funct3 == F3_OP_IMM32_ADDIW {
                Opcode::ADDIW {
                    imm12: i_i_type_imm12(instr),
                    rs1,
                    rd,
                }
            } else {
                let bits31_25 = instr.bits(31, 25);
                match (bits31_25, funct3) {
                    (0b0000000, F3_OP_IMM32_SLLIW) => Opcode::SLLIW {
                        shamt: i_rs2(instr),
                        rs1,
                        rd,
                    },
                    (_, _) => Opcode::Uknown,
                }
            }
        }
        OPC_OP => dec_opc_op(instr),
        OPC_SYSTEM => dec_opc_system(instr),
        OPC_AMO => Opcode::Amo {
            funct5: instr.bits(31, 27) as u8,
            // If the aq bit is set, then no later memory operations in this RISC-V hart can be
            // observed to take place before the AMO.
            // If the rl bit is set, then other RISC-V harts will not observe the AMO before
            // memory accesses preceding the AMO in this RISC-V hart.
            // Setting both the aq and the rl bit on an AMO makes the sequence sequentially
            // consistent, meaning that it cannot be reordered with earlier or later memory
            // operations from the same hart.
            // TODO: aq, rl fields are ingonred for now
            aq: instr.bit(26),
            rl: instr.bit(25),
            rs2: i_rs2(instr),
            rs1: i_rs1(instr),
            funct3: i_funct3(instr),
            rd: i_rd(instr),
        },
        _ => Opcode::Uknown,
    }
}
