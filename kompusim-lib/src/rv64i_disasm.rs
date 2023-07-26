// Disassembler

use crate::{alu::Imm, bits::BitOps, rv64i_dec::*};

/// instr_a - instruction address
pub fn disasm(instr: u32, instr_a: u64) -> String {
    match decode_instr(instr) {
        Opcode::Lui { uimm20, rd } => format!("lui x{rd}, 0x{:x}", uimm20 >> 12),

        Opcode::Auipc { uimm20, rd } => format!("auipc x{rd}, 0x{uimm20:x}"),

        Opcode::Branch {
            off13,
            rs2,
            rs1,
            funct3,
        } => {
            let addr = instr_a.add_i13(off13);
            match funct3 {
                // Branch Not Equal
                F3_BRANCH_BNE => format!("bne x{rs1}, x{rs2}, 0x{addr:x} # PC + 0x{off13:x}"),
                // Branch EQual
                F3_BRANCH_BEQ => format!("beq x{rs1}, x{rs2}, 0x{addr:x} # PC + 0x{off13:x}"),
                // Branch Less Than (signed comparison)
                F3_BRANCH_BLT => format!("blt x{rs1}, x{rs2}, 0x{addr:x} # PC + 0x{off13:x}"),
                _ => "uknown BRANCH instruction".to_string(),
            }
        }

        Opcode::Jal { imm21, rd } => {
            format!(
                "jal x{rd}, 0x{0:x} # PC + {imm21:x}",
                instr_a.add_i21(imm21)
            )
        }

        Opcode::Jalr { imm12, rs1, rd } => {
            format!("jalr x{rd}, 0x{imm12:x}(x{rs1}) # ")
        }

        Opcode::Load {
            imm12,
            rs1,
            funct3,
            rd,
        } => match funct3 {
            F3_OP_LOAD_LB => format!("lb x{rd}, 0x{imm12}(x{rs1})"),
            F3_OP_LOAD_LBU => format!("lbu x{rd}, 0x{imm12}(x{rs1})"),
            F3_OP_LOAD_LW => format!("lw x{rd}, 0x{imm12}(x{rs1})"),
            _ => "uknown LOAD instruction".to_string(),
        },

        Opcode::Store {
            imm12,
            rs2,
            rs1,
            funct3,
        } => match funct3 {
            F3_OP_STORE_SB => format!("sb x{rs2}, 0x{imm12:x}(x{rs1})"),
            F3_OP_STORE_SW => format!("sw x{rs2}, 0x{imm12:x}(x{rs1})"),
            _ => "uknown_STORE".to_string(),
        },

        Opcode::OpImm {
            imm12,
            rs1,
            funct3,
            rd,
        } => match funct3 {
            F3_OP_IMM_ADDI => format!("addi: x{rd}, x{rs1}, 0x{imm12:x} # ({imm12})"),
            _ => "uknown_OP_IMM".to_string(),
        },

        Opcode::System {
            csr,
            rs1,
            funct3,
            rd,
        } => {
            match funct3 {
                // TODO: convert csr to string name, e.g. "mhartid"
                F3_SYSTEM_CSRRS => format!("csrrs x{rd}, {}, x{rs1:x}", csr_name(csr)),
                _ => "uknown_SYSTEM".to_string(),
            }
        }

        Opcode::Uknown => "uknown instr".to_string(),
    }
}

// Get ABI register name
pub fn reg_idx2abi(r: u8) -> &'static str {
    match r {
        0 => "zero",
        1 => "ra",
        2 => "sp",
        3 => "gp",
        4 => "tp",
        5 => "t0",
        6 => "t1",
        7 => "t2",
        8 => "s0",
        9 => "s1",
        10 => "a0",
        11 => "a1",
        12 => "a2",
        13 => "a3",
        14 => "a4",
        15 => "a5",
        16 => "a6",
        17 => "a7",
        18 => "s2",
        19 => "s3",
        20 => "a4",
        21 => "s5",
        22 => "s6",
        23 => "s7",
        24 => "s8",
        25 => "s9",
        26 => "s10",
        27 => "s11",
        28 => "t3",
        29 => "t4",
        30 => "t5",
        31 => "t6",
        _ => panic!("Unknow register idx"),
    }
}

pub fn csr_name(csr: u16) -> &'static str {
    match csr {
        0xf14 => "mhartid",
        _ => "UKNOWN",
    }
}

/// Converts u32 to binary string. E.g.: 0x_1234_abcd to "0001_0010_0011_0100_1010_1011_1100_1101"
pub fn u32_bin4(v: u32) -> String {
    format!(
        "{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}",
        v.bits(31, 28),
        v.bits(27, 24),
        v.bits(23, 20),
        v.bits(19, 16),
        v.bits(15, 12),
        v.bits(11, 8),
        v.bits(7, 4),
        v.bits(3, 0)
    )
}

pub fn u32_hex4(v: u32) -> String {
    format!("{:04x}_{:04x}", v.bits(31, 16), v.bits(15, 0))
}

pub fn u64_hex4(v: u64) -> String {
    format!(
        "{:04x}_{:04x}_{:04x}_{:04x}",
        v.bits(63, 48),
        v.bits(47, 32),
        v.bits(31, 16),
        v.bits(15, 0)
    )
}

pub fn reg_hex(v: u64) -> String {
    u64_hex4(v)
}

#[test]
fn test_u32_bin4() {
    assert_eq!(
        u32_bin4(0x_1234_abcd),
        "0001_0010_0011_0100_1010_1011_1100_1101".to_string()
    );
    assert_eq!(
        u32_bin4(0x_ffff_a5a5),
        "1111_1111_1111_1111_1010_0101_1010_0101".to_string()
    )
}
