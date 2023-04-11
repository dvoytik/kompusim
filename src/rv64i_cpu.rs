use crate::alu::{Imm, I12, I13, I21};
use crate::bits::BitOps;
use crate::bus::Bus;
use crate::csr;
use crate::rv64i_dec::*;

// RV64I Unprivileged Registers
#[derive(Debug, Default)]
pub struct RV64IURegs {
    // x0: is always zero
    pub x: [u64; 32],
    pub pc: u64,
}

impl RV64IURegs {
    // Get ABI register name
    fn reg_idx2abi(r: u8) -> &'static str {
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

    fn print_reg(&self, ri: u8) {
        if ri == 0 {
            return;
        }
        let r_abi = Self::reg_idx2abi(ri);
        println!(
            " x{ri} ({r_abi}): 0x{0:016x} | b'{0:064b}",
            self.x[ri as usize]
        );
    }

    fn print_2regs(&self, ri1: u8, ri2: u8) {
        if ri1 != 0 {
            let r1_abi = Self::reg_idx2abi(ri1);
            println!(
                " x{ri1} ({r1_abi}): 0x{0:016x} | b'{0:064b}",
                self.x[ri1 as usize]
            );
        }
        if ri2 != 0 {
            let r2_abi = Self::reg_idx2abi(ri2);
            println!(
                " x{ri1} ({r2_abi}): 0x{0:016x} | b'{0:064b}",
                self.x[ri2 as usize]
            );
        }
    }
}

// TODO: make regs private?
#[derive(Default)]
pub struct RV64ICpu {
    pub regs: RV64IURegs,
    pub bus: Bus,
    breakpoints: Vec<u64>, // TODO: optimize - use hashmap
    tracing: bool,
}

fn bad_instr(ins: u32) {
    let opc = i_opcode(ins);
    panic!("ERROR: bad instr: 0x{ins:x} (0b_{ins:b}), opcode: 0x{opc:x} (0b_{opc:07b})");
}

impl RV64ICpu {
    pub fn new(bus: Bus) -> RV64ICpu {
        RV64ICpu {
            bus,
            regs: RV64IURegs::default(),
            tracing: false,
            breakpoints: Vec::with_capacity(2),
        }
    }

    pub fn get_cur_instr(&self) -> u32 {
        self.bus.read32(self.regs.pc)
    }

    pub fn get_pc(&self) -> u64 {
        self.regs.pc
    }

    /// Enable printing CPU state on console
    pub fn enable_tracing(&mut self, enable: bool) {
        self.tracing = enable;
        self.bus.all_dev_enable_tracing(enable);
    }

    pub fn tracing(&self) -> bool {
        self.tracing
    }

    pub fn get_regs(&self) -> &RV64IURegs {
        &self.regs
    }

    pub fn add_breakpoint(&mut self, breakpoint: u64) {
        self.breakpoints.push(breakpoint)
    }

    fn trace_pc(&self, old: u64, new: u64) {
        if self.tracing {
            println!("PC: 0x{old:x} -> 0x{new:x}")
        }
    }

    fn trace_pc_add(&self, old_pc: u64, add: u64, new_pc: u64) {
        if self.tracing {
            println!("PC: 0x{old_pc:x} + 0x{add:x} -> 0x{new_pc:x}");
        }
    }

    fn trace_print_reg(&self, ri: u8) {
        if self.tracing {
            self.regs.print_reg(ri)
        }
    }

    fn trace_print_2regs(&self, r1: u8, r2: u8) {
        if self.tracing {
            self.regs.print_2regs(r1, r2)
        }
    }

