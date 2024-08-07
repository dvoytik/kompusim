// Disassembler

use std::num::ParseIntError;

use crate::{alu::Imm, bits::BitOps, csr, rv64i_dec::*, rvc_dec::instr_is_rvc, rvc_disasm::*};

pub fn instr_hex(instr: u32) -> String {
    if instr_is_rvc(instr) {
        format!("{:04x}", instr as u16)
    } else {
        u32_hex4(instr)
    }
}

pub fn disasm_operation_name(instr: u32) -> String {
    if instr_is_rvc(instr) {
        return disasm_rvc_operation_name(instr as u16);
    }
    match decode_instr(instr) {
        Opcode::LUI { .. } => "Load Upper Immediate".to_string(),

        Opcode::Auipc { .. } => "Add Upper Immediate to PC".to_string(),

        // TODO:
        Opcode::Branch { funct3, .. } => match funct3 {
            F3_BRANCH_BNE => "Branch Not Equal".to_string(),
            F3_BRANCH_BEQ => "Branch EQual".to_string(),
            F3_BRANCH_BGE => "Branch if Greater or Equal (Signed comparison)".to_string(),
            F3_BRANCH_BGEU => "Branch if Greater or Equal (Unsigned comparison)".to_string(),
            F3_BRANCH_BLT => "Branch Less Than (signed comparison)".to_string(),
            F3_BRANCH_BLTU => "Branch Less Than (Unsgined comparison)".to_string(),
            _ => "Unknown BRANCH opcode".to_string(),
        },

        Opcode::Jal { .. } => "Jump And Link".to_string(),

        Opcode::Jalr { .. } => "Jump And Link Register".to_string(),

        Opcode::Load { funct3, .. } => match funct3 {
            F3_OP_LOAD_LB => "Load Byte (sign extend)".to_string(),
            F3_OP_LOAD_LBU => "Load Byte Unsigned".to_string(),
            F3_OP_LOAD_LW => "Load Word (sign extend)".to_string(),
            F3_OP_LOAD_LWU => "Load Word Unsigned".to_string(),
            F3_OP_LOAD_LD => "Load Double Word".to_string(),
            _ => "Unknown LOAD opcode".to_string(),
        },

        Opcode::Store { funct3, .. } => match funct3 {
            F3_OP_STORE_SB => "Store Byte".to_string(),
            F3_OP_STORE_SW => "Store Word".to_string(),
            F3_OP_STORE_SD => "Store Double Word".to_string(),
            _ => "Unknown STORE opcode".to_string(),
        },

        Opcode::OpImm { funct3, .. } => match funct3 {
            F3_OP_IMM_ADDI => "ADD Immediate".to_string(),
            F3_OP_IMM_SLTIU => "Set Less Than Immediate Unsigned".to_string(),
            F3_OP_IMM_XORI => "XOR Immediate".to_string(),
            F3_OP_IMM_ANDI => "AND Immediate".to_string(),
            F3_OP_IMM_SLLI => "Shift Left Logical Immediate".to_string(),
            _ => "Unknown OP-IMM opcode".to_string(),
        },

        Opcode::ADDIW { .. } => "ADD Word Immediate".to_string(),
        Opcode::SLLIW { .. } => "Shift Left Logical Immediate Word".to_string(),
        Opcode::SRLIW { .. } => "Shift Right Logical Immediate Word".to_string(),
        Opcode::Op { funct7, funct3, .. } => match (funct7, funct3) {
            (F7_OP_ADD, F3_OP_ADD_SUB) => "Add register to register".to_string(),
            (F7_OP_SUB, F3_OP_ADD_SUB) => "Subtract register from regiser".to_string(),
            _ => format!("Unknown OP instruction: funct7: {funct7:x}, funct3: {funct3:x}"),
        },

        Opcode::SUBW { .. } => "Subtract Word".to_string(),

        Opcode::Amo { funct5, funct3, .. } => match (funct5, funct3) {
            (F5_OP_AMO_SWAP, F3_OP_AMO_WORD) => "Atomic swap".to_string(),
            (F5_OP_AMO_ADD, F3_OP_AMO_WORD) => "Atomic Add".to_string(),
            (F5_OP_AMO_LRW, F3_OP_AMO_WORD) => "Load Reserve Word".to_string(),
            _ => format!("Unknown AMO instruction: funct5: {funct5:x}, funct3: {funct3:x}"),
        },

        Opcode::Fence { funct3, .. } => match funct3 {
            F3_OP_FENCE => "Fence - sync memory access order".to_string(),
            F3_OP_FENCE_I => "Sync I-cache with D-cache".to_string(),
            _ => format!("Unknown FENCE instruction: funct3: 0b_{funct3:b}"),
        },

        Opcode::System { funct3, .. } => match funct3 {
            F3_SYSTEM_WFI => "Wait For Interrupt".to_string(),
            F3_SYSTEM_CSRRS => "Control Status Register - Read, Set bitmask".to_string(),
            F3_SYSTEM_CSRRWI => "Control Status Register - Read, Write Immediate".to_string(),
            F3_SYSTEM_CSRRW => "Control Status Register - Read, Write".to_string(),
            _ => "Unknown SYSTEM opcode".to_string(),
        },

        Opcode::Uknown => "Unknown Operation".to_string(),
    }
}

