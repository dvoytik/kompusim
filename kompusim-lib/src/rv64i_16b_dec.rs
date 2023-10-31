// RV64i 16-bit compressed instruction decoder

// TODO: rename 16b to rvc everywhere

use crate::{alu::I6, bits::BitOps};

pub enum COpcode {
    CLI { imm6: I6, rd: u8 },
    CJR { rs1: u8 },
    Uknown,
}

// Use mod to disable auto-formatting for all definitions
/// RVC (compressed) 16b instructin opcodes (inst[15:13], inst[1:0])
#[rustfmt::skip]
mod c_opcodes {
pub const OPC_C_JR_MV_EBREAK_JALR_ADD: u8 = 0b_100_10;
pub const OPC_C_LI: u8 =                    0b_010_01;
}
use c_opcodes::*;

/// Check whether the instruction is in compressed format (16-bit)
#[inline(always)]
pub fn instr_is_16b(instr: u32) -> bool {
    !(instr & 0x3 == 0x3)
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
pub fn decode_16b_instr(c_instr: u16) -> COpcode {
    let rd = c_i_rd(c_instr);

    match c_i_opcode(c_instr) {
        OPC_C_LI if rd != 0 => COpcode::CLI {
            imm6: c_i_imm6(c_instr),
            rd: c_i_rd(c_instr),
        },
        OPC_C_JR_MV_EBREAK_JALR_ADD => {
            let rs2 = c_i_rs2(c_instr);
            let rs1 = c_i_rs1(c_instr);
            match (c_instr.bits(12, 12), rs2) {
                //  C.JR (jump register) performs an unconditional control transfer to the address in
                //  register rs1. C.JR expands to jalr x0, 0(rs1). C.JR is only valid when rs!=x0;
                //  the code point with rs1=x0 is reserved.
                (0, 0) if rs1 != 0 => COpcode::CJR { rs1 },
                _ => COpcode::Uknown,
            }
        }
        _ => COpcode::Uknown,
    }
}
