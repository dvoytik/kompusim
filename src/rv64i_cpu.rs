use core::fmt;

use crate::bits::Bit;
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

const OP_SYSTEM: u8 = 0b11_100_11;
const OP_BRANCH: u8 = 0b11_000_11;
const OP_AUIPC: u8 = 0b00_101_11;
const OP_OP_IMM: u8 = 0b00_100_11;

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

// Decode 12-bit signed offset from a B-type instruction
#[inline(always)]
fn i_b_off12(ins: u32) -> i16 {
    let off_4_1 = ins.bits(11, 8) as u16;
    let off_11 = ins.bits(7, 7) as u16;
    let off_10_5 = ins.bits(30, 25) as u16;
    let off_12 = ins.bits(31, 31) as u16;
    let off12 = off_11 << 11 | off_10_5 << 5 | off_4_1 << 1;
    if off_12 == 1 {
        -(off12 as i16)
    } else {
        off12 as i16
    }
}

fn i_u_uimm20(ins: u32) -> u32 {
    ins & 0xffff_f000
}
impl RV64ICpu {
    pub fn new(mem: Pmem) -> RV64ICpu {
        RV64ICpu {
            mem,
            regs: RV64IUnprivRegs::default(),
        }
    }

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

    // pc = pc + signed_offset[12:0]
    fn pc_add_soff12(&mut self, s_off12: i16) {
        // TODO: exceptions?
        if s_off12 < 0 {
            self.regs.pc -= s_off12.abs() as u64
        } else {
            self.regs.pc += s_off12 as u64
        }
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
        //let rd = i_rd(ins);
        let rs1 = i_rs1(ins);
        let rs2 = i_rs2(ins);
        let off12 = i_b_off12(ins);
        println!(
            "DBG: BRANCH: imm[12:0]: 0x{off12:x}, rs2: {rs2}, \
             rs1: {rs1}, f3: 0x{funct3:x}"
        );
        match funct3 {
            F3_BRANCH_BNE => {
                println!("DBG: bne x{}, x{}, 0x{:x}", rs1, rs2, off12);
                // Branch Not Equal
                if self.regs_r64(rs1) != self.regs_r64(rs2) {
                    self.pc_add_soff12(off12);
                } else {
                    self.pc_inc()
                }
            }
            _ => {
                panic!("wrong Zicsr instr")
            }
        }
    }

    //
    fn opc_auipc(&mut self, ins: u32) {
        let rd = i_rd(ins);
        let uimm20 = i_u_uimm20(ins);
        println!("DBG: AUIPC: uimm[31:12]: 0x{uimm20:x}, rd: {rd}");
        self.regs_w64(rd, self.regs.pc + uimm20 as u64);
        //println!("{}", self.regs);
        self.pc_inc()
    }

    fn execute_instr(&mut self, ins: u32) {
        // TODO: macro with bits matching
        println!("\nDBG: instr: 0x{:08x}", ins);
        let opcode = i_opcode(ins);
        match opcode {
            OP_SYSTEM => self.opc_system(ins),
            OP_BRANCH => self.opc_branch(ins),
            OP_AUIPC => self.opc_auipc(ins),
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