pub fn disasm_pseudo_code(instr: u32) -> String {
    if instr_is_rvc(instr) {
        return disasm_rvc_pseudo_code(instr as u16);
    }
    match decode_instr(instr) {
        // TODO:
        Opcode::LUI { uimm20, rd } => format!("x{rd} = 0x{:x} << 12", uimm20 >> 12),

        // TODO:
        Opcode::Auipc { uimm20, rd } => format!("x{rd} = PC + 0x{uimm20:x} << 12"),

        Opcode::Branch {
            off13,
            rs2,
            rs1,
            funct3,
        } => {
            match funct3 {
                // Branch Not Equal
                F3_BRANCH_BNE => format!("if x{rs1} != x{rs2} then PC = PC {:+}", off13.0),
                // Branch EQual
                F3_BRANCH_BEQ => format!("if x{rs1} == x{rs2} then PC = PC {:+}", off13.0),
                // Branch Less Than (signed comparison)
                F3_BRANCH_BLT => format!("if x{rs1} < x{rs2} then PC = PC {:+}", off13.0),
                // Branch Less Than (Unsigned comparison)
                F3_BRANCH_BLTU => format!("if x{rs1} < x{rs2} then PC = PC {:+}", off13.0),
                // Branch if Greater or Equal (signed comparison)
                F3_BRANCH_BGE => format!("if x{rs1} >= x{rs2} then PC = PC {:+}", off13.0),
                // Branch if Greater or Equal (Unsigned comparison)
                F3_BRANCH_BGEU => format!("if x{rs1} >= x{rs2} then PC = PC {:+}", off13.0),
                _ => "Unknown BRANCH opcode".to_string(),
            }
        }

        Opcode::Jal { imm21, rd } => {
            format!(
                "x{rd} = PC + 4; PC = PC {:+}",
                imm21.0 // instr_addr.add_i21(imm21)
            )
        }

        Opcode::Jalr { imm12, rs1, rd } => {
            format!("x{rd} = PC + 4; PC = x{rs1} {:+}; PC[0] = 0", imm12.0)
        }

        Opcode::Load {
            imm12,
            rs1,
            funct3,
            rd,
        } => match funct3 {
            F3_OP_LOAD_LB => {
                format!("x{rd}[7:0] = m8[x{rs1} {:+}]; s-ext", imm12.0)
            }
            F3_OP_LOAD_LBU => {
                format!("x{rd}[7:0] = m8[x{rs1} {:+})]; z-ext", imm12.0)
            }
            F3_OP_LOAD_LW => {
                format!("x{rd}[31:0] = m32[x{rs1} {:+}]; s-ext", imm12.0)
            }
            F3_OP_LOAD_LWU => {
                format!("x{rd}[31:0] = m32[x{rs1} {:+}]; z-ext", imm12.0)
            }
            F3_OP_LOAD_LD => format!("x{rd} = m64[x{rs1} {:+}]", imm12.0),
            _ => "Unknown LOAD opcode".to_string(),
        },

        Opcode::Store {
            imm12,
            rs2,
            rs1,
            funct3,
        } => match funct3 {
            F3_OP_STORE_SB => format!("m8[x{rs1} {:+}] = x{rs2}[7:0]", imm12.0),
            F3_OP_STORE_SW => format!("m32[x{rs1} {:+}] = x{rs2}[31:0]", imm12.0),
            F3_OP_STORE_SD => format!("m64[x{rs1} {:+}] = x{rs2}", imm12.0),
            _ => "Unknown STORE opcode".to_string(),
        },

        Opcode::OpImm {
            imm12,
            rs1,
            funct3,
            rd,
        } => match funct3 {
            F3_OP_IMM_ADDI => format!("x{rd} = x{rs1} {:+}", imm12.0),
            F3_OP_IMM_SLTIU => format!("If x{rs1} < {:+} then x{rd} = 1 else x{rd} = 0", imm12.0),
            F3_OP_IMM_XORI => format!("x{rd} = x{rs1} ^ 0x{imm12:x}"),
            F3_OP_IMM_ANDI => format!("x{rd} = x{rs1} & 0x{imm12:x}"),
            F3_OP_IMM_SLLI => format!("x{rd} = x{rs1} << {imm12}"),
            _ => "Unknown OP-IMM opcode".to_string(),
        },

        Opcode::ADDIW { imm12, rs1, rd } => format!("x{rd}[31:0] = x{rs1}[31:0] + 0x{imm12:x}"),
        Opcode::SLLIW { shamt, rs1, rd } => {
            format!("x{rd}[31:0] = x{rs1}[31:0] << {shamt}; s-ext")
        }
        Opcode::SRLIW { shamt, rs1, rd } => {
            format!("x{rd}[31:0] = x{rs1}[31:0] >> {shamt}; sign extend")
        }
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

        Opcode::SUBW { rs2, rs1, rd } => {
            format!("x{rd}[31:0] = x{rs1}[31:0] - x{rs2}[31:0]; s-ext")
        }

        Opcode::Amo {
            funct5,
            aq,
            rl,
            rs2,
            rs1,
            funct3,
            rd,
        } => match (funct5, funct3) {
            (F5_OP_AMO_SWAP, F3_OP_AMO_WORD) => {
                format!(
                    "x{rd} <= m32[x{rs1}]; m32[x{rs1}] <= x{rs2}{}{}",
                    if aq { "; acquire" } else { "" },
                    if rl { "; release" } else { "" }
                )
            }
            (F5_OP_AMO_ADD, F3_OP_AMO_WORD) => {
                format!(
                    "x{rd} <= m32[x{rs1}]; m32[x{rs1}] <= rd + x{rs2}{}{}",
                    if aq { "; acquire" } else { "" },
                    if rl { "; release" } else { "" }
                )
            }
            (F5_OP_AMO_LRW, F3_OP_AMO_WORD) => format!("x{rd} = m32[x{rs1}] ; todo"),
            _ => format!("Unknown AMO instruction: funct5: {funct5:x}, funct3: {funct3:x}"),
        },

        Opcode::System {
            csr,
            rs1,
            funct3,
            rd,
        } => match funct3 {
            F3_SYSTEM_WFI => "no effect".to_string(),
            F3_SYSTEM_CSRRS => format!(
                "x{rd} = {csrn}; {csrn} = {csrn} | x{rs1:b}",
                csrn = csr_name(csr)
            ),
            F3_SYSTEM_CSRRWI => format!("x{rd} = {csrn}; {csrn} = 0x{rs1:x}", csrn = csr_name(csr)),
            F3_SYSTEM_CSRRW => format!("x{rd} = {csrn}; {csrn} = x{rs1}", csrn = csr_name(csr)),
            _ => "Unknown SYSTEM opcode".to_string(),
        },

        Opcode::Fence { imm12, funct3 } => match funct3 {
            F3_OP_FENCE => format!(
                "fence: FM:0b_{:b}, PI:{}, PO:{}, PR:{}, PW:{}, SI:{}, SO:{}, SR:{}, SW:{}",
                imm12.0.bits(11, 8),
                imm12.0.bits(7, 7),
                imm12.0.bits(6, 6),
                imm12.0.bits(5, 5),
                imm12.0.bits(4, 4),
                imm12.0.bits(3, 3),
                imm12.0.bits(2, 2),
                imm12.0.bits(1, 1),
                imm12.0.bits(0, 0)
            ),
            F3_OP_FENCE_I => ("fence: sync I-cache").to_string(),
            _ => "Unknown FENCE instruction".to_string(),
        },

        Opcode::Uknown => "Unknown instruction".to_string(),
    }
}

