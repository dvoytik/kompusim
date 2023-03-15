use core::fmt;

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
        writeln!(f, "x1: 0x{0:16X} | b'{0:064b}", self.x[1])?;
        writeln!(f, "x2: 0x{0:16X} | b'{0:064b}", self.x[2])?;
        writeln!(f, "x3: 0x{0:16X} | b'{0:064b}", self.x[3])?;
        writeln!(f, "x4: 0x{0:16X} | b'{0:064b}", self.x[4])?;
        writeln!(f, "x5: 0x{0:16X} | b'{0:064b}", self.x[5])?;
        writeln!(f, "x6: 0x{0:16X} | b'{0:064b}", self.x[6])?;
        writeln!(f, "x7: 0x{0:16X} | b'{0:064b}", self.x[7])?;
        writeln!(f, "x8: 0x{0:16X} | b'{0:064b}", self.x[8])?;
        writeln!(f, "pc: 0x{0:16X} | b'{0:064b}", self.pc)
    }
}

#[derive(Default)]
pub struct RV64ICpu {
    pub regs: RV64IUnprivRegs,
    pub mem: Pmem,
}

const OP_SYSTEM: u8 = 0b11_100_11;
const OP_BRANCH: u8 = 0b11_000_11;

const F3_BRANCH_BNE: u8 = 0b001;
const F3_SYSTEM_CSRRS: u8 = 0b010;

// TODO: use macros to bit!(ins, 24, 20)
#[inline(always)]
fn i_opcode(ins: u32) -> u8 {
    ins as u8 & 0b111_1111
}

#[inline(always)]
fn i_funct3(ins: u32) -> u8 {
    (ins >> 12) as u8 & 0x7
}

#[inline(always)]
fn i_rd(ins: u32) -> u8 {
    (ins >> 7) as u8 & 0x1f
}

#[inline(always)]
fn i_rs1(ins: u32) -> u8 {
    (ins >> 15) as u8 & 0x1f // [19:15]
}

#[inline(always)]
fn i_rs2(ins: u32) -> u8 {
    (ins >> 20) as u8 & 0x1f // [24:20]
}

#[inline(always)]
fn i_csr(ins: u32) -> u16 {
    (ins >> 20) as u16 & 0xfff
}

// Decode 12-bit signed offset from a B-type instruction
#[inline(always)]
fn i_b_off12(ins: u32) -> i16 {
    let off_4_1 = (ins >> 8) as u16 & 0xf; // [11:8]
    let off_11 = (ins >> 7) as u16 & 1; // [7]
    let off_10_5 = (ins >> 25) as u16 & 0x3f; // [30:25]
    let off_12 = (ins >> 31) as u16 & 1; // [31]
    let off12 = off_11 << 11 | off_10_5 << 5 | off_4_1 << 1;
    if off_12 == 1 {
        -(off12 as i16)
    } else {
        off12 as i16
    }
}

impl RV64ICpu {
    pub fn new(mem: Pmem) -> RV64ICpu {
        RV64ICpu {
            mem,
            regs: RV64IUnprivRegs::default(),
        }
    }

    fn reg_w64(&mut self, reg_i: u8, val: u64) {
        if reg_i == 0 {
            return; // writes to x0 are ignored
        }
        self.regs.x[reg_i as usize] = val;
    }

    fn reg_r64(&self, reg_i: u8) -> u64 {
        self.regs.x[reg_i as usize]
    }

    fn pc_inc(&mut self) {
        self.regs.pc += 4;
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
                self.reg_w64(rd, csr_v);
                csr_v |= self.reg_r64(rs1);
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
        let rs1 = self.reg_r64(i_rs1(ins));
        let rs2 = self.reg_r64(i_rs2(ins));
        let off12 = i_b_off12(ins);
        println!(
            "DBG: BRANCH: imm[12:0]: {:x}, rs2: {:x}, rs1: {:x}, f3: {:x}",
            off12, rs2, rs1, funct3
        );
        match funct3 {
            F3_BRANCH_BNE => {
                println!("DBG: BNE");
                // Branch Not Equal
                if rs1 != rs2 {
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

    fn execute_instr(&mut self, ins: u32) {
        // TODO: macro with bits matching
        println!("DBG: instr: 0x{:08x}", ins);
        let opcode = i_opcode(ins);
        match opcode {
            OP_SYSTEM => self.opc_system(ins),
            OP_BRANCH => self.opc_branch(ins),
            _ => {
                panic!("not implemented opcode: {:x}", opcode)
            }
        }
    }

    pub fn run_until(&mut self, break_point: u64) {
        while self.regs.pc != break_point {
            println!("DBG: pc: 0x{:08x}", self.regs.pc);
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
