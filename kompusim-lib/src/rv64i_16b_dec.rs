// RV64i 16-bit compressed instruction decoder

use crate::{alu::I6, bits::BitOps};

pub enum COpcode {
    CLI { imm6: I6, rd: u8 },
    Uknown,
}

// Use mod to disable auto-formatting for all definitions
/// RV32/RV64 compressed (16b) instruction opcodes (inst[6:0])
#[rustfmt::skip]
mod c_opcodes {
/// RVC (compressed) instructin opcodes (inst[15:13], inst[1:0])
pub const OPC_C_LI:    u8 = 0b_010_01;
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
    I6::from(c_instr.bits(12, 12) | c_instr.bits(6, 2))
}

#[inline(always)]
pub fn c_i_rd(c_instr: u16) -> u8 {
    c_instr.bits(11, 7) as u8
}

/// Decode the compressed (16 bit) instruction
pub fn decode_c_instr(c_instr: u16) -> COpcode {
    let rd = c_i_rd(c_instr); // rd also encodes instruction
    match c_i_opcode(c_instr) {
        OPC_C_LI if rd != 0 => COpcode::CLI {
            imm6: c_i_imm6(c_instr),
            rd,
        },
        _ => COpcode::Uknown,
    }
}
