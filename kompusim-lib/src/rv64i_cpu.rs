use crate::alu::{Imm, I12, I13, I21, I6};
use crate::bits::BitOps;
use crate::bus::Bus;
use crate::csr;
use crate::rv64i_dec::*;
use crate::rvc_dec::{c_i_opcode, decode_rvc_instr, instr_is_rvc, COpcode};

/// exec_continue() returns:
pub enum ExecEvent {
    /// CPU stopped at addr because it executed maximum instructions passed to exec_continue()
    MaxInstructions(u64),
    /// Hit a breakpoint at addr
    Breakpoint(u64),
}

// RV64I Unprivileged Registers
#[derive(Clone, Debug, Default)]
pub struct RV64IURegs {
    // x0: is always zero
    pub x: [u64; 32],
    pub pc: u64,
}

// TODO: make regs private?
#[derive(Default)]
pub struct RV64ICpu {
    pub regs: RV64IURegs,
    lr_sc_reservation: u64,
    pub bus: Bus,
    // TODO: optimize - use hashmap:
    breakpoints: Vec<u64>,
    /// Number of executed instructions
    num_exec_instr: u64,
}

impl RV64ICpu {
    pub fn new(bus: Bus) -> RV64ICpu {
        RV64ICpu {
            bus,
            regs: RV64IURegs::default(),
            lr_sc_reservation: 0,
            breakpoints: Vec::with_capacity(2),
            num_exec_instr: 0,
        }
    }

    pub fn get_num_exec_instr(&self) -> u64 {
        self.num_exec_instr
    }

    pub fn fetch_instr(&self) -> u32 {
        // TODO: raise fault
        self.get_instr(self.get_pc())
    }

    pub fn get_instr(&self, addr: u64) -> u32 {
        self.bus.read32(addr)
    }

    // TODO: remove it because it doesn't support compressed instructions
    pub fn get_n_instr(&self, addr: u64, n_instr: usize) -> Vec<u32> {
        let mut instructions = Vec::with_capacity(n_instr);
        for i in 0..n_instr {
            instructions.push(self.get_instr(addr + 4 * i as u64))
        }
        instructions
    }

    pub fn get_pc(&self) -> u64 {
        self.regs.pc
    }

    pub fn set_ram_sz(&mut self, ram_sz: u64) {
        self.bus.set_ram_sz(ram_sz);
    }

    pub fn get_ram(&self, addr: u64, size: u64) -> Option<&[u8]> {
        self.bus.get_ram(addr, size)
    }

    pub fn get_regs(&self) -> &RV64IURegs {
        &self.regs
    }

    pub fn add_breakpoint(&mut self, breakpoint: u64) {
        self.breakpoints.push(breakpoint)
    }

    // reg_i - register index (0 - 31)
    pub fn regs_w64(&mut self, reg_i: u8, val: u64) {
        if reg_i == 0 {
            return; // writes to x0 are ignored
        }
        self.regs.x[reg_i as usize] = val;
    }

    /// writes sign extended u32 to reg_i register
    pub fn regs_wi32(&mut self, reg_i: u8, val_i32: u32) {
        // extend sign
        let val_u64 = val_i32 as i32 as i64 as u64;
        self.regs_w64(reg_i, val_u64)
    }

