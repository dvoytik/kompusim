use core::fmt;

use crate::alu::{Imm, I12, I13, I21};
use crate::bits::BitOps;
use crate::csr;
use crate::pmem::Pmem;

//
#[derive(Debug, Default)]
pub struct RV64IUnprivRegs {
    //x0: is always zero
    x: [u64; 32],
    pub pc: u64,
}

impl fmt::Display for RV64IUnprivRegs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //let (x1, x2) = self;
        writeln!(f, " x1: 0x{:016x} | b'{0:064b}", self.x[1])?;
        writeln!(f, " x2: 0x{:016x} | b'{0:064b}", self.x[2])?;
        writeln!(f, " x3: 0x{:016x} | b'{0:064b}", self.x[3])?;
        writeln!(f, " x4: 0x{:016x} | b'{0:064b}", self.x[4])?;
        writeln!(f, " x5: 0x{:016x} | b'{0:064b}", self.x[5])?;
        writeln!(f, " x6: 0x{:016x} | b'{0:064b}", self.x[6])?;
        writeln!(f, " x7: 0x{:016x} | b'{0:064b}", self.x[7])?;
        writeln!(f, " x8: 0x{:016x} | b'{0:064b}", self.x[8])?;
        writeln!(f, " x9: 0x{:016x} | b'{0:064b}", self.x[8])?;
        writeln!(f, "x10: 0x{:016x} | b'{0:064b}", self.x[8])?;
        writeln!(f, " pc: 0x{:016x} | b'{0:064b}", self.pc)
    }
}

#[derive(Default)]
pub struct RV64ICpu {
    pub regs: RV64IUnprivRegs,
    pub mem: Pmem,
}

const OPC_SYSTEM: u8 = 0b11_100_11;
const OPC_BRANCH: u8 = 0b11_000_11;
const OPC_AUIPC: u8 = 0b00_101_11;
const OPC_OP_IMM: u8 = 0b00_100_11;
const OPC_JAL: u8 = 0b11_011_11;

const F3_BRANCH_BNE: u8 = 0b001;
const F3_SYSTEM_CSRRS: u8 = 0b010;

const F3_OP_IMM_ADDI: u8 = 0b000;

#[inline(always)]
fn i_opcode(ins: u32) -> u8 {
    ins.bits(6, 0) as u8
}

#[inline(always)]
fn i_funct3(ins: u32) -> u8 {
    ins.bits(14, 12) as u8
}

#[inline(always)]
fn i_rd(ins: u32) -> u8 {
    ins.bits(11, 7) as u8
}

#[inline(always)]
fn i_rs1(ins: u32) -> u8 {
    ins.bits(19, 15) as u8
}

#[inline(always)]
fn i_rs2(ins: u32) -> u8 {
    ins.bits(24, 20) as u8
}

#[inline(always)]
fn i_csr(ins: u32) -> u16 {
    ins.bits(31, 20) as u16
}

// Decode 13-bit signed offset from a B-type instruction
#[inline(always)]
fn i_b_off13(ins: u32) -> u16 {
    let off_4_1 = ins.bits(11, 8) as u16;
    let off_11 = ins.bits(7, 7) as u16;
    let off_10_5 = ins.bits(30, 25) as u16;
    let off_12 = ins.bits(31, 31) as u16;
    off_12 << 12 | off_11 << 11 | off_10_5 << 5 | off_4_1 << 1
}

// TODO: is it used only in one place?
fn i_u_uimm20(ins: u32) -> u32 {
    ins & 0xffff_f000
}

// Decode signed 12-bit immidiate from I-type instruction
#[inline(always)]
fn i_i_imm12(ins: u32) -> u16 {
    ins.bits(31, 20) as u16
}

impl RV64ICpu {
    pub fn new(mem: Pmem) -> RV64ICpu {
        RV64ICpu {
            mem,
            regs: RV64IUnprivRegs::default(),
        }
    }

    // reg_i - register index (0 - 31)
    fn regs_w64(&mut self, reg_i: u8, val: u64) {
        if reg_i == 0 {
            return; // writes to x0 are ignored
        }
        self.regs.x[reg_i as usize] = val;
    }

    fn regs_r64(&self, reg_i: u8) -> u64 {
        self.regs.x[reg_i as usize]
    }

    fn pc_inc(&mut self) {
        self.regs.pc += 4;
        println!("DBG: pc: 0x{:x} -> 0x{:x}", self.regs.pc - 4, self.regs.pc)
    }

    // Zics SYSTEM opcodes: CSRRS, ...
    fn opc_system(&mut self, ins: u32) {
        // I-type instruction
        let funct3 = i_funct3(ins);
        let rd = i_rd(ins);
        let rs1 = i_rs1(ins);
        let csr = i_csr(ins);
        println!(
            "DBG: SYSTEM: csr: {:x}, rs1: {:x}, f3: {:x}, rd: {:x}",
            csr, rs1, funct3, rd
        );
        match funct3 {
            F3_SYSTEM_CSRRS => {
                println!("DBG: CSRRS");
                let mut csr_v = csr::csr_r64(csr);
                self.regs_w64(rd, csr_v);
                csr_v |= self.regs_r64(rs1);
                csr::csr_w64(csr, csr_v);
            }
            _ => {
                panic!("wrong Zicsr instr")
            }
        }
        self.pc_inc();
    }

