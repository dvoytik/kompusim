// RV64i 16-bit compressed instruction decoder

// TODO: rename 16b to rvc everywhere

use crate::{
    alu::{I12, I6},
    bits::BitOps,
};

pub enum COpcode {
    CNOP,
    CADDI { imm6: I6, rd: u8 },
    CLUI { imm6: I6, rd: u8 },
    ADDI6SP { imm6: I6 },
    CSLLI { uimm6: u8, rd: u8 },
    CLI { imm6: I6, rd: u8 },
    CJR { rs1: u8 },
    CADD { rd: u8, rs2: u8 },
    CJ { imm12: I12 },
    Hint,
    Reserved,
    Uknown,
}

// TODO: do we need these const defines or should I move it to match?
// Use mod to disable auto-formatting for all definitions
/// RVC (compressed) 16b instructin opcodes (inst[15:13], inst[1:0])
#[rustfmt::skip]
mod c_opcodes {
pub const OPC_C_NOP_ADDI: u8 =              0b_000_01;
pub const OPC_C_SLLI: u8 =                  0b_000_10; // shift logical left immidiate
pub const OPC_C_LI: u8 =                    0b_010_01;
pub const OPC_C_LUI_ADDI16SP: u8 =          0b_011_01;
pub const OPC_C_J: u8 =                     0b_101_01;
pub const OPC_C_JR_MV_EBREAK_JALR_ADD: u8 = 0b_100_10;
}
use c_opcodes::*;

/// Check whether the instruction is in compressed format (16-bit)
#[inline(always)]
pub fn instr_is_rvc(instr: u32) -> bool {
    instr & 0x3 != 0x3
}

/// Get opcode of the compressed (16 bit) instruction
#[inline(always)]
pub fn c_i_opcode(c_instr: u16) -> u8 {
    // 5-bit opcode
    (c_instr.bits(15, 13) << 2 | c_instr.bits(1, 0)) as u8
}

// Decode signed 6-bit immidiate from CI instruction format
#[inline(always)]
pub fn c_i_imm6(c_instr: u16) -> I6 {
    I6::from(c_instr.bits(12, 12) << 5 | c_instr.bits(6, 2))
}

// Decode signed 12-bit immidiate from C.J and C.JAL instruction format
#[inline(always)]
pub fn c_i_imm12(c_instr: u16) -> I12 {
    I12::from(
        c_instr.bits(12, 12) << 11
            | c_instr.bits(11, 11) << 4
            | c_instr.bits(10, 9) << 8
            | c_instr.bits(8, 8) << 10
            | c_instr.bits(7, 7) << 6
            | c_instr.bits(6, 6) << 7
            | c_instr.bits(5, 3) << 1
            | c_instr.bits(2, 2) << 5,
    )
}

#[inline(always)]
pub fn c_i_rd(c_instr: u16) -> u8 {
    c_instr.bits(11, 7) as u8
}

#[inline(always)]
pub fn c_i_rs1(c_instr: u16) -> u8 {
    c_instr.bits(11, 7) as u8
}

#[inline(always)]
pub fn c_i_rs2(c_instr: u16) -> u8 {
    c_instr.bits(6, 2) as u8
}

/// Decode a compressed (16 bit) instruction
pub fn decode_rvc_instr(c_instr: u16) -> COpcode {
    let rd = c_i_rd(c_instr);

    match c_i_opcode(c_instr) {
        OPC_C_NOP_ADDI => {
            let imm6 = c_i_imm6(c_instr);
            if rd == 0 {
                COpcode::CNOP
            } else {
                COpcode::CADDI { imm6, rd }
            }
        }
        OPC_C_SLLI => {
            let uimm6: u8 = (c_instr.bits(12, 12) << 5 | c_instr.bits(6, 2)) as u8;
            if uimm6 != 0 && rd != 0 {
                COpcode::CSLLI { uimm6, rd }
            } else {
                COpcode::Hint
            }
        }
        OPC_C_LI if rd != 0 => COpcode::CLI {
            imm6: c_i_imm6(c_instr),
            rd: c_i_rd(c_instr),
        },
        OPC_C_LUI_ADDI16SP => {
            let imm6 = c_i_imm6(c_instr);
            match (rd, imm6.0) {
                (0, _) => COpcode::Hint,
                (_, 0) => COpcode::Reserved,
                (2, _) => {
                    let imm6 = c_instr.bits(12, 12) << 5
                        | c_instr.bits(4, 3) << 3
                        | c_instr.bits(5, 5) << 2
                        | c_instr.bits(2, 2) << 1
                        | c_instr.bits(6, 6);
                    COpcode::ADDI6SP { imm6: imm6.into() }
                }
                (_, _) => COpcode::CLUI { imm6, rd },
            }
        }
        OPC_C_J => COpcode::CJ {
            imm12: c_i_imm12(c_instr),
        },
        OPC_C_JR_MV_EBREAK_JALR_ADD => {
            let rs2 = c_i_rs2(c_instr);
            let rs1 = c_i_rs1(c_instr); // also rd for C.MV and C.ADD
            let bit12 = c_instr.bits(12, 12);
            match (bit12, rs1, rs2) {
                //  C.JR (jump register) performs an unconditional control transfer to the address in
                //  register rs1. C.JR expands to jalr x0, 0(rs1). C.JR is only valid when rs!=x0;
                //  the code point with rs1=x0 is reserved.
                (0, rs1, 0) if rs1 != 0 => COpcode::CJR { rs1 },
                (1, rd, rs2) if rd != 0 && rs2 != 0 => COpcode::CADD { rd, rs2 },
                _ => COpcode::Uknown,
            }
        }
        _ => COpcode::Uknown,
    }
}
