use core::fmt;

use crate::alu::{Imm, I12, I13, I21};
use crate::bits::BitOps;
use crate::bus::Bus;
use crate::csr;

//
#[derive(Debug, Default)]
pub struct RV64IUnprivRegs {
    // x0: is always zero
    x:      [u64; 32],
    pub pc: u64,
}

impl fmt::Display for RV64IUnprivRegs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, " x1 (ra): 0x{:016x} | b'{0:064b}", self.x[1])?;
        writeln!(f, " x2 (sp): 0x{:016x} | b'{0:064b}", self.x[2])?;
        writeln!(f, " x3 (gp): 0x{:016x} | b'{0:064b}", self.x[3])?;
        writeln!(f, " x4 (tp): 0x{:016x} | b'{0:064b}", self.x[4])?;
        writeln!(f, " x5 (t0): 0x{:016x} | b'{0:064b}", self.x[5])?;
        writeln!(f, " x6 (t1): 0x{:016x} | b'{0:064b}", self.x[6])?;
        writeln!(f, " x7 (t2): 0x{:016x} | b'{0:064b}", self.x[7])?;
        writeln!(f, " x8 (s0): 0x{:016x} | b'{0:064b}", self.x[8])?;
        writeln!(f, " x9 (s1): 0x{:016x} | b'{0:064b}", self.x[9])?;
        writeln!(f, "x10 (a0): 0x{:016x} | b'{0:064b}", self.x[10])?;
        writeln!(f, "x11 (a1): 0x{:016x} | b'{0:064b}", self.x[11])?;
        writeln!(f, "      pc: 0x{:016x} | b'{0:064b}", self.pc)
    }
}

// TODO: make regs private?
#[derive(Default)]
pub struct RV64ICpu {
    pub regs: RV64IUnprivRegs,
    pub bus:  Bus,
}

// TODO:
// #[repr(u8)]
// enum Opcodes {
//
// }
const OPC_SYSTEM: u8 = 0b11_100_11;
const OPC_BRANCH: u8 = 0b11_000_11;
const OPC_AUIPC: u8 = 0b00_101_11;
const OPC_OP_IMM: u8 = 0b00_100_11;
const OPC_JAL: u8 = 0b11_011_11;
const OPC_LUI: u8 = 0b01_101_11;
const OPC_LOAD: u8 = 0b00_000_11;

const F3_BRANCH_BEQ: u8 = 0b000;
const F3_BRANCH_BNE: u8 = 0b001;

const F3_SYSTEM_CSRRS: u8 = 0b010;

const F3_OP_IMM_ADDI: u8 = 0b000;

const F3_OP_LOAD_LB: u8 = 0b000;
const F3_OP_LOAD_LBU: u8 = 0b100;
const F3_OP_LOAD_LW: u8 = 0b010;

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
fn i_b_off13(ins: u32) -> I13 {
    let off_4_1 = ins.bits(11, 8) as u16;
    let off_11 = ins.bits(7, 7) as u16;
    let off_10_5 = ins.bits(30, 25) as u16;
    let off_12 = ins.bits(31, 31) as u16;
    I13::from_u16(off_12 << 12 | off_11 << 11 | off_10_5 << 5 | off_4_1 << 1)
}

// extract upper 20-bit for LUI, AUIPC instructions
fn i_u_uimm20(ins: u32) -> u64 {
    (ins & 0xffff_f000) as u64
}

// Decode signed 12-bit immidiate from I-type instruction
#[inline(always)]
fn i_i_imm12(ins: u32) -> u16 {
    ins.bits(31, 20) as u16
}

fn bad_instr(ins: u32) {
    let opc = i_opcode(ins);
    panic!("DBG: bad instr: 0x{ins:x} (0b_{ins:b}), opcode: 0x{opc:x} (0b_{opc:07b})");
}

impl RV64ICpu {
    pub fn new(bus: Bus) -> RV64ICpu {
        RV64ICpu { bus,
                   regs: RV64IUnprivRegs::default() }
    }

    // reg_i - register index (0 - 31)
    fn regs_w64(&mut self, reg_i: u8, val: u64) {
        if reg_i == 0 {
            return; // writes to x0 are ignored
        }
        self.regs.x[reg_i as usize] = val;
    }

    // writes i8 LSB and sign extends
    // fn regs_wi8(&mut self, reg_i: u8, val: u8) {
    // todo!()
    // }

    /// writes sign extended u32 to reg_i register
    fn regs_wi32(&mut self, reg_i: u8, val_i32: u32) {
        // extend sign
        let val_u64 = val_i32 as i32 as i64 as u64;
        self.regs_w64(reg_i, val_u64)
    }

