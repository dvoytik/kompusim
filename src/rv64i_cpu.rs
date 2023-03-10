use core::fmt;

use crate::pmem::Pmem;

//
#[derive(Debug, Default)]
pub struct RV64IUnprivRegs {
    //x0: is always zero
    x1: u64,
    x2: u64,
    x3: u64,
    x4: u64,
    x5: u64,
    x6: u64,
    x7: u64,
    x8: u64,
    pub pc: u64,
}

impl fmt::Display for RV64IUnprivRegs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //let (x1, x2) = self;
        writeln!(f, "x1: 0x{0:16X} | b'{0:064b}", self.x1)?;
        writeln!(f, "x2: 0x{0:16X} | b'{0:064b}", self.x2)?;
        writeln!(f, "x3: 0x{0:16X} | b'{0:064b}", self.x3)?;
        writeln!(f, "x4: 0x{0:16X} | b'{0:064b}", self.x4)?;
        writeln!(f, "x5: 0x{0:16X} | b'{0:064b}", self.x5)?;
        writeln!(f, "x6: 0x{0:16X} | b'{0:064b}", self.x6)?;
        writeln!(f, "x7: 0x{0:16X} | b'{0:064b}", self.x7)?;
        writeln!(f, "x8: 0x{0:16X} | b'{0:064b}", self.x8)?;
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

    fn execute_instr(&mut self, ins: u32) {
        // TODO: macro with bits matching
        println!("DBG: instr: {:x}", ins);
        let opcode = i_opcode(ins);
        match opcode {
            OP_ZICSR => {
                let funct3 = i_funct3(ins);
                let rd = i_rd(ins);
                let rs1 = i_rs1(ins);
                let csr = i_csr(ins);
                match (csr, rs1, funct3, rd) {
                    (_, _, F3_CSRRS, _) => {
                        println!("got CSRRS")
                    }
                    (_, _, _, _) => {
                        panic!("wrong Zicsr instr")
                    }
                }
            }
            _ => {
                panic!("not implemented")
            }
        }
    }

    pub fn run_until(&mut self, break_point: u64) {
        while self.regs.pc != break_point {
            println!("DBG: {:x}", self.regs.pc);
            let instr = self.mem.read32(self.regs.pc);
            self.execute_instr(instr);
        }
    }
}