/// Returns used registers indexes (rs1, rs2, rd)
pub fn disasm_get_used_regs(instr: u32) -> (Option<u8>, Option<u8>, Option<u8>) {
    if instr_is_rvc(instr) {
        return disasm_rvc_get_used_regs(instr as u16);
    }
    match decode_instr(instr) {
        Opcode::LUI { rd, .. } => (None, None, Some(rd)),
        Opcode::Auipc { rd, .. } => (None, None, Some(rd)),
        Opcode::Branch { rs2, rs1, .. } => (Some(rs1), Some(rs2), None),
        Opcode::Jal { rd, .. } => (None, None, Some(rd)),
        Opcode::Jalr { rs1, rd, .. } => (Some(rs1), None, Some(rd)),
        Opcode::Load { rs1, rd, .. } => (Some(rs1), None, Some(rd)),
        Opcode::Store { rs2, rs1, .. } => (Some(rs1), Some(rs2), None),
        Opcode::OpImm { rs1, rd, .. } => (Some(rs1), None, Some(rd)),
        Opcode::ADDIW { rs1, rd, .. } => (Some(rs1), None, Some(rd)),
        Opcode::SLLIW { rs1, rd, .. } => (Some(rs1), None, Some(rd)),
        Opcode::SRLIW { rs1, rd, .. } => (Some(rs1), None, Some(rd)),
        Opcode::SUBW { rs2, rs1, rd } => (Some(rs1), Some(rs2), Some(rd)),
        Opcode::Op { rs2, rs1, rd, .. } => (Some(rs1), Some(rs2), Some(rd)),
        Opcode::Amo { rs2, rs1, rd, .. } => (Some(rs1), Some(rs2), Some(rd)),
        Opcode::System {
            rs1, rd, funct3, ..
        } => {
            if funct3 == F3_SYSTEM_WFI {
                (None, None, None)
            } else {
                (Some(rs1), None, Some(rd))
            }
        }
        Opcode::Fence { .. } => (None, None, None),
        Opcode::Uknown => (None, None, None),
    }
}