    /// writes zero extended u8 to reg_i register
    fn regs_wu8(&mut self, reg_i: u8, val: u8) {
        self.regs_w64(reg_i, val as u64)
    }

    fn regs_r64(&self, reg_i: u8) -> u64 {
        self.regs.x[reg_i as usize]
    }

    fn pc_inc(&mut self) {
        self.regs.pc += 4;
        println!("DBG: pc: 0x{:x} -> 0x{:x}", self.regs.pc - 4, self.regs.pc)
    }

    fn pc_add_i13(&mut self, off13: I13) {
        let old_pc = self.regs.pc;
        self.regs.pc = self.regs.pc.add_i13(off13);
        println!("DBG: pc: 0x{old_pc:x} + 0x{off13:x} -> 0x{:x}",
                 self.regs.pc);
    }

    fn pc_add_i21(&mut self, off21: u32) {
        let old_pc = self.regs.pc;
        self.regs.pc = self.regs.pc.add_i21(I21::from_u32(off21));
        println!("DBG: pc: 0x{old_pc:x} + 0x{off21:x} -> 0x{:x}",
                 self.regs.pc);
    }

    // Zics SYSTEM opcodes: CSRRS, ...
    fn opc_system(&mut self, ins: u32) {
        // I-type instruction
        let funct3 = i_funct3(ins);
        let rd = i_rd(ins);
        let rs1 = i_rs1(ins);
        let csr = i_csr(ins);
        println!("DBG: SYSTEM: csr: {:x}, rs1: {:x}, f3: {:x}, rd: {:x}",
                 csr, rs1, funct3, rd);
        match funct3 {
            F3_SYSTEM_CSRRS => {
                println!("DBG: CSRRS");
                let mut csr_v = csr::csr_r64(csr);
                self.regs_w64(rd, csr_v);
                csr_v |= self.regs_r64(rs1);
                csr::csr_w64(csr, csr_v);
            }
            _ => {
                println!("wrong SYSTEM instr");
                bad_instr(ins);
            }
        }
        self.pc_inc();
    }

    // BRANCH opcodes: BEQ, BNE, ...
    fn opc_branch(&mut self, ins: u32) {
        // B-type instructions
        let funct3 = i_funct3(ins);
        let rs1 = i_rs1(ins);
        let rs2 = i_rs2(ins);
        let off13 = i_b_off13(ins);
        println!("DBG: BRANCH: imm[12:0]: 0x{off13:x}, rs2: {rs2}, rs1: {rs1}, f3: 0x{funct3:x}");
        match funct3 {
            // Branch Not Equal
            F3_BRANCH_BNE => {
                println!("DBG: bne x{rs1}, x{rs2}, 0x{:x}",
                         self.regs.pc.add_i13(off13));
                if self.regs_r64(rs1) != self.regs_r64(rs2) {
                    self.pc_add_i13(off13);
                } else {
                    self.pc_inc()
                }
            }
            // Branch EQual
            F3_BRANCH_BEQ => {
                println!("DBG: beq x{rs1}, x{rs2}, 0x{:x}",
                         self.regs.pc.add_i13(off13));
                if self.regs_r64(rs1) == self.regs_r64(rs2) {
                    self.pc_add_i13(off13);
                } else {
                    self.pc_inc()
                }
            }
            _ => {
                println!("unsupported BRACH instr");
                bad_instr(ins);
            }
        }
    }

    // LUI - Load Upper Immidiate
    fn opc_lui(&mut self, ins: u32) {
        let rd = i_rd(ins);
        let uimm20 = i_u_uimm20(ins);
        println!("DBG: LUI: uimm[31:12]: 0x{uimm20:x}, rd: {rd}");
        println!("DBG: lui x{rd}, 0x{:x}", uimm20 >> 12);
        self.regs_w64(rd, uimm20);
        self.pc_inc()
    }

