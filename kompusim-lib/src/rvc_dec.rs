// RV64i 16-bit compressed instruction decoder

// TODO: rename 16b to rvc everywhere

use crate::{
    alu::{I12, I6, I9},
    bits::BitOps,
};

pub enum COpcode {
    CNOP,
    CADDI {
        imm6: I6,
        rd: u8,
    },
    CLUI {
        imm6: I6,
        rd: u8,
    },
    ADDI16SP {
        imm6: I6,
    },
    CSLLI {
        uimm6: u8,
        rd: u8,
    },
    /// Shift Right Logical Immidiate
    CSRLI {
        shamt6: u8,
        rd: u8,
    },
    CLI {
        imm6: I6,
        rd: u8,
    },
    CJR {
        rs1: u8,
    },
    CADD {
        rd: u8,
        rs2: u8,
    },
    CADDW {
        rd: u8,
        rs2: u8,
    },
    CANDI {
        imm6: I6,
        rd: u8,
    },
    COR {
        rd: u8,
        rs2: u8,
    },
    CJ {
        imm12: I12,
    },
    BEQZ {
        imm9: I9,
        rs1: u8,
    },
    BNEZ {
        imm9: I9,
        rs1: u8,
    },
    SDSP {
        uimm6: u8,
        rs2: u8,
    },
    LDSP {
        uimm6: u8,
        rd: u8,
    },
    ADDI4SPN {
        uimm8: u8,
        rd: u8,
    },
    LD {
        uoff8: u8,
        rs1: u8,
        rd: u8,
    },
    SW {
        uoff7: u8,
        rs1: u8,
        rs2: u8,
    },
    MV {
        rd: u8,
        rs2: u8,
    },
    ADDIW {
        rd: u8,
        uimm6: u8,
    },
    Hint,
    Reserved,
    Uknown,
}

