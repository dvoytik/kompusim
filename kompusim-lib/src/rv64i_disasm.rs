// Disassembler

use std::num::ParseIntError;

use crate::{alu::Imm, bits::BitOps, rv64i_dec::*, rvc_dec::instr_is_16b, rvc_disasm::*};

pub fn disasm_operation_name(instr: u32) -> String {
    if instr_is_16b(instr) {
        return disasm_16b_operation_name(instr as u16);
    }
    match decode_instr(instr) {
        Opcode::Lui { .. } => "Load Upper Immediate".to_string(),

        Opcode::Auipc { .. } => "Add Upper Immediate to PC".to_string(),

        // TODO:
        Opcode::Branch { funct3, .. } => match funct3 {
            F3_BRANCH_BNE => "Branch Not Equal".to_string(),
            F3_BRANCH_BEQ => "Branch EQual".to_string(),
            F3_BRANCH_BLT => "Branch Less Than (signed comparison)".to_string(),
            _ => "Unknown BRANCH opcode".to_string(),
        },

        Opcode::Jal { .. } => "Jump And Link".to_string(),

        Opcode::Jalr { .. } => "Jump And Link Register".to_string(),

        Opcode::Load { funct3, .. } => match funct3 {
            F3_OP_LOAD_LB => "Load Byte (sign extend)".to_string(),
            F3_OP_LOAD_LBU => "Load Byte Unsigned".to_string(),
            F3_OP_LOAD_LW => "Load Word (sign extend)".to_string(),
            _ => "Unknown LOAD opcode".to_string(),
        },

        Opcode::Store { funct3, .. } => match funct3 {
            F3_OP_STORE_SB => "Store Byte".to_string(),
            F3_OP_STORE_SW => "Store Word".to_string(),
            _ => "Unknown STORE opcode".to_string(),
        },

        Opcode::OpImm { funct3, .. } => match funct3 {
            F3_OP_IMM_ADDI => "ADD Immediate".to_string(),
            _ => "Unknown OP-IMM opcode".to_string(),
        },

        Opcode::Op { funct7, funct3, .. } => match (funct7, funct3) {
            (F7_OP_ADD, F3_OP_ADD_SUB) => format!("Add register to register"),
            (F7_OP_SUB, F3_OP_ADD_SUB) => format!("Subtract register from regiser"),
            _ => format!("Unknown OP instruction: funct7: {funct7:x}, funct3: {funct3:x}"),
        },

        Opcode::System { funct3, .. } => match funct3 {
            F3_SYSTEM_CSRRS => "Control Status Register - Read, Set bitmask".to_string(),
            _ => "Unknown SYSTEM opcode".to_string(),
        },

        Opcode::Uknown => "Unknown Operation".to_string(),
    }
}

