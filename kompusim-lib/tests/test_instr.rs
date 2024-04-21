use kompusim::bus::Bus;
use kompusim::rv64i_cpu::RV64ICpu;

#[test]
fn registers_writes() {
    let mut cpu = RV64ICpu::default();
    // test sign extension
    cpu.regs_wi32(1, 0x_8000_0000);
    assert!(cpu.regs_r64(1) == 0xffff_ffff_8000_0000);
}

#[test]
fn test_instruction_csrrs() {
    let mut cpu = RV64ICpu::default();
    // pollute x5
    cpu.regs_w64(5, 1);
    // csrrs  x5, mhartid, zero
    cpu.execute_instr(0xf14022f3);
    assert_eq!(cpu.regs_r64(5), 0);
}

// csrrwi	zero,mscratch,0
#[test]
fn test_csrrwi() {
    let mut cpu = RV64ICpu::default();
    cpu.execute_instr(0x_3400_5073);
}

// csrrw x1, mtvec, x4
#[test]
fn test_csrrw() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(4, 0x_dead_c0de);
    assert_eq!(cpu.regs_r64(1), 0x0);
    // csrrw x1, mtvec, x4
    cpu.execute_instr(0x_3052_10f3);
    // csrrw x1, mtvec, x4
    cpu.execute_instr(0x_3052_10f3);
    assert_eq!(cpu.regs_r64(1), 0x_dead_c0de);
}

#[test]
fn test_instruction_bne() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(5, 1);
    // BNE t0, x0, 0x10
    cpu.execute_instr(0x00029863);
    assert_eq!(cpu.get_pc(), 0x10);
}

#[test]
fn test_instruction_lui() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(5, 0x123);
    // lui x5, 0x10010
    cpu.execute_instr(0x100102b7);
    assert_eq!(cpu.regs_r64(5), 0x10010000);
}

#[test]
fn test_instruction_auipc() {
    let mut cpu = RV64ICpu::default();
    cpu.pc_jump(0x100);
    cpu.regs_w64(10, 0x123);
    // auipc x10, 0x0
    cpu.execute_instr(0x00000517);
    assert_eq!(cpu.regs_r64(10), 0x100);
}

#[test]
// addi x10, x10, 52
fn test_instruction_addi() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(10, 0x123);
    cpu.execute_instr(0x03450513);
    assert_eq!(cpu.regs_r64(10), 0x123 + 52);
}

#[test]
// addiw x6, x0, 0x1
fn test_instruction_addiw() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(6, 0x123);
    cpu.execute_instr(0x_0010_031b);
    assert_eq!(cpu.regs_r64(6), 1);
}

#[test]
// jal ra, 80000018
fn test_instruction_jal() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(5, 1);
    cpu.pc_jump(0x80000010);
    cpu.execute_instr(0x008000ef);
    assert_eq!(cpu.get_pc(), 0x80000018);
}

#[test]
// jal  zero,8000001c
fn test_instruction_jal_2() {
    let mut cpu = RV64ICpu::default();
    cpu.pc_jump(0x80000034);
    cpu.execute_instr(0xfe9ff06f);
    assert_eq!(cpu.get_pc(), 0x8000001c);
}

#[test]
// jalr x0, 0x0(x1)
fn test_instruction_jalr() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(1, 0x20);
    cpu.execute_instr(0x00008067);
    assert_eq!(cpu.get_pc(), 0x20);
}

#[test]
// lbu x6, 0x0(x10)
fn test_instruction_lbu() {
    let mut bus = Bus::new_with_ram(0x0000_0000_8000_0000, 4 * 1024);
    bus.write8(0x0000_0000_8000_003c, 0x48);
    let mut cpu = RV64ICpu::new(bus);

    cpu.regs_w64(6, 0xa5a5_a5a5_a5a5_a5a5);
    cpu.regs_w64(10, 0x0000_0000_8000_003c);
    cpu.execute_instr(0x00054303);
    assert!(cpu.regs_r64(6) == 0x48);
}

// TODO: lb test
#[test]
fn test_instruction_lb() {}

#[test]
// lw x7, 0x0(x5)
fn test_instruction_lw() {
    let mut bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    bus.write32(0x0000_0000_0000_0000, 0xa5a5_a5a5);
    let mut cpu = RV64ICpu::new(bus);
    cpu.regs_w64(7, 0xdead_beef_dead_beef);
    cpu.execute_instr(0x0002a383);
    // lw sign extends 32-bit word
    assert!(cpu.regs_r64(7) == 0xffff_ffff_a5a5_a5a5);
}

#[test]
// sw x6, 0x0(x5)
fn test_instruction_sw() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);
    cpu.regs_w64(5, 0x10); // address
    cpu.regs_w64(6, 0xdead_beef); // what to store
    cpu.execute_instr(0x0062a023);
    // lw sign extends 32-bit word
    assert!(cpu.bus.read32(0x10) == 0xdead_beef);
}