// TODO: do we need these const defines or should I move it to match?
// Use mod to disable auto-formatting for all definitions
/// RVC (compressed) 16b instructin opcodes (inst[15:13], inst[1:0])
#[rustfmt::skip]
mod c_opcodes {
pub const OPC_C_ADDI4SPN: u8 =              0b_000_00; // add immediate x 4 to SP
pub const OPC_C_LD: u8 =                    0b_011_00; // Load Double-word
pub const OPC_C_SW: u8 =                    0b_110_00; // Store Word
pub const OPC_C_NOP_ADDI: u8 =              0b_000_01;
pub const OPC_C_ADDIW : u8 =                0b_001_01; // Add Immidiate Word
pub const OPC_C_LI: u8 =                    0b_010_01;
pub const OPC_C_LUI_ADDI16SP: u8 =          0b_011_01;
pub const OPC_C_MISC_ALU: u8 =              0b_100_01; // C.{SRLI, SRAI, ANDI, SUB, XOR, OR, ...}
pub const OPC_C_J: u8 =                     0b_101_01;
pub const OPC_C_BEQZ: u8 =                  0b_110_01; // Branch Equal Zero
pub const OPC_C_BNEZ: u8 =                  0b_111_01; // Branch Not Equal Zero
pub const OPC_C_SLLI: u8 =                  0b_000_10; // shift logical left immidiate
pub const OPC_C_JR_MV_EBREAK_JALR_ADD: u8 = 0b_100_10;
pub const OPC_C_SDSP: u8 =                  0b_111_10; // Store (in memory) Dword by Stack Pointer
pub const OPC_C_LDSP: u8 =                  0b_011_10; // Load (from memory) Dword by Stack Pointer
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

// Decode immediate 9-bit field of instructions c.beqz and c.bnez
#[inline(always)]
fn c_i_imm9(c_instr: u16) -> I9 {
    let imm9: u16 = c_instr.bits(12, 12) << 8
        | c_instr.bits(6, 5) << 6
        | c_instr.bits(2, 2) << 5
        | c_instr.bits(11, 10) << 3
        | c_instr.bits(4, 3) << 1;
    imm9.into()
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
/// rd' field
pub fn c_i_rd_s(c_instr: u16) -> u8 {
    c_instr.bits(4, 2) as u8 + 8
}

#[inline(always)]
/// rs1' field
pub fn c_i_rs1_s(c_instr: u16) -> u8 {
    c_instr.bits(9, 7) as u8 + 8
}

#[inline(always)]
/// rs2' field
pub fn c_i_rs2_s(c_instr: u16) -> u8 {
    c_instr.bits(4, 2) as u8 + 8
}

#[inline(always)]
pub fn c_i_rs1(c_instr: u16) -> u8 {
    c_instr.bits(11, 7) as u8
}

#[inline(always)]
pub fn c_i_rs2(c_instr: u16) -> u8 {
    c_instr.bits(6, 2) as u8
}

/// Decode a compressed (16 bit) RV64 instruction
pub fn rv64c_decode_instr(c_instr: u16) -> COpcode {
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
                    COpcode::ADDI16SP { imm6: imm6.into() }
                }
                (_, _) => COpcode::CLUI { imm6, rd },
            }
        }
        OPC_C_J => COpcode::CJ {
            imm12: c_i_imm12(c_instr),
        },
        OPC_C_BEQZ => {
            let rs1 = c_instr.bits(9, 7) as u8 + 8; // rs1' field
            COpcode::BEQZ {
                imm9: c_i_imm9(c_instr),
                rs1,
            }
        }
        OPC_C_BNEZ => {
            let rs1 = c_instr.bits(9, 7) as u8 + 8; // rs1' field
            COpcode::BNEZ {
                imm9: c_i_imm9(c_instr),
                rs1,
            }
        }
        OPC_C_JR_MV_EBREAK_JALR_ADD => {
            let rs2 = c_i_rs2(c_instr);
            let rs1 = c_i_rs1(c_instr); // also rd for C.MV and C.ADD
            let bit12 = c_instr.bits(12, 12);
            match (bit12, rs1, rs2) {
                //  C.JR (jump register) performs an unconditional control transfer to the address in
                //  register rs1. C.JR expands to jalr x0, 0(rs1). C.JR is only valid when rs!=x0;
                //  the code point with rs1=x0 is reserved.
                (0, 0, 0) => COpcode::Reserved,
                (0, 0, _) => COpcode::Hint,
                (0, rs1, 0) => COpcode::CJR { rs1 },
                (0, rd, rs2) => COpcode::MV { rd, rs2 },
                (1, rd, rs2) if rd != 0 && rs2 != 0 => COpcode::CADD { rd, rs2 },
                _ => COpcode::Uknown,
            }
        }
        OPC_C_SDSP => {
            let uimm6 = c_instr.bits(9, 7) << 3 | c_instr.bits(12, 10);
            let rs2 = c_i_rs2(c_instr);
            COpcode::SDSP {
                uimm6: uimm6 as u8,
                rs2,
            }
        }
        OPC_C_LDSP => {
            if rd != 0 {
                let uimm6 =
                    c_instr.bits(4, 2) << 3 | c_instr.bits(12, 12) << 2 | c_instr.bits(6, 5);
                COpcode::LDSP {
                    uimm6: uimm6 as u8,
                    rd,
                }
            } else {
                COpcode::Reserved
            }
        }
        OPC_C_ADDI4SPN => {
            let nz_uimm8 = c_instr.bits(10, 7) << 4
                | c_instr.bits(12, 11) << 2
                | c_instr.bits(5, 5) << 1
                | c_instr.bits(6, 6);
            if nz_uimm8 == 0 {
                COpcode::Reserved
            } else {
                COpcode::ADDI4SPN {
                    uimm8: nz_uimm8 as u8,
                    rd: c_i_rd_s(c_instr),
                }
            }
        }
        OPC_C_LD => {
            let uimm8 = c_instr.bits(6, 5) << 6 | c_instr.bits(12, 10) << 3;
            COpcode::LD {
                uoff8: uimm8 as u8,
                rs1: c_i_rs1_s(c_instr),
                rd: c_i_rd_s(c_instr),
            }
        }
        OPC_C_SW => {
            let uimm7 =
                c_instr.bits(5, 5) << 6 | c_instr.bits(12, 10) << 3 | c_instr.bits(6, 6) << 2;
            COpcode::SW {
                uoff7: uimm7 as u8,
                rs1: c_i_rs1_s(c_instr),
                rs2: c_i_rs2_s(c_instr),
            }
        }
        OPC_C_ADDIW => {
            if rd != 0 {
                let uimm6 = c_instr.bits(12, 12) << 5 | c_instr.bits(6, 2);
                COpcode::ADDIW {
                    rd,
                    uimm6: uimm6 as u8,
                }
            } else {
                COpcode::Reserved
            }
        }
        // C.SRLI, C.SRAI, C.ANDI, C.SUB, C.XOR, C.OR, C.AND, C.SUBW, C.ADDW
        OPC_C_MISC_ALU => {
            let bit12 = c_instr.bits(12, 12);
            let bits11_10 = c_instr.bits(11, 10);
            let bits6_5 = c_instr.bits(6, 5);
            let rd = c_instr.bits(9, 7) as u8 + 8;
            let rs2 = c_instr.bits(4, 2) as u8 + 8;
            match (bit12, bits11_10, bits6_5) {
                (0b_0, 0b_00, _) => COpcode::Hint,
                (bit12, 0b_00, _) => COpcode::CSRLI {
                    shamt6: (bit12 << 5 | c_instr.bits(6, 2)) as u8,
                    rd,
                },
                (0b_1, 0b_11, 0b_01) => COpcode::CADDW { rd, rs2 },
                (imm5, 0b_10, _) => COpcode::CANDI {
                    imm6: I6::from(imm5 << 5 | c_instr.bits(6, 2)),
                    rd,
                },
                (0b_0, 0b_11, 0b_10) => COpcode::COR { rd, rs2 },
                (_, _, _) => COpcode::Uknown,
            }
        }

        _ => COpcode::Uknown,
    }
}