    // writes i8 LSB and sign extends
    fn regs_wi8(&mut self, reg_i: u8, val: u8) {
        let mut val: u64 = val as u64;
        if val.bit(7) {
            // sign extend
            val |= 0xfff_ffff_ffff_ff00;
        }
        self.regs_w64(reg_i, val)
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

    /// common 32 bit instruction executed
    fn pc_inc(&mut self) {
        self.regs.pc += 4;
    }

    /// compressed 16-bit instruction executed
    fn pc_inc_rvc(&mut self) {
        self.regs.pc += 2;
    }

    // set PC to new_addr
    fn pc_jump(&mut self, new_addr: u64) {
        self.regs.pc = new_addr;
    }

    fn pc_add_i13(&mut self, off13: I13) {
        self.regs.pc = self.regs.pc.add_i13(off13);
    }

    fn pc_add_i21(&mut self, off21: I21) {
        self.regs.pc = self.regs.pc.add_i21(off21);
    }

    fn exe_opc_system(&mut self, csr: u16, rs1: u8, funct3: u8, rd: u8) -> Result<(), String> {
        match funct3 {
            F3_SYSTEM_CSRRS => {
                let mut csr_v = csr::csr_r64(csr);
                self.regs_w64(rd, csr_v);
                csr_v |= self.regs_r64(rs1);
                csr::csr_w64(csr, csr_v);
            }
            _ => {
                return Err(format!("SYSTEM, funct3: {funct3:x}"));
            }
        }
        self.pc_inc();
        Ok(())
    }

    // BRANCH opcodes: BEQ, BNE, BLT, ...
    fn exe_opc_branch(&mut self, off13: I13, rs2: u8, rs1: u8, funct3: u8) -> Result<(), String> {
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
                return Err(format!("BRANCH, funct3: 0b{funct3:b}"));
            }
        }
        Ok(())
    }

    // LUI - Load Upper Immidiate
    fn exe_opc_lui(&mut self, uimm20: u64, rd: u8) -> Result<(), String> {
        self.regs_w64(rd, uimm20);
        self.pc_inc();
        Ok(())
    }

    // Only one instruction AUIPC - Add Upper Immidiate to PC
    fn exe_opc_auipc(&mut self, uimm20: u64, rd: u8) -> Result<(), String> {
        self.regs_w64(rd, self.regs.pc + uimm20);
        self.pc_inc();
        Ok(())
    }

    fn exe_opc_op_imm(
        &mut self,
        imm12: I12,
        rs1: u8,
        funct3: u8,
        rd: u8,
        rvc: bool,
    ) -> Result<(), String> {
        match funct3 {
            // arithmetic overflow is ignored
            F3_OP_IMM_ADDI => {
                self.regs_w64(rd, self.regs_r64(rs1).add_i12(imm12));
            }
            _ => {
                return Err(format!("OP_IMM, funct3: 0b{funct3:b}"));
            }
        }
        if rvc {
            self.pc_inc_rvc()
        } else {
            self.pc_inc()
        }
        Ok(())
    }

    // ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND
    fn exe_opc_op(
        &mut self,
        funct7: u8,
        rs2: u8,
        rs1: u8,
        funct3: u8,
        rd: u8,
        rvc: bool,
    ) -> Result<(), String> {
        match (funct7, funct3) {
            (F7_OP_ADD, F3_OP_ADD_SUB) => {
                // ignore overflow with wrapping_add()
                self.regs_w64(rd, self.regs_r64(rs1).wrapping_add(self.regs_r64(rs2)))
            }
            (F7_OP_SUB, F3_OP_ADD_SUB) => {
                // ignore overflow with wrapping_sub()
                self.regs_w64(rd, self.regs_r64(rs1).wrapping_sub(self.regs_r64(rs2)))
            }
            (_, _) => return Err(format!("OP, funct7: {funct7:x}, funct3: {funct3:x}")),
        }
        if rvc {
            self.pc_inc_rvc()
        } else {
            self.pc_inc();
        }
        Ok(())
    }

    // Only one instrucitn JAL - Jump and Link
    fn exe_opc_jal(&mut self, imm21: I21, rd: u8) -> Result<(), String> {
        self.regs_w64(rd, self.regs.pc + 4);
        self.pc_add_i21(imm21);
        Ok(())
    }

    // JALR - Jump and Link Register
    fn exe_opc_jalr(&mut self, imm12: I12, rs1: u8, rd: u8) -> Result<(), String> {
        let new_addr = self.regs_r64(rs1).add_i12(imm12).rst_bits(0, 0);
        self.regs_w64(rd, self.regs.pc + 4);
        self.pc_jump(new_addr);
        Ok(())
    }

    // LOAD instructions: LB, LBU, LW, ...
    fn exe_opc_load(&mut self, imm12: I12, rs1: u8, funct3: u8, rd: u8) -> Result<(), String> {
        let addr = self.regs_r64(rs1).add_i12(imm12);
        match funct3 {
            F3_OP_LOAD_LB => {
                self.regs_wi8(rd, self.bus.read8(addr));
            }
            // Load Byte Unsigned
            F3_OP_LOAD_LBU => {
                self.regs_wu8(rd, self.bus.read8(addr));
            }
            // Load Word
            F3_OP_LOAD_LW => {
                // TODO raise fault if returns 0xffff_ffff?
                self.regs_wi32(rd, self.bus.read32(addr));
            }
            _ => {
                return Err(format!("LOAD, funct3: 0b{funct3:b}"));
            }
        }
        self.pc_inc();
        Ok(())
    }

    fn exe_opc_store(&mut self, imm12: I12, rs2: u8, rs1: u8, funct3: u8) -> Result<(), String> {
        let addr = self.regs_r64(rs1).add_i12(imm12);
        match funct3 {
            F3_OP_STORE_SB => {
                todo!();
            }
            F3_OP_STORE_SW => self.bus.write32(addr, self.regs_r32(rs2)),
            _ => {
                return Err(format!("STORE, funct3: 0b{funct3:b}"));
            }
        }
        self.pc_inc();
        Ok(())
    }

    fn exe_opc_amo(
        &mut self,
        funct5: u8,
        rs2: u8,
        rs1: u8,
        funct3: u8,
        rd: u8,
    ) -> Result<(), String> {
        // TODO: check whether rs1 value (address) is 8-byte aligned. If not aligned then generate
        // an address misalgined exception.
        match (funct5, funct3) {
            // lr.w
            (F5_OP_AMO_LRW, F3_OP_AMO_WORD) if rs2 == 0 => {
                let addressed_word = self.bus.read32(self.regs_r64(rs1));
                self.regs_wi32(rd, addressed_word);
                // registers a reservation set — a set of bytes that subsumes the bytes in the addressed word.
                self.lr_sc_reservation = addressed_word as u64;
                // TODO: check lr_sc_reservation in sc.w
            }
            _ => {
                return Err(format!("AMO, funct5: {funct5:x}, funct3: {funct3:x}"));
            }
        }
        self.pc_inc();
        Ok(())
    }

    pub fn execute_instr(&mut self, instr: u32) {
        if let Err(e) = match decode_instr(instr) {
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
            } => self.exe_opc_op_imm(imm12, rs1, funct3, rd, /* rvc = */ false),
            Opcode::Op {
                funct7,
                rs2,
                rs1,
                funct3,
                rd,
            } => self.exe_opc_op(funct7, rs2, rs1, funct3, rd, /* rvc = */ false),
            Opcode::Amo {
                funct5,
                rs2,
                rs1,
                funct3,
                rd,
            } => self.exe_opc_amo(funct5, rs2, rs1, funct3, rd),
            Opcode::System {
                csr,
                rs1,
                funct3,
                rd,
            } => self.exe_opc_system(csr, rs1, funct3, rd),
            Opcode::Uknown => Err(String::new()),
        } {
            let opc = i_opcode(instr);
            eprintln!("ERROR: Uknown instruction {e}\nPC = 0x{:x}, code: 0x{instr:x} (0b_{instr:032b}), opcode: 0x{opc:x} (0b_{opc:07b})",
            self.get_pc());
            // TODO: remove panic and trigger CPU exception
            panic!();
        }
        self.num_exec_instr += 1;
    }

    /// C.LI compressed instruction
    pub fn exe_opc_c_li(&mut self, imm6: I6, rd: u8) -> Result<(), String> {
        let imm6: i8 = imm6.into();
        self.regs_wi8(rd, imm6 as u8);
        self.pc_inc_rvc();
        Ok(())
    }

    /// Execute a compressed instruction
    pub fn execute_rvc_instr(&mut self, c_instr: u16) {
        if let Err(e) = match decode_rvc_instr(c_instr) {
            COpcode::CNOP => {
                self.pc_inc_rvc();
                Ok(())
            }
            // C.ADDI expands into addi rd, rd, nzimm[5:0]
            COpcode::CADDI { imm6, rd } => {
                self.exe_opc_op_imm(imm6.into(), rd, F3_OP_IMM_ADDI, rd, /* rvc = */ true)
            }
            COpcode::CLI { imm6, rd } => self.exe_opc_c_li(imm6, rd),
            // C.JR expands to JALR x0, 0(rs1)
            COpcode::CJR { rs1 } => self.exe_opc_jalr(0_u16.into(), rs1, 0),
            COpcode::CADD { rd, rs2 } => {
                self.exe_opc_op(F3_OP_ADD_SUB, rs2, rd, F7_OP_ADD, rd, /* rvc = */ true)
            }
            // C.J expands to jal x0, offset[11:1].
            COpcode::CJ { imm12 } => self.exe_opc_jal(imm12.into(), /* rd = x0 */ 0),
            COpcode::Uknown => Err(String::new()),
        } {
            let opc = c_i_opcode(c_instr);
            eprintln!("ERROR: Unknown RVC instruction {e}\nPC = 0x{:x}, code: 0x{c_instr:x} (0b_{c_instr:016b}), opcode: 0x{opc:x} (0b_{opc:05b})",
            self.get_pc());
            // TODO: remove panic and trigger CPU exception
            panic!();
        }

        self.num_exec_instr += 1;
    }

    fn check_break_points(&self, addr: u64) -> bool {
        // TODO: check all breakpoints
        // TODO: optimize to use hashmap
        !self.breakpoints.is_empty() && self.breakpoints[0] == addr
    }

    /// Returns PC (i.e. where stopped)
    pub fn exec_continue(&mut self, max_instr: u64) -> ExecEvent {
        for _ in 0..max_instr {
            let instr = self.fetch_instr();
            if instr_is_rvc(instr) {
                self.execute_rvc_instr(instr.bits(15, 0) as u16);
            } else {
                self.execute_instr(instr);
            }
            if self.check_break_points(self.regs.pc) {
                return ExecEvent::Breakpoint(self.regs.pc);
            }
        }
        ExecEvent::MaxInstructions(self.regs.pc)
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