    // BRANCH opcodes: BNE, ...
    fn opc_branch(&mut self, ins: u32) {
        // B-type instructions
        let funct3 = i_funct3(ins);
        let rs1 = i_rs1(ins);
        let rs2 = i_rs2(ins);
        let off13 = i_b_off13(ins);
        println!(
            "DBG: BRANCH: imm[12:0]: 0x{off13:x}, rs2: {rs2}, \
             rs1: {rs1}, f3: 0x{funct3:x}"
        );
        match funct3 {
            F3_BRANCH_BNE => {
                println!("DBG: bne x{}, x{}, 0x{:x}", rs1, rs2, off13);
                // Branch Not Equal
                if self.regs_r64(rs1) != self.regs_r64(rs2) {
                    self.regs.pc = self.regs.pc.add_i13(I13::from_u16(off13));
                } else {
                    self.pc_inc()
                }
            }
            _ => {
                panic!("unsupported SYSTEM instr")
            }
        }
    }

    // Only one instruction AUIPC - Add Upper Immidiate to PC
    fn opc_auipc(&mut self, ins: u32) {
        let rd = i_rd(ins);
        let uimm20 = i_u_uimm20(ins);
        println!("DBG: AUIPC: uimm[31:12]: 0x{uimm20:x}, rd: {rd}");
        self.regs_w64(rd, self.regs.pc + uimm20 as u64);
        self.pc_inc()
    }

    fn opc_op_imm(&mut self, ins: u32) {
        // I-type instructions
        let imm12 = i_i_imm12(ins);
        let rs1 = i_rs1(ins);
        let funct3 = i_funct3(ins);
        let rd = i_rd(ins);
        match funct3 {
            // arithmetic overflow is ignored
            F3_OP_IMM_ADDI => {
                println!("DBG: addi: x{rd}, x{rs1}, 0x{imm12:x} # ({imm12})");
                self.regs_w64(rd, self.regs_r64(rs1).add_i12(I12::from_u16(imm12)));
            }
            _ => {
                panic!("unsupported OP-IMM instr")
            }
        }
        self.pc_inc()
    }

    // Only one instrucitn JAL - Jump and Link
    fn opc_jal(&mut self, ins: u32) {
        //
        let rd = i_rd(ins);
        let imm21 = ins.bits(31, 31) << 20
            | ins.bits(19, 12) << 12
            | ins.bits(20, 20) << 11
            | ins.bits(30, 21) << 1;
        println!("DBG: jal x{rd}, 0x{imm21:x} # {imm21}");
        self.regs_w64(rd, self.regs.pc + 4);
        self.regs.pc = self.regs.pc.add_i21(I21::from_u32(imm21));
    }

    fn execute_instr(&mut self, ins: u32) {
        // TODO: macro with bits matching
        println!("\nDBG: instr: 0x{:08x}", ins);
        let opcode = i_opcode(ins);
        match opcode {
            OPC_SYSTEM => self.opc_system(ins),
            OPC_BRANCH => self.opc_branch(ins),
            OPC_AUIPC => self.opc_auipc(ins),
            OPC_OP_IMM => self.opc_op_imm(ins),
            OPC_JAL => self.opc_jal(ins),
            _ => {
                panic!("not implemented opcode: 0x{:x}", opcode)
            }
        }
    }

    pub fn run_until(&mut self, break_point: u64) {
        println!("DBG: pc: 0x{:08x}", self.regs.pc);
        while self.regs.pc != break_point {
            let instr = self.mem.read32(self.regs.pc);
            self.execute_instr(instr);
        }
    }
}

#[test]
fn test_opcode_csrrs() {}

#[test]
fn test_opcode_bne() {
    let mut cpu = RV64ICpu::default();
    cpu.regs.x[5] = 1;
    // BNE t0, x0, 0x10
    cpu.execute_instr(0x00029863);
    assert!(cpu.regs.pc == 0x10);
}

#[test]
fn test_opcode_auipc() {
    let mut cpu = RV64ICpu::default();
    cpu.regs.pc = 0x100;
    cpu.regs.x[10] = 0x123;
    // auipc x10, 0x0
    cpu.execute_instr(0x00000517);
    assert!(cpu.regs.x[10] == 0x100);
}

#[test]
fn test_instr_decode_immidiates() {
    let imm12 = i_i_imm12(0xffff_ffff);
    assert!(I12::from_u16(imm12).0 == -1);

    let imm12 = i_i_imm12(0x800f_ffff);
    assert!(I12::from_u16(imm12).0 == -2048);

    let imm12 = i_i_imm12(0x0fff_ffff);
    assert!(imm12 == 255);
}

#[test]
// addi x10, x10, 52
fn test_opcode_addi() {
    let mut cpu = RV64ICpu::default();
    cpu.regs.x[10] = 0x123;
    cpu.execute_instr(0x03450513);
    assert!(cpu.regs.x[10] == 0x123 + 52);
}

#[test]
// jal ra, 80000018
fn test_opcode_jal() {
    let mut cpu = RV64ICpu::default();
    cpu.regs.x[5] = 1;
    cpu.regs.pc = 0x80000010;
    cpu.execute_instr(0x008000ef);
    assert!(cpu.regs.pc == 0x80000018);
}
