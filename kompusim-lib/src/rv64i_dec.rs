// RV64i decoder

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
    Jal {
        imm21: I21,
        rd: u8,
    },
    Jalr {
        imm12: I12,
        rs1: u8,
        rd: u8,
    },
    Lui {
        uimm20: u64,
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
    Uknown,
}

// TODO:
// #[repr(u8)]
// enum Opcodes {
//
// }
pub const OPC_SYSTEM: u8 = 0b11_100_11;
pub const OPC_BRANCH: u8 = 0b11_000_11;
pub const OPC_AUIPC: u8 = 0b00_101_11;
pub const OPC_OP_IMM: u8 = 0b00_100_11;
pub const OPC_JALR: u8 = 0b11_001_11;
pub const OPC_JAL: u8 = 0b11_011_11;
pub const OPC_LUI: u8 = 0b01_101_11;
pub const OPC_LOAD: u8 = 0b00_000_11;
pub const OPC_STORE: u8 = 0b01_000_11;

pub const F3_BRANCH_BEQ: u8 = 0b000;
pub const F3_BRANCH_BNE: u8 = 0b001;
pub const F3_BRANCH_BLT: u8 = 0b100;

pub const F3_SYSTEM_CSRRS: u8 = 0b010;

pub const F3_OP_IMM_ADDI: u8 = 0b000;

pub const F3_OP_LOAD_LB: u8 = 0b000;
pub const F3_OP_LOAD_LBU: u8 = 0b100;
pub const F3_OP_LOAD_LW: u8 = 0b010;

pub const F3_OP_STORE_SB: u8 = 0b000;
pub const F3_OP_STORE_SW: u8 = 0b010;

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
pub fn i_u_uimm20(ins: u32) -> u64 {
    (ins & 0xffff_f000) as u64
}

// Decode signed 12-bit immidiate from I-type instruction
#[inline(always)]
pub fn i_i_type_imm12(ins: u32) -> I12 {
    I12::from(ins.bits(31, 20) as u16)
}

// Decode signed 12-bit immidiate from S-type instruction
#[inline(always)]
pub fn i_s_type_imm12(ins: u32) -> I12 {
    let imm11_5 = ins.bits(31, 25) as u16;
    let imm4_0 = ins.bits(11, 7) as u16;
    I12::from(imm11_5 << 5 | imm4_0)
}

/// Decodes Zics SYSTEM opcodes: CSRRS, ...
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
        OPC_LUI => Opcode::Lui {
            uimm20: i_u_uimm20(instr),
            rd: i_rd(instr),
        },
        OPC_AUIPC => Opcode::Auipc {
            uimm20: i_u_uimm20(instr),
            rd: i_rd(instr),
        },
        OPC_JAL => dec_opc_jal(instr),
        OPC_JALR => dec_opc_jalr(instr),
        OPC_BRANCH => dec_opc_branch(instr),
        OPC_LOAD => dec_opc_load(instr),
        OPC_STORE => dec_opc_store(instr),
        OPC_OP_IMM => dec_opc_op_imm(instr),
        OPC_SYSTEM => dec_opc_system(instr),
        _ => Opcode::Uknown,
    }
}