pub fn disasm_pseudo_code(instr: u32, _instr_addr: u64) -> String {
    if instr_is_16b(instr) {
        return disasm_16b_pseudo_code(instr as u16);
    }
    match decode_instr(instr) {
        // TODO:
        Opcode::Lui { uimm20, rd } => format!("x{rd} = 0x{:x} << 12", uimm20 >> 12),

        // TODO:
        Opcode::Auipc { uimm20, rd } => format!("x{rd} = PC + 0x{uimm20:x} << 12"),

        Opcode::Branch {
            off13,
            rs2,
            rs1,
            funct3,
        } => {
            // let addr = instr_addr.add_i13(off13);
            match funct3 {
                // Branch Not Equal
                F3_BRANCH_BNE => format!("if x{rs1} != x{rs2} then PC = PC + 0x{off13:x}"),
                // Branch EQual
                F3_BRANCH_BEQ => format!("if x{rs1} == x{rs2} then PC = PC + 0x{off13:x}"),
                // Branch Less Than (signed comparison)
                F3_BRANCH_BLT => format!("if x{rs1} < x{rs2} then PC = PC + 0x{off13:x}"),
                _ => "Unknown BRANCH opcode".to_string(),
            }
        }

        Opcode::Jal { imm21, rd } => {
            format!(
                "x{rd} = PC + 4; PC = PC + 0x{imm21:x}",
                // instr_addr.add_i21(imm21)
            )
        }

        Opcode::Jalr { imm12, rs1, rd } => {
            format!("x{rd} = PC + 4; PC = x{rs1} + 0x{imm12:x}; PC[0] = 0")
        }

        Opcode::Load {
            imm12,
            rs1,
            funct3,
            rd,
        } => match funct3 {
            F3_OP_LOAD_LB => {
                format!("x{rd}[7:0] = mem8[x{rs1} + sign_ext(0x{imm12})]; x{rd}[63:8] = x{rd}[7]")
            }
            F3_OP_LOAD_LBU => {
                format!("x{rd}[7:0] = mem8[x{rs1} + sign_ext(0x{imm12})]; x{rd}[63:8] = 0")
            }
            F3_OP_LOAD_LW => {
                format!(
                    "x{rd}[31:0] = mem32[x{rs1} + sign_ext(0x{imm12})]; x{rd}[63:32] = x{rd}[31]"
                )
            }
            _ => "Unknown LOAD opcode".to_string(),
        },

        Opcode::Store {
            imm12,
            rs2,
            rs1,
            funct3,
        } => match funct3 {
            F3_OP_STORE_SB => format!("mem8[x{rs1} + sign_extend(0x{imm12:x})] = x{rs2}[7:0]"),
            F3_OP_STORE_SW => format!("mem32[x{rs1} + sign_extend(0x{imm12:x})] = x{rs2}[31:0]"),
            _ => "Unknown STORE opcode".to_string(),
        },

        Opcode::OpImm {
            imm12,
            rs1,
            funct3,
            rd,
        } => match funct3 {
            F3_OP_IMM_ADDI => format!("x{rd} = x{rs1} + 0x{imm12:x}"),
            _ => "Unknown OP-IMM opcode".to_string(),
        },

        Opcode::Op {
            funct7,
            rs2,
            rs1,
            funct3,
            rd,
        } => match (funct7, funct3) {
            (F7_OP_ADD, F3_OP_ADD_SUB) => format!("x{rd} = x{rs1} + x{rs2}"),
            (F7_OP_SUB, F3_OP_ADD_SUB) => format!("x{rd} = x{rs1} - x{rs2}"),
            _ => format!("Unknown OP instruction: funct7: {funct7:x}, funct3: {funct3:x}"),
        },

        Opcode::System {
            csr,
            rs1,
            funct3,
            rd,
        } => match funct3 {
            F3_SYSTEM_CSRRS => format!(
                "x{rd} = {csrn}; {csrn} = {csrn} | x{rs1:b}",
                csrn = csr_name(csr)
            ),
            _ => "Unknown SYSTEM opcode".to_string(),
        },

        Opcode::Uknown => "Unknown instruction".to_string(),
    }
}

/// Returns used registers indexes (rs1, rs2, rd)
pub fn disasm_get_used_regs(instr: u32) -> (Option<u8>, Option<u8>, Option<u8>) {
    if instr_is_16b(instr) {
        return disasm_16b_get_used_regs(instr as u16);
    }
    match decode_instr(instr) {
        Opcode::Lui { rd, .. } => (None, None, Some(rd)),
        Opcode::Auipc { rd, .. } => (None, None, Some(rd)),
        Opcode::Branch { rs2, rs1, .. } => (Some(rs1), Some(rs2), None),
        Opcode::Jal { rd, .. } => (None, None, Some(rd)),
        Opcode::Jalr { rs1, rd, .. } => (Some(rs1), None, Some(rd)),
        Opcode::Load { rs1, rd, .. } => (Some(rs1), None, Some(rd)),
        Opcode::Store { rs2, rs1, .. } => (Some(rs1), Some(rs2), None),
        Opcode::OpImm { rs1, rd, .. } => (Some(rs1), None, Some(rd)),
        Opcode::Op { rs2, rs1, rd, .. } => (Some(rs1), Some(rs2), Some(rd)),
        Opcode::System { rs1, rd, .. } => (Some(rs1), None, Some(rd)),
        Opcode::Uknown => (None, None, None),
    }
}