    // reg_i - register index (0 - 31)
    pub fn regs_w64(&mut self, reg_i: u8, val: u64) {
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
    pub fn regs_wi32(&mut self, reg_i: u8, val_i32: u32) {
        // extend sign
        let val_u64 = val_i32 as i32 as i64 as u64;
        self.regs_w64(reg_i, val_u64)
    }

    /// writes zero extended u8 to reg_i register
    fn regs_wu8(&mut self, reg_i: u8, val: u8) {
        self.regs_w64(reg_i, val as u64)
    }

    pub fn regs_r64(&self, reg_i: u8) -> u64 {
        self.regs.x[reg_i as usize]
    }

    // reads [31:0] from register x[reg_i]
    fn regs_r32(&self, reg_i: u8) -> u32 {
        self.regs.x[reg_i as usize] as u32
    }

    // Treat register as signed 64 bit
    fn regs_ri64(&self, reg_i: u8) -> i64 {
        self.regs.x[reg_i as usize] as i64
    }

    fn pc_inc(&mut self) {
        self.regs.pc += 4;
        self.trace_pc(self.regs.pc - 4, self.regs.pc);
    }

    // set PC to new_addr
    fn pc_jump(&mut self, new_addr: u64) {
        let old_pc = self.regs.pc;
        self.regs.pc = new_addr;
        self.trace_pc(old_pc, new_addr)
    }

    fn pc_add_i13(&mut self, off13: I13) {
        let old_pc = self.regs.pc;
        self.regs.pc = self.regs.pc.add_i13(off13);
        self.trace_pc_add(old_pc, off13.into(), self.regs.pc);
    }

    fn pc_add_i21(&mut self, off21: I21) {
        let old_pc = self.regs.pc;
        self.regs.pc = self.regs.pc.add_i21(I21::from(off21));
        self.trace_pc_add(old_pc, off21.into(), self.regs.pc);
    }

    fn exe_opc_system(&mut self, csr: u16, rs1: u8, funct3: u8, rd: u8) {
        match funct3 {
            F3_SYSTEM_CSRRS => {
                let mut csr_v = csr::csr_r64(csr);
                self.regs_w64(rd, csr_v);
                csr_v |= self.regs_r64(rs1);
                csr::csr_w64(csr, csr_v);
            }
            _ => {
                // TODO: generate exception
                println!("wrong SYSTEM instr (funct3: {funct3:x})");
            }
        }
        self.pc_inc();
    }

    // BRANCH opcodes: BEQ, BNE, BLT, ...
    fn exe_opc_branch(&mut self, off13: I13, rs2: u8, rs1: u8, funct3: u8) {
        match funct3 {
            // Branch Not Equal
            F3_BRANCH_BNE => {
                if self.regs_r64(rs1) != self.regs_r64(rs2) {
                    self.pc_add_i13(off13);
                } else {
                    self.pc_inc()
                }
            }
            // Branch EQual
            F3_BRANCH_BEQ => {
                if self.regs_r64(rs1) == self.regs_r64(rs2) {
                    self.pc_add_i13(off13);
                } else {
                    self.pc_inc()
                }
            }
            // Branch Less Than (signed comparison)
            F3_BRANCH_BLT => {
                if self.regs_ri64(rs1) < self.regs_ri64(rs2) {
                    self.pc_add_i13(off13);
                } else {
                    self.pc_inc()
                }
            }
            _ => {
                println!("ERROR: unsupported BRACH instr, funct3: 0b{funct3:b}");
            }
        }
    }

    // LUI - Load Upper Immidiate
    fn exe_opc_lui(&mut self, uimm20: u64, rd: u8) {
        self.regs_w64(rd, uimm20);
        self.pc_inc()
    }

    // Only one instruction AUIPC - Add Upper Immidiate to PC
    fn exe_opc_auipc(&mut self, uimm20: u64, rd: u8) {
        self.regs_w64(rd, self.regs.pc + uimm20);
        self.pc_inc()
    }

    fn exe_opc_op_imm(&mut self, imm12: I12, rs1: u8, funct3: u8, rd: u8) {
        match funct3 {
            // arithmetic overflow is ignored
            F3_OP_IMM_ADDI => {
                self.trace_print_2regs(rd, rs1);
                self.regs_w64(rd, self.regs_r64(rs1).add_i12(imm12));
                self.trace_print_reg(rd);
            }
            _ => {
                println!("ERROR: unsupported OP_IMM instr, funct3: 0b{funct3:b}");
            }
        }
        self.pc_inc()
    }

    // Only one instrucitn JAL - Jump and Link
    fn exe_opc_jal(&mut self, imm21: I21, rd: u8) {
        self.regs_w64(rd, self.regs.pc + 4);
        self.pc_add_i21(imm21);
        self.trace_print_reg(rd);
    }

    // JALR - Jump and Link Register
    fn exe_opc_jalr(&mut self, imm12: I12, rs1: u8, rd: u8) {
        let new_addr = self.regs_r64(rs1).add_i12(imm12).rst_bits(0, 0);
        self.trace_print_reg(rs1);
        self.regs_w64(rd, self.regs.pc + 4);
        self.pc_jump(new_addr);
    }

    // LOAD instructions: LB, LBU, LW, ...
    fn exe_opc_load(&mut self, imm12: I12, rs1: u8, funct3: u8, rd: u8) {
        let addr = self.regs_r64(rs1).add_i12(imm12);
        match funct3 {
            F3_OP_LOAD_LB => {
                todo!();
            }
            // Load Byte Unsigned
            F3_OP_LOAD_LBU => {
                self.regs_wu8(rd, self.bus.read8(addr));
            }
            // Load Word
            F3_OP_LOAD_LW => {
                self.regs_wi32(rd, self.bus.read32(addr));
            }
            _ => {
                println!("ERROR: unsupported LOAD instruction, funct3: 0b{funct3:b}");
            }
        }
        self.pc_inc()
    }

    fn exe_opc_store(&mut self, imm12: I12, rs2: u8, rs1: u8, funct3: u8) {
        let addr = self.regs_r64(rs1).add_i12(imm12);
        match funct3 {
            F3_OP_STORE_SB => {
                todo!();
            }
            F3_OP_STORE_SW => self.bus.write32(addr, self.regs_r32(rs2)),
            _ => {
                println!("ERROR: unsupported STORE instruction, funct3: 0b{funct3:b}");
            }
        }
        self.pc_inc()
    }

    pub fn execute_instr(&mut self, instr: u32) {
        match decode_instr(instr) {
            Opcode::Lui { uimm20, rd } => self.exe_opc_lui(uimm20, rd),
            Opcode::Auipc { uimm20, rd } => self.exe_opc_auipc(uimm20, rd),
            Opcode::Branch {
                off13,
                rs2,
                rs1,
                funct3,
            } => self.exe_opc_branch(off13, rs2, rs1, funct3),
            Opcode::Jal { imm21, rd } => self.exe_opc_jal(imm21, rd),
            Opcode::Jalr { imm12, rs1, rd } => self.exe_opc_jalr(imm12, rs1, rd),
            Opcode::Load {
                imm12,
                rs1,
                funct3,
                rd,
            } => self.exe_opc_load(imm12, rs1, funct3, rd),
            Opcode::Store {
                imm12,
                rs2,
                rs1,
                funct3,
            } => self.exe_opc_store(imm12, rs2, rs1, funct3),
            Opcode::OpImm {
                imm12,
                rs1,
                funct3,
                rd,
            } => self.exe_opc_op_imm(imm12, rs1, funct3, rd),
            Opcode::System {
                csr,
                rs1,
                funct3,
                rd,
            } => self.exe_opc_system(csr, rs1, funct3, rd),
            Opcode::Uknown => bad_instr(instr),
        }
    }

    /// Returns PC (i.e. where stopped)
    pub fn exec_continue(&mut self, max_instr: u64) -> u64 {
        for _ in 0..max_instr {
            self.execute_instr(self.get_cur_instr());
            // TODO: check all breakpoints
            // TODO: optimize to use hashmap
            if self.breakpoints[0] == self.regs.pc {
                break;
            }
        }
        self.regs.pc
    }
}

#[test]
fn test_instr_decode_immidiates() {
    let imm12 = i_i_type_imm12(0xffff_ffff);
    assert!(imm12.0 == -1);

    let imm12 = i_i_type_imm12(0x800f_ffff);
    assert!(imm12.0 == -2048);

    let imm12 = i_i_type_imm12(0x0fff_ffff);
    assert!(imm12.0 == 255);
}