#[test]
// beq x6, x0, 0x00000018
fn test_instruction_beq() {
    let mut cpu = RV64ICpu::default();
    // equal
    cpu.regs_w64(6, 0);
    // pc = 0, offset = 18
    cpu.execute_instr(0x00030c63);
    assert_eq!(cpu.get_pc(), 0x18);

    // not equal
    cpu.regs_w64(6, 1);
    cpu.execute_instr(0x00030c63);
    // pc = 0x18 + 4
    assert_eq!(cpu.get_pc(), 0x1c);
}

#[test]
// blt x7, x0, -4
fn test_instruction_blt() {
    let mut cpu = RV64ICpu::default();
    cpu.pc_jump(0x4);

    // less
    cpu.regs_w64(7, -1_i64 as u64);
    cpu.execute_instr(0xfe03cee3);
    // pc = 0x4 - 4
    assert_eq!(cpu.get_pc(), 0x0);

    // equal
    cpu.regs_w64(7, 0);
    cpu.execute_instr(0xfe03cee3);
    // pc = 0x0 + 4
    assert_eq!(cpu.get_pc(), 0x4);
}

#[test]
// add x8, x10, x0
fn test_instruction_add() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(10, 0x123);
    cpu.execute_instr(0x0005_0433);
    assert_eq!(cpu.regs_r64(8), 0x123);
}

#[test]
// sub x1, x1, x1
fn test_instruction_sub() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(1, 0x123);
    cpu.execute_instr(0x_4010_80b3);
    assert_eq!(cpu.regs_r64(1), 0);
}

#[test]
// lr.w x1, (x0)
fn test_instruction_lrw() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);
    cpu.bus.write32(0x0, 0x0000_beef);
    cpu.execute_instr(0x_1000_20af);
    assert_eq!(cpu.regs_r64(1), 0x0000_beef);
}

#[test]
// amoswap.w.aq rd, rs2, rs1 # rd <= mem[rs1]; mem[rs1] <= rs2
fn test_amoswap() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);
    cpu.bus.write32(0x0, 0x0000_beef);
    assert!(cpu.bus.read32(0x0) == 0x0000_beef);
    cpu.regs_w64(5, 0xc0fe);
    // amoswap.w.aq  x6, x5, (x10) # x6 <= mem[x10]; mem[x10] <= x5
    cpu.execute_instr(0x_0c55_232f);
    assert_eq!(cpu.regs_r64(6), 0x0000_beef);
    assert_eq!(cpu.bus.read32(0x0), 0xc0fe);
}

#[test]
// amoadd.w.aq x2, x1, (x0)
fn test_amoadd() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);
    cpu.bus.write32(0x0, 0x0000_0001);
    cpu.regs_w64(1, 0x1);
    // amoadd.w rd, rs2, rs1 # rd <= mem[rs1]; mem[rs1] <= rd + rs2
    // amoadd.w.aq x2, x1, (x0)
    cpu.execute_instr(0x_0410_212f);
    assert_eq!(cpu.regs_r64(2), 0x1);
    assert_eq!(cpu.bus.read32(0x0), 0x0000_0002);
}

#[test]
// amoadd.w x16, x17, (x16)
fn test_amoadd_rd_equals_rs1() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);
    cpu.bus.write32(0x0, 0x0000_0001);
    cpu.regs_w64(17, 0x2);
    // amoadd.w rd,  rs2, (rs1) # rd <= mem[rs1]; mem[rs1] <= rd + rs2
    // amoadd.w x16, x17, (x16)
    cpu.execute_instr(0x_0118_282f);
    assert_eq!(cpu.regs_r64(16), 0x1);
    assert_eq!(cpu.bus.read32(0x0), 0x0000_0003);
}

// sd x6, 0x0(x5)
#[test]
fn test_sd() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);
    assert!(cpu.bus.read32(0x10) == 0);
    cpu.regs_w64(5, 0x10); // address
    cpu.regs_w64(6, 0x_badc_0ffe_dead_beef); // what to store
    cpu.execute_instr(0x0062_b023);
    assert!(cpu.bus.read32(0x10) == 0x_dead_beef);
    assert!(cpu.bus.read32(0x14) == 0x_badc0ffe);
}

// ld x6, 0x0(x5)
#[test]
fn test_ld() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);
    assert!(cpu.bus.read32(0x10) == 0);
    cpu.regs_w64(5, 0x10); // address
    cpu.bus.write64(0x10, 0x_badc_0ffe_dead_beef);
    cpu.execute_instr(0x_0002_b303);
    assert_eq!(cpu.regs_r64(6), 0x_badc_0ffe_dead_beef);
}

// fence rw,rw
#[test]
fn test_fence() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);
    cpu.execute_instr(0x_0330_000f);
    // no effect for now
}

// slliw rd, rs1, uimm5
#[test]
fn test_slliw() {
    let mut cpu = RV64ICpu::default();

    cpu.regs_w64(15, 1);
    // slliw	x15, x15, 0x18
    cpu.execute_instr(0x_0187_979b);
    assert_eq!(cpu.regs_r64(15), 1 << 0x18);

    cpu.regs_w64(15, -1_i64 as u64);
    // slliw	x15, x15, 0x18
    cpu.execute_instr(0x_0187_979b);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_ff00_0000); // shift left 24 bits
}

// #[test]
// fn test_intermixed_instruction {
//     // TODO:
//     // compressed, 32-bit, compressed
// }