pub fn disasm(instr: u32, instr_addr: u64) -> String {
    if instr_is_rvc(instr) {
        return disasm_rvc(instr as u16, instr_addr);
    }
    match decode_instr(instr) {
        Opcode::LUI { uimm20, rd } => format!("lui x{rd}, 0x{:x}", uimm20 >> 12),

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
                // Branch Less Than (Unsigned comparison)
                F3_BRANCH_BLTU => format!("bltu x{rs1}, x{rs2}, 0x{addr:x}"),
                // Branch if Greater or Equal (signed comparison)
                F3_BRANCH_BGE => format!("bge x{rs1}, x{rs2}, 0x{addr:x}"),
                // Branch if Greater or Equal (Unsigned comparison)
                F3_BRANCH_BGEU => format!("bgeu x{rs1}, x{rs2}, 0x{addr:x}"),
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
            F3_OP_LOAD_LB => format!("lb x{rd}, {imm12}(x{rs1})"),
            F3_OP_LOAD_LBU => format!("lbu x{rd}, {imm12}(x{rs1})"),
            F3_OP_LOAD_LW => format!("lw x{rd}, {imm12}(x{rs1})"),
            F3_OP_LOAD_LWU => format!("lwu x{rd}, {imm12}(x{rs1})"),
            F3_OP_LOAD_LD => format!("ld x{rd}, {imm12}(x{rs1})"),
            _ => "Unknown LOAD opcode".to_string(),
        },

        Opcode::Store {
            imm12,
            rs2,
            rs1,
            funct3,
        } => match funct3 {
            F3_OP_STORE_SB => format!("sb x{rs2}, {imm12}(x{rs1})"),
            F3_OP_STORE_SW => format!("sw x{rs2}, {imm12}(x{rs1})"),
            F3_OP_STORE_SD => format!("sd x{rs2}, {imm12}(x{rs1})"),
            _ => "Unknown STORE opcode".to_string(),
        },

        Opcode::OpImm {
            imm12,
            rs1,
            funct3,
            rd,
        } => match funct3 {
            F3_OP_IMM_ADDI => format!("addi x{rd}, x{rs1}, 0x{imm12:x}"),
            F3_OP_IMM_SLTIU => format!("sltiu x{rd}, x{rs1}, {imm12}"),
            F3_OP_IMM_XORI => format!("xori x{rd}, x{rs1}, {imm12}"),
            F3_OP_IMM_ANDI => format!("andi x{rd}, x{rs1}, {imm12}"),
            F3_OP_IMM_SLLI => format!("slli x{rd}, x{rs1}, 0x{imm12:x}"),
            _ => "Unknown OP-IMM opcode".to_string(),
        },

        Opcode::ADDIW { imm12, rs1, rd } => format!("addiw x{rd}, x{rs1}, 0x{imm12:x}"),

        Opcode::SLLIW { shamt, rs1, rd } => format!("slliw x{rd}, x{rs1}, 0x{shamt:x}"),
        Opcode::SRLIW { shamt, rs1, rd } => format!("srliw x{rd}, x{rs1}, 0x{shamt:x}"),
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

        Opcode::SUBW { rs2, rs1, rd } => format!("subw x{rd}, x{rs1}, x{rs2}"),

        Opcode::Amo {
            funct5,
            aq,
            rl,
            rs2,
            rs1,
            funct3,
            rd,
        } => match (funct5, funct3) {
            (F5_OP_AMO_SWAP, F3_OP_AMO_WORD) => {
                format!(
                    "amoswap.w{}{} x{rd}, x{rs2}, x{rs1}",
                    if aq { ".aq" } else { "" },
                    if rl { ".rl" } else { "" }
                )
            }
            (F5_OP_AMO_ADD, F3_OP_AMO_WORD) => {
                format!(
                    "amoadd.w{}{} x{rd}, x{rs2}, x{rs1}",
                    if aq { ".aq" } else { "" },
                    if rl { ".rl" } else { "" }
                )
            }
            (F5_OP_AMO_LRW, F3_OP_AMO_WORD) => format!("lr.w x{rd}, (x{rs1})"),
            _ => format!("Uknown AMO instruction: funct5: {funct5:x}, funct3: {funct3:x}"),
        },

        Opcode::System {
            csr,
            rs1,
            funct3,
            rd,
        } => match funct3 {
            F3_SYSTEM_WFI => "wfi".to_string(),
            F3_SYSTEM_CSRRS => format!("csrrs x{rd}, {}, x{rs1}", csr_name(csr)),
            F3_SYSTEM_CSRRWI => format!("csrrwi x{rd}, {}, {rs1:x}", csr_name(csr)),
            F3_SYSTEM_CSRRW => format!("csrrw x{rd}, {}, x{rs1}", csr_name(csr)),
            _ => "Unknown SYSTEM opcode".to_string(),
        },

        Opcode::Fence { imm12, funct3 } => match funct3 {
            F3_OP_FENCE => format!(
                "fence{} {}{}{}{}, {}{}{}{}",
                match imm12.0.bits(11, 8) {
                    0b_0000 => "",
                    0b_1000 => ".TSO",
                    _ => ".UKNOWN",
                },
                if imm12.0.bit(7) { "i" } else { "" },
                if imm12.0.bit(6) { "o" } else { "" },
                if imm12.0.bit(5) { "r" } else { "" },
                if imm12.0.bit(4) { "w" } else { "" },
                if imm12.0.bit(3) { "i" } else { "" },
                if imm12.0.bit(2) { "o" } else { "" },
                if imm12.0.bit(1) { "r" } else { "" },
                if imm12.0.bit(0) { "w" } else { "" },
            ),
            F3_OP_FENCE_I => format!("fence.i 0x{imm12:x}"),
            _ => "Unknown FENCE instruction".to_string(),
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
        20 => "s4",
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
        csr::MTVEC => "mtvec",
        csr::MHARTID => "mhartid",
        csr::MSCRATCH => "mscratch",
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

/// Converts u16 to binary string. E.g.: 0x_abcd to "1010_1011_1100_1101"
pub fn u16_bin4(v: u16) -> String {
    format!(
        "{:04b}_{:04b}_{:04b}_{:04b}",
        v.bits(15, 12),
        v.bits(11, 8),
        v.bits(7, 4),
        v.bits(3, 0)
    )
}

pub fn instr_bin4(instr: u32) -> String {
    if instr_is_rvc(instr) {
        u16_bin4(instr as u16)
    } else {
        u32_bin4(instr)
    }
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

#[test]
fn test_disasm() {
    assert_eq!(disasm(0x_fcc4_6783, 0x0), "lwu x15, -52(x8)");
    assert_eq!(disasm(0x_0002_b303, 0x0), "ld x6, 0(x5)");
    assert_eq!(disasm(0x_fd84_3783, 0x0), "ld x15, -40(x8)");
    assert_eq!(disasm(0x_fef4_3423, 0x0), "sd x15, -24(x8)");
    assert_eq!(disasm(0x_0330_000f, 0x0), "fence rw, rw");
    assert_eq!(disasm(0x_3400_5073, 0x0), "csrrwi x0, mscratch, 0");
    assert_eq!(disasm(0x_3052_10f3, 0x0), "csrrw x1, mtvec, x4");
    assert_eq!(disasm(0x_0187_979b, 0x0), "slliw x15, x15, 0x18");
    assert_eq!(disasm(0x_01f6_d71b, 0x0), "srliw x14, x13, 0x1f");
    assert_eq!(disasm(0x_0007_d563, 0x1f30a), "bge x15, x0, 0x1f314");
    assert_eq!(
        disasm(0x_04e7_f563, 0x_8001_dd5a),
        "bgeu x15, x14, 0x8001dda4"
    );
    assert_eq!(
        disasm(0x_00e7_e463, 0x_8001_dd3e),
        "bltu x15, x14, 0x8001dd46"
    );
    assert_eq!(disasm(0x_40f7_07bb, 0x0), "subw x15, x14, x15");
    assert_eq!(disasm(0x_0027_9713, 0x0), "slli x14, x15, 0x2");
    assert_eq!(disasm(0x_8006b713, 0x0), "sltiu x14, x13, -2048");
    assert_eq!(disasm(0x_f0f6c713, 0x0), "xori x14, x13, -241");
    assert_eq!(disasm(0x_70f6_f713, 0x0), "andi x14, x13, 1807");
    assert_eq!(disasm(0x_1050_0073, 0x0), "wfi");
}
