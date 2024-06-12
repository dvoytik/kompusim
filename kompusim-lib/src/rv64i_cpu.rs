use crate::alu::{Imm, I12, I13, I21, I6};
use crate::bits::BitOps;
use crate::bus::Bus;
use crate::csr::Csrs;
// use crate::rv64fd::RV64FDRegs;
use crate::rv64i_dec::*;
use crate::rvc_dec::{c_i_opcode, instr_is_rvc, rv64c_decode_instr, COpcode};

/// exec_continue() returns:
pub enum ExecEvent {
    /// CPU stopped at addr because it executed maximum instructions passed to exec_continue()
    MaxInstructions(u64),
    /// Hit a breakpoint at addr
    Breakpoint(u64),
}

const ILEN_32B: u8 = 4;
const ILEN_RVC: u8 = 2;

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
    regs: RV64IURegs,
    // fregs: RV64FDRegs,
    lr_sc_reservation: u64,
    pub bus: Bus,
    csrs: Csrs,
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
            // fregs: RV64FDRegs::default(),
            lr_sc_reservation: 0,
            breakpoints: Vec::with_capacity(2),
            csrs: Csrs::new(),
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

    pub fn regs_w32(&mut self, reg_i: u8, val_i32: u32) {
        self.regs_w64(reg_i, val_i32 as u64)
    }

    // writes i8 LSB and sign extends
    fn regs_wi8(&mut self, reg_i: u8, val: u8) {
        let mut val: u64 = val as u64;
        if val.bit(7) {
            // sign extend
            val |= 0xffff_ffff_ffff_ff00;
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
    fn pc_inc(&mut self, inc: u8) {
        self.regs.pc += inc as u64;
    }

    // set PC to new_addr
    pub fn pc_jump(&mut self, new_addr: u64) {
        self.regs.pc = new_addr;
    }

    fn pc_add_i13(&mut self, off13: I13) {
        self.regs.pc = self.regs.pc.add_i13(off13);
    }

    fn pc_add_i21(&mut self, off21: I21) {
        self.regs.pc = self.regs.pc.add_i21(off21);
    }

    // csrrs, csrrwi
    fn exe_opc_system(&mut self, csr: u16, rs1: u8, funct3: u8, rd: u8) -> Result<(), String> {
        // TODO: each operation is atomic
        match funct3 {
            // csrrs rd, csr, rs1
            F3_SYSTEM_CSRRS => {
                let mut csr_v = self.csrs.r64(csr);
                self.regs_w64(rd, csr_v);
                csr_v |= self.regs_r64(rs1);
                self.csrs.w64(csr, csr_v);
            }
            // csrrw rd, csr, rs1
            F3_SYSTEM_CSRRW => {
                self.regs_w64(rd, self.csrs.r64(csr));
                self.csrs.w64(csr, self.regs_r64(rs1));
            }
            // csrrwi rd, csr, uimm5
            F3_SYSTEM_CSRRWI => {
                self.regs_w64(rd, self.csrs.r64(csr));
                // rs1 is uimm[4:0]
                self.csrs.w64(csr, rs1 as u64);
            }
            F3_SYSTEM_WFI => {
                // TODO: hint to check interrupts
            }
            _ => {
                return Err(format!("SYSTEM, funct3: {funct3:x}"));
            }
        }
        self.pc_inc(ILEN_32B);
        Ok(())
    }

    // BRANCH opcodes: BEQ, BNE, BLT, BLTU, ...
    fn exe_opc_branch(
        &mut self,
        off13: I13,
        rs2: u8,
        rs1: u8,
        funct3: u8,
        ilen: u8,
    ) -> Result<(), String> {
        match funct3 {
            // Branch Not Equal
            F3_BRANCH_BNE => {
                if self.regs_r64(rs1) != self.regs_r64(rs2) {
                    self.pc_add_i13(off13);
                } else {
                    self.pc_inc(ilen)
                }
            }
            // Branch EQual
            F3_BRANCH_BEQ => {
                if self.regs_r64(rs1) == self.regs_r64(rs2) {
                    self.pc_add_i13(off13);
                } else {
                    self.pc_inc(ilen)
                }
            }
            // Branch Less Than (signed comparison)
            F3_BRANCH_BLT => {
                if self.regs_ri64(rs1) < self.regs_ri64(rs2) {
                    self.pc_add_i13(off13);
                } else {
                    self.pc_inc(ilen)
                }
            }
            // Branch Less Than (Unsigned comparison)
            F3_BRANCH_BLTU => {
                if self.regs_r64(rs1) < self.regs_r64(rs2) {
                    self.pc_add_i13(off13);
                } else {
                    self.pc_inc(ilen)
                }
            }
            // Branch Greater or Equal (signed comparison)
            F3_BRANCH_BGE => {
                if self.regs_ri64(rs1) >= self.regs_ri64(rs2) {
                    self.pc_add_i13(off13);
                } else {
                    self.pc_inc(ilen)
                }
            }
            // Branch if Greater or Equal (Unsigned comparison)
            F3_BRANCH_BGEU => {
                if self.regs_r64(rs1) >= self.regs_r64(rs2) {
                    self.pc_add_i13(off13);
                } else {
                    self.pc_inc(ilen)
                }
            }
            _ => {
                return Err(format!("BRANCH, funct3: 0b{funct3:b}"));
            }
        }
        Ok(())
    }

    // LUI - Load Upper Immidiate
    fn exe_opc_lui(&mut self, uimm20: u32, rd: u8) {
        // sign extend
        self.regs_wi32(rd, uimm20);
    }

    // Only one instruction AUIPC - Add Upper Immidiate to PC
    fn exe_opc_auipc(&mut self, uimm20: u64, rd: u8) -> Result<(), String> {
        // appends 12 low-order zero bits to the 20-bit U-immediate,
        // sign-extends the result to 64 bits, adds it to the address of the AUIPC instruction,
        // then places the result in register rd.
        self.regs_w64(rd, self.regs.pc + (uimm20 as i32 as i64 as u64));
        self.pc_inc(ILEN_32B);
        Ok(())
    }

    fn exe_opc_op_imm(
        &mut self,
        imm12: I12,
        rs1: u8,
        funct3: u8,
        rd: u8,
        isize: u8,
    ) -> Result<(), String> {
        match funct3 {
            // arithmetic overflow is ignored
            F3_OP_IMM_ADDI => {
                self.regs_w64(rd, self.regs_r64(rs1).add_i12(imm12));
            }
            F3_OP_IMM_XORI => {
                self.regs_w64(rd, self.regs_r64(rs1) ^ u64::from(imm12));
            }
            F3_OP_IMM_SLLI => {
                self.regs_w64(rd, self.regs_r64(rs1) << imm12.0.bits(5, 0));
            }
            F3_OP_IMM_SRLI => {
                self.regs_w64(rd, self.regs_r64(rs1) >> imm12.0.bits(5, 0));
            }
            _ => {
                return Err(format!("OP_IMM, funct3: 0b{funct3:b}"));
            }
        }
        self.pc_inc(isize);
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
        // TODO raise fault if returns 0xffff_ffff?
        match funct3 {
            // Load Byte
            F3_OP_LOAD_LB => self.regs_wi8(rd, self.bus.read8(addr)),
            // Load Byte Unsigned
            F3_OP_LOAD_LBU => self.regs_wu8(rd, self.bus.read8(addr)),
            // Load Word
            F3_OP_LOAD_LW => self.regs_wi32(rd, self.bus.read32(addr)),
            // Load Double Word
            F3_OP_LOAD_LD => self.regs_w64(rd, self.bus.read64(addr)),
            _ => {
                return Err(format!("LOAD, funct3: 0b{funct3:b}"));
            }
        }
        Ok(())
    }

    fn exe_opc_store(&mut self, imm12: I12, rs2: u8, rs1: u8, funct3: u8) -> Result<(), String> {
        let addr = self.regs_r64(rs1).add_i12(imm12);
        match funct3 {
            F3_OP_STORE_SB => {
                todo!();
            }
            F3_OP_STORE_SW => self.bus.write32(addr, self.regs_r32(rs2)),
            F3_OP_STORE_SD => self.bus.write64(addr, self.regs_r64(rs2)),
            _ => {
                return Err(format!("STORE, funct3: 0b{funct3:b}"));
            }
        }
        Ok(())
    }

    // Atomic operations
    // TODO: aq, rl are ignored for now
    #[allow(clippy::too_many_arguments)]
    fn exe_opc_amo(
        &mut self,
        funct5: u8,
        _aq: bool,
        _rl: bool,
        rs2: u8,
        rs1: u8,
        funct3: u8,
        rd: u8,
    ) -> Result<(), String> {
        // These AMO instructions atomically load a data value from the address in rs1,
        // place the value into register rd, apply a binary operator to the loaded value and
        // the original value in rs2, then store the result back to the address in rs1.
        // TODO: check whether rs1 value (address) is 8-byte aligned. If not aligned then generate
        // an address misalgined exception.
        match (funct5, funct3) {
            // amoadd.w rd, rs2, rs1 # rd <= mem[rs1]; mem[rs1] <= rd + rs2
            (F5_OP_AMO_ADD, F3_OP_AMO_WORD) => {
                // TODO: use native atomic operation
                let address = self.regs_r64(rs1); // preserve address to avoid problem when rd == rs1
                self.regs_wi32(rd, self.bus.read32(address));
                self.bus.write32(
                    address,
                    self.regs_r64(rs2).wrapping_add(self.regs_r64(rd)) as u32,
                );
            }
            // amoswap.w rd, rs2, rs1 # rd <= mem[rs1]; mem[rs1] <= rs2
            (F5_OP_AMO_SWAP, F3_OP_AMO_WORD) => {
                // TODO: use native atomic swap
                let address = self.regs_r64(rs1); // preserve address to avoid problem when rd == rs1
                self.regs_wi32(rd, self.bus.read32(self.regs_r64(rs1)));
                self.bus.write32(address, self.regs_r64(rs2) as u32);
            }
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
        self.pc_inc(ILEN_32B);
        Ok(())
    }

    pub fn execute_instr(&mut self, instr: u32) {
        if let Err(e) = match decode_instr(instr) {
            Opcode::LUI { uimm20, rd } => {
                self.exe_opc_lui(uimm20, rd);
                self.pc_inc(ILEN_32B);
                Ok(())
            }
            Opcode::Auipc { uimm20, rd } => self.exe_opc_auipc(uimm20, rd),
            Opcode::Branch {
                off13,
                rs2,
                rs1,
                funct3,
            } => self.exe_opc_branch(off13, rs2, rs1, funct3, ILEN_32B),
            Opcode::Jal { imm21, rd } => self.exe_opc_jal(imm21, rd),
            Opcode::Jalr { imm12, rs1, rd } => self.exe_opc_jalr(imm12, rs1, rd),
            Opcode::Fence { .. } => {
                // FENCE and FENCE.I are ignored for now
                self.pc_inc(ILEN_32B);
                Ok(())
            }
            Opcode::Load {
                imm12,
                rs1,
                funct3,
                rd,
            } => {
                let res = self.exe_opc_load(imm12, rs1, funct3, rd);
                self.pc_inc(ILEN_32B);
                res
            }
            Opcode::Store {
                imm12,
                rs2,
                rs1,
                funct3,
            } => {
                let res = self.exe_opc_store(imm12, rs2, rs1, funct3);
                self.pc_inc(ILEN_32B);
                res
            }
            Opcode::OpImm {
                imm12,
                rs1,
                funct3,
                rd,
            } => {
                let res = self.exe_opc_op_imm(imm12, rs1, funct3, rd, ILEN_32B);
                res
            }
            Opcode::ADDIW { imm12, rs1, rd } => {
                self.regs_wi32(rd, self.regs_r32(rs1).add_i12(imm12));
                self.pc_inc(ILEN_32B);
                Ok(())
            }
            Opcode::SLLIW { shamt, rs1, rd } => {
                self.regs_wi32(rd, self.regs_r32(rs1) << shamt);
                self.pc_inc(ILEN_32B);
                Ok(())
            }
            Opcode::SRLIW { shamt, rs1, rd } => {
                self.regs_wi32(rd, self.regs_r32(rs1) >> shamt);
                self.pc_inc(ILEN_32B);
                Ok(())
            }
            Opcode::Op {
                funct7,
                rs2,
                rs1,
                funct3,
                rd,
            } => {
                let res = self.exe_opc_op(funct7, rs2, rs1, funct3, rd);
                self.pc_inc(ILEN_32B);
                res
            }
            Opcode::SUBW { rs2, rs1, rd } => {
                self.regs_wi32(rd, self.regs_r32(rs1).wrapping_sub(self.regs_r32(rs2)));
                self.pc_inc(ILEN_32B);
                Ok(())
            }
            Opcode::Amo {
                funct5,
                aq,
                rl,
                rs2,
                rs1,
                funct3,
                rd,
            } => self.exe_opc_amo(funct5, aq, rl, rs2, rs1, funct3, rd),
            Opcode::System {
                csr,
                rs1,
                funct3,
                rd,
            } => self.exe_opc_system(csr, rs1, funct3, rd),
            Opcode::Uknown => Err(String::new()),
        } {
            let opc = i_opcode(instr);
            eprintln!("ERROR: Uknown instruction {e}\nPC = 0x{:x}, code: 0x{instr:08x} (0b_{instr:032b}), opcode: 0x{opc:x} (0b_{opc:07b})",
            self.get_pc());
            // TODO: trigger CPU exception
        }
        self.num_exec_instr += 1;
    }

    /// C.LI compressed instruction
    pub fn exe_opc_c_li(&mut self, imm6: I6, rd: u8) -> Result<(), String> {
        let imm6: i8 = imm6.into();
        self.regs_wi8(rd, imm6 as u8);
        self.pc_inc(ILEN_RVC);
        Ok(())
    }

    /// Execute a compressed instruction
    pub fn execute_rvc_instr(&mut self, c_instr: u16) {
        if let Err(e) = match rv64c_decode_instr(c_instr) {
            COpcode::CNOP => {
                self.pc_inc(ILEN_RVC);
                Ok(())
            }
            // TODO: HINT instructions are defined as NOP
            COpcode::Hint => {
                self.pc_inc(ILEN_RVC);
                Ok(())
            }
            COpcode::Reserved => Err("Reserved instruction".to_string()),
            // C.ADDI expands into addi rd, rd, nzimm[5:0]
            COpcode::CADDI { imm6, rd } => {
                self.exe_opc_op_imm(imm6.into(), rd, F3_OP_IMM_ADDI, rd, ILEN_RVC)
            }
            COpcode::CLUI { imm6, rd } => {
                self.exe_opc_lui(((imm6.0 as i32) << 12) as u32, rd);
                self.pc_inc(ILEN_RVC);
                Ok(())
            }
            COpcode::ADDI16SP { imm6 } => self.exe_opc_op_imm(
                I12::from((imm6.0 as i16) << 4),
                2,
                F3_OP_IMM_ADDI,
                2,
                ILEN_RVC,
            ),
            // C.SLLI rd, nzimm[5:0] expands into SLLI rd, rd, nzimm[5:0]
            COpcode::CSLLI { uimm6, rd } => {
                self.exe_opc_op_imm(I12(uimm6 as i16), rd, F3_OP_IMM_SLLI, rd, ILEN_RVC)
            }
            COpcode::CSRLI { shamt6, rd } => {
                self.exe_opc_op_imm(I12(shamt6 as i16), rd, F3_OP_IMM_SRLI, rd, ILEN_RVC)
            }
            COpcode::CLI { imm6, rd } => self.exe_opc_c_li(imm6, rd),
            // C.JR expands to JALR x0, 0(rs1)
            COpcode::CJR { rs1 } => self.exe_opc_jalr(0_u16.into(), rs1, 0),
            COpcode::CADD { rd, rs2 } => {
                let res = self.exe_opc_op(F3_OP_ADD_SUB, rs2, rd, F7_OP_ADD, rd);
                self.pc_inc(ILEN_RVC);
                res
            }
            // C.ADDW rd, rs2
            COpcode::CADDW { rd, rs2 } => {
                self.regs_wi32(rd, self.regs_r32(rd).wrapping_add(self.regs_r32(rs2)));
                self.pc_inc(ILEN_RVC);
                Ok(())
            }
            COpcode::SDSP { uimm6, rs2 } => {
                let res = self.exe_opc_store(
                    ((uimm6 as u16) << 3).into(),
                    rs2,
                    /* SP */ 2,
                    F3_OP_STORE_SD,
                );
                self.pc_inc(ILEN_RVC);
                res
            }
            COpcode::LDSP { uimm6, rd } => {
                let res = self.exe_opc_load(
                    ((uimm6 as u16) << 3).into(),
                    /* SP */ 2,
                    F3_OP_LOAD_LD,
                    rd,
                );
                self.pc_inc(ILEN_RVC);
                res
            }
            // c.ld expands to ld rd, offset[7:3](rs1).
            COpcode::LD { uoff8, rs1, rd } => {
                let res = self.exe_opc_load(uoff8.into(), rs1, F3_OP_LOAD_LD, rd);
                self.pc_inc(ILEN_RVC);
                res
            }
            // C.SW expands to SW rs2, offset[6:2](rs1)
            COpcode::SW { uoff7, rs1, rs2 } => {
                let res = self.exe_opc_store(uoff7.into(), rs2, rs1, F3_OP_STORE_SW);
                self.pc_inc(ILEN_RVC);
                res
            }
            // C.LW expands to LW rd′, offset[6:2](rs1′)
            COpcode::LW { uoff7, rs1, rd } => {
                let res = self.exe_opc_load(uoff7.into(), rs1, F3_OP_LOAD_LW, rd);
                self.pc_inc(ILEN_RVC);
                res
            }
            // c.addi4spn expands to addi rd, x2, nzuimm[9:2]
            COpcode::ADDI4SPN { uimm8, rd } => self.exe_opc_op_imm(
                I12::from((uimm8 as i16) << 2),
                2,
                F3_OP_IMM_ADDI,
                rd,
                ILEN_RVC,
            ),
            // c.mv expands to add rd, x0, rs2
            COpcode::MV { rd, rs2 } => {
                let res = self.exe_opc_op(F3_OP_ADD_SUB, rs2, 0, F7_OP_ADD, rd);
                self.pc_inc(ILEN_RVC);
                res
            }
            // c.addiw expands to addiw rd, rd, imm[5:0]
            COpcode::ADDIW { rd, uimm6 } => {
                self.regs_wi32(rd, self.regs_r32(rd).add_i12(uimm6.into()));
                self.pc_inc(ILEN_RVC);
                Ok(())
            }
            COpcode::COR { rd, rs2 } => {
                self.regs_w64(rd, self.regs_r64(rd) | self.regs_r64(rs2));
                self.pc_inc(ILEN_RVC);
                Ok(())
            }
            COpcode::CAND { rd, rs2 } => {
                self.regs_w64(rd, self.regs_r64(rd) & self.regs_r64(rs2));
                self.pc_inc(ILEN_RVC);
                Ok(())
            }
            COpcode::CANDI { imm6, rd } => {
                self.regs_w64(rd, self.regs_r64(rd) & u64::from(imm6));
                self.pc_inc(ILEN_RVC);
                Ok(())
            }
            // C.J expands to jal x0, offset[11:1].
            COpcode::CJ { imm12 } => self.exe_opc_jal(imm12.into(), /* rd = x0 */ 0),
            // c.beqz expands to beq rs1′, x0, offset[8:1].
            COpcode::BEQZ { imm9, rs1 } => {
                self.exe_opc_branch(imm9.into(), 0, rs1, F3_BRANCH_BEQ, ILEN_RVC)
            }
            // c.bnez expands to bne rs1′, x0, offset[8:1].
            COpcode::BNEZ { imm9, rs1 } => {
                self.exe_opc_branch(imm9.into(), 0, rs1, F3_BRANCH_BNE, ILEN_RVC)
            }
            COpcode::Uknown => Err(String::new()),
        } {
            let opc = c_i_opcode(c_instr);
            eprint!(
                "ERROR: Unknown RVC instruction {e}\nPC = 0x{:x}, code: 0x{c_instr:04x} ",
                self.get_pc()
            );
            eprintln!("(0b_{c_instr:016b}), opcode: 0x{opc:02x} (0b_{opc:05b})");
            // TODO: trigger CPU exception
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