pub fn disasm(instr: u32, instr_addr: u64) -> String {
    if instr_is_16b(instr) {
        return disasm_16b(instr as u16, instr_addr);
    }
    match decode_instr(instr) {
        Opcode::Lui { uimm20, rd } => format!("lui x{rd}, 0x{:x}", uimm20 >> 12),

        Opcode::Auipc { uimm20, rd } => format!("auipc x{rd}, 0x{uimm20:x}"),

        Opcode::Branch {
            off13,
            rs2,
            rs1,
            funct3,
        } => {
            let addr = instr_addr.add_i13(off13);
            match funct3 {
                // Branch Not Equal
                F3_BRANCH_BNE => format!("bne x{rs1}, x{rs2}, 0x{addr:x}"),
                // Branch EQual
                F3_BRANCH_BEQ => format!("beq x{rs1}, x{rs2}, 0x{addr:x}"),
                // Branch Less Than (signed comparison)
                F3_BRANCH_BLT => format!("blt x{rs1}, x{rs2}, 0x{addr:x}"),
                _ => "Unknown BRANCH opcode".to_string(),
            }
        }

        Opcode::Jal { imm21, rd } => {
            format!("jal x{rd}, 0x{0:x}", instr_addr.add_i21(imm21))
        }

        Opcode::Jalr { imm12, rs1, rd } => {
            format!("jalr x{rd}, 0x{imm12:x}(x{rs1})")
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
            _ => "Unknown LOAD opcode".to_string(),
        },

        Opcode::Store {
            imm12,
            rs2,
            rs1,
            funct3,
        } => match funct3 {
            F3_OP_STORE_SB => format!("sb x{rs2}, 0x{imm12:x}(x{rs1})"),
            F3_OP_STORE_SW => format!("sw x{rs2}, 0x{imm12:x}(x{rs1})"),
            _ => "Unknown STORE opcode".to_string(),
        },

        Opcode::OpImm {
            imm12,
            rs1,
            funct3,
            rd,
        } => match funct3 {
            F3_OP_IMM_ADDI => format!("addi x{rd}, x{rs1}, 0x{imm12:x}"),
            _ => "Unknown OP-IMM opcode".to_string(),
        },

        Opcode::Op {
            funct7,
            rs2,
            rs1,
            funct3,
            rd,
        } => match (funct7, funct3) {
            (F7_OP_ADD, F3_OP_ADD_SUB) => format!("add x{rd}, x{rs1}, x{rs2}"),
            (F7_OP_SUB, F3_OP_ADD_SUB) => format!("sub x{rd}, x{rs1}, x{rs2}"),
            _ => format!("Unknown OP instruction: funct7: {funct7:x}, funct3: {funct3:x}"),
        },

        Opcode::System {
            csr,
            rs1,
            funct3,
            rd,
        } => match funct3 {
            F3_SYSTEM_CSRRS => format!("csrrs x{rd}, {}, x{rs1:x}", csr_name(csr)),
            _ => "Unknown SYSTEM opcode".to_string(),
        },

        Opcode::Uknown => "Unknown instruction".to_string(),
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
        _ => panic!("Unknown register idx"),
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

/// Convert a hex string (e.g, "0x1234_9393") to u32
pub fn hex_to_u32(hex_str: &str) -> Result<u32, ParseIntError> {
    let no_prefix = hex_str.trim_start_matches("0x");
    if no_prefix.find('_').is_some() {
        // allocate new String and remove '_'
        let mut str = no_prefix.to_owned();
        str.retain(|c| c != '_');
        return u32::from_str_radix(&str, 16);
    }
    u32::from_str_radix(no_prefix, 16)
}

/// Convert a hex string (e.g, "0x1234_9393") to u64
pub fn hex_to_u64(hex_str: &str) -> Result<u64, ParseIntError> {
    let no_prefix = hex_str.trim_start_matches("0x");
    if no_prefix.find('_').is_some() {
        // allocate new String and remove '_'
        let mut str = no_prefix.to_owned();
        str.retain(|c| c != '_');
        return u64::from_str_radix(&str, 16);
    }
    u64::from_str_radix(no_prefix, 16)
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
    );
    assert_eq!(hex_to_u64("0x1234abcd").unwrap(), 0x1234_abcd);
    assert_eq!(hex_to_u64("0x1234_abcd").unwrap(), 0x1234_abcd);
    assert_eq!(
        hex_to_u64("0x0101_1234_abcd0000").unwrap(),
        0x0101_1234_abcd_0000
    );
    assert_eq!(hex_to_u64("__1__").unwrap(), 0x0000_0000_0000_0001);
}
