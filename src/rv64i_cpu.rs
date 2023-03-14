use core::fmt;

use crate::csr::{self, *};
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

const OP_ZICSR: u8 = 0b111_0011;
const F3_CSRRS: u8 = 0b010;

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
    (ins >> 15) as u8 & 0x1f
}

#[inline(always)]
fn i_csr(ins: u32) -> u16 {
    (ins >> 20) as u16 & 0xfff
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

    // SYSTEM opcodes Zicsr: CSRRS
    fn opc_zicsr(&mut self, ins: u32) {
        // I-type instruction
        let funct3 = i_funct3(ins);
        let rd = i_rd(ins);
        let rs1 = i_rs1(ins);
        let csr = i_csr(ins);
        println!(
            "DBG: Zicsr: csr: {:x}, rs1: {:x}, f3: {:x}, rd: {:x}",
            csr, rs1, funct3, rd
        );
        match funct3 {
            F3_CSRRS => {
                println!("DBG: CSRRS");
                let mut csr_v = csr::read_csr(csr);
                self.reg_w64(rd, csr_v);
                csr_v |= self.reg_r64(rs1);
                csr::write_csr(csr, csr_v);
            }
            _ => {
                panic!("wrong Zicsr instr")
            }
        }
        self.pc_inc();
    }

    fn execute_instr(&mut self, ins: u32) {
        // TODO: macro with bits matching
        println!("DBG: instr: 0x{:08x}", ins);
        let opcode = i_opcode(ins);
        match opcode {
            OP_ZICSR => self.opc_zicsr(ins),
            _ => {
                panic!("not implemented opcode: {:x}", opcode)
            }
        }
    }

    pub fn run_until(&mut self, break_point: u64) {
        while self.regs.pc != break_point {
            println!("DBG: 0x{:08x}", self.regs.pc);
            let instr = self.mem.read32(self.regs.pc);
            self.execute_instr(instr);
        }
    }
}