    // Only one instruction AUIPC - Add Upper Immidiate to PC
    fn opc_auipc(&mut self, ins: u32) {
        let rd = i_rd(ins);
        let uimm20 = i_u_uimm20(ins);
        println!("DBG: AUIPC: uimm[31:12]: 0x{uimm20:x}, rd: {rd}");
        self.regs_w64(rd, self.regs.pc + uimm20);
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
                bad_instr(ins);
            }
        }
        self.pc_inc()
    }

    // Only one instrucitn JAL - Jump and Link
    fn opc_jal(&mut self, ins: u32) {
        let rd = i_rd(ins);
        let imm21 = ins.bits(31, 31) << 20 |
                    ins.bits(19, 12) << 12 |
                    ins.bits(20, 20) << 11 |
                    ins.bits(30, 21) << 1;
        println!("DBG: jal x{rd}, 0x{imm21:x} # {imm21}");
        self.regs_w64(rd, self.regs.pc + 4);
        self.pc_add_i21(imm21);
    }

    fn opc_load(&mut self, ins: u32) {
        let imm12 = i_i_imm12(ins);
        let rs1 = i_rs1(ins);
        let funct3 = i_funct3(ins);
        let rd = i_rd(ins);
        let addr = self.regs_r64(rs1).add_i12(I12::from_u16(imm12));
        match funct3 {
            F3_OP_LOAD_LB => {
                todo!();
            }
            // Load Byte Unsigned
            F3_OP_LOAD_LBU => {
                println!("DBG: lbu x{rd}, 0x{imm12}(x{rs1}) # addr: 0x{addr:x}");
                self.regs_wu8(rd, self.bus.read8(addr));
            }
            // Load Word
            F3_OP_LOAD_LW => {
                println!("DBG: lw x{rd}, 0x{imm12}(x{rs1}) # addr: 0x{addr:x}");
                self.regs_wi32(rd, self.bus.read32(addr));
            }
            _ => {
                println!("DBG: unsupported LOAD instruction, funct3: 0b{funct3:b}");
                bad_instr(ins);
            }
        }
        self.pc_inc()
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
            OPC_LUI => self.opc_lui(ins),
            OPC_LOAD => self.opc_load(ins),
            _ => {
                bad_instr(ins);
            }
        }
    }

    pub fn run_until(&mut self, break_point: u64) {
        println!("Running until breakpoint 0x{break_point:x}");
        println!("DBG: pc: 0x{:08x}", self.regs.pc);
        while self.regs.pc != break_point {
            let instr = self.bus.read32(self.regs.pc);
            self.execute_instr(instr);
            // println!("{}", self.regs);
        }
        println!("Stopped at breakpoint 0x{break_point:x}");
    }
}

#[test]
fn test_opcode_csrrs() {
}

#[test]
fn test_opcode_bne() {
    let mut cpu = RV64ICpu::default();
    cpu.regs.x[5] = 1;
    // BNE t0, x0, 0x10
    cpu.execute_instr(0x00029863);
    assert!(cpu.regs.pc == 0x10);
}

#[test]
fn test_opcode_lui() {
    let mut cpu = RV64ICpu::default();
    cpu.regs.x[5] = 0x123;
    // lui x5, 0x10010
    cpu.execute_instr(0x100102b7);
    assert!(cpu.regs.x[5] == 0x10010000);
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

#[test]
// lbu x6, 0x0(x10)
fn test_opcode_lbu() {
    let mut bus = Bus::new_with_ram(0x0000_0000_8000_0000, 4 * 1024);
    bus.write8(0x00000000_8000_003c, 0x48);
    let mut cpu = RV64ICpu::new(bus);

    cpu.regs_w64(6, 0xa5a5a5a5_a5a5_a5a5);
    cpu.regs_w64(10, 0x00000000_8000_003c);
    cpu.execute_instr(0x00054303);
    assert!(cpu.regs_r64(6) == 0x48);
}

#[test]
// lw x7, 0x0(x5)
fn test_opcode_lw() {
    let mut bus = Bus::new_with_ram(0x00000000_0000_0000, 4 * 1024);
    bus.write32(0x00000000_0000_0000, 0xa5a5_a5a5);
    let mut cpu = RV64ICpu::new(bus);
    cpu.regs_w64(7, 0xdead_beef_dead_beef);
    cpu.execute_instr(0x0002a383);
    // lw sign extends 32-bit word
    assert!(cpu.regs_r64(7) == 0xffff_ffff_a5a5_a5a5);
}

#[test]
// beq x6, x0, 0x80000038
fn test_opcode_beq() {
    let mut cpu = RV64ICpu::default();
    // equal
    cpu.regs.x[6] = 0;
    // pc = 0, offset = 18
    cpu.execute_instr(0x00030c63);
    assert!(cpu.regs.pc == 0x18);

    // not equal
    cpu.regs.x[6] = 1;
    cpu.execute_instr(0x00030c63);
    // pc = 0x18 + 4
    assert!(cpu.regs.pc == 0x1c);
}

#[test]
fn registers_writes() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_wi32(1, 0x_8000_0000);
    println!("{:x}", cpu.regs.x[1]); // == 0xffff_ffff_8000_000);
    assert!(cpu.regs.x[1] == 0xffff_ffff_8000_0000);
}
