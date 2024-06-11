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
    assert_eq!(cpu.get_pc(), 4);
}

// csrrwi rd, csr, uimm5
#[test]
fn test_csrrwi() {
    let mut cpu = RV64ICpu::default();
    // csrrwi x0, mscratch, 0
    cpu.execute_instr(0x_3400_5073);
    assert_eq!(cpu.get_pc(), 4);
}

// csrrw rd, csr, rs1
#[test]
fn test_csrrw() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(4, 0x_dead_c0de);
    assert_eq!(cpu.regs_r64(1), 0x0);
    // csrrw x1, mtvec, x4
    cpu.execute_instr(0x_3052_10f3);
    assert_eq!(cpu.get_pc(), 4);
    // csrrw x1, mtvec, x4
    cpu.execute_instr(0x_3052_10f3);
    assert_eq!(cpu.regs_r64(1), 0x_dead_c0de);
    assert_eq!(cpu.get_pc(), 8);
}

#[test]
fn test_instruction_bne() {
    let mut cpu = RV64ICpu::default();
    // BNE t0, x0, 0x10
    cpu.execute_instr(0x00029863);
    assert_eq!(cpu.get_pc(), 4);

    cpu.regs_w64(5, 1);
    // BNE t0, x0, 0x10
    cpu.execute_instr(0x00029863);
    assert_eq!(cpu.get_pc(), 4 + 0x10);
}

#[test]
// lui rd, imm20
fn test_instruction_lui() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(5, 0x123);
    // lui x5, 0x10010
    cpu.execute_instr(0x_1001_02b7);
    assert_eq!(cpu.regs_r64(5), 0x10010000);

    // lui x1, 0x0
    cpu.execute_instr(0x_0000_00b7);
    assert_eq!(cpu.regs_r64(1), 0x_0000_0000_0000_0000);

    // lui x1, 0xfffff
    cpu.execute_instr(0x_ffff_f0b7);
    assert_eq!(cpu.regs_r64(1), 0x_ffff_ffff_ffff_f000);

    // lui x1, 0x7ffff
    cpu.execute_instr(0x_7fff_f0b7);
    assert_eq!(cpu.regs_r64(1), 0x_0000_0000_7fff_f000);

    // lui x1, 0x80000
    cpu.execute_instr(0x_8000_00b7);
    assert_eq!(cpu.regs_r64(1), 0x_ffff_ffff_8000_0000);

    assert_eq!(cpu.get_pc(), 5 * 4);
}

#[test]
// auipc rd, imm20
fn test_instruction_auipc() {
    let mut cpu = RV64ICpu::default();
    cpu.pc_jump(0x100);
    cpu.regs_w64(10, 0x123);
    // auipc x10, 0x0
    cpu.execute_instr(0x00000517);
    assert_eq!(cpu.regs_r64(10), 0x100);
    assert_eq!(cpu.get_pc(), 0x100 + 4);

    // auipc x10, 0xfffff
    cpu.execute_instr(0x_ffff_f517);
    assert_eq!(cpu.regs_r64(10), 0x104 + 0x_ffff_ffff_ffff_f000);
}

#[test]
// addi x10, x10, 52
fn test_instruction_addi() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(10, 0x123);
    cpu.execute_instr(0x03450513);
    assert_eq!(cpu.regs_r64(10), 0x123 + 52);
    assert_eq!(cpu.get_pc(), 4);
}

#[test]
// addiw rd, rs1, imm12
fn test_instruction_addiw() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(6, 0x123);
    // addiw x6, x0, 0x1
    cpu.execute_instr(0x_0010_031b);
    assert_eq!(cpu.regs_r64(6), 1);

    cpu.regs_w64(13, 0);
    // addiw x14, x13, 0
    cpu.execute_instr(0x_0006_871b);
    assert_eq!(cpu.regs_r64(14), 0);

    cpu.regs_w64(13, 1);
    // addiw x14, x13, 1
    cpu.execute_instr(0x_0016_871b);
    assert_eq!(cpu.regs_r64(14), 2);

    cpu.regs_w64(13, 3);
    // addiw x14, x13, 7
    cpu.execute_instr(0x_0076_871b);
    assert_eq!(cpu.regs_r64(14), 10);

    cpu.regs_w64(13, 0);
    // addiw x14, x13, -2048
    cpu.execute_instr(0x_8006_871b);
    assert_eq!(cpu.regs_r64(14), 0x_ffff_ffff_ffff_f800);

    cpu.regs_w64(13, 0x_ffff_ffff_8000_0000);
    // addiw x14, x13, 0
    cpu.execute_instr(0x_0006_871b);
    assert_eq!(cpu.regs_r64(14), 0x_ffff_ffff_8000_0000);

    cpu.regs_w64(13, 0x_ffff_ffff_8000_0000);
    // addiw x14, x13, -2048
    cpu.execute_instr(0x_8006_871b);
    assert_eq!(cpu.regs_r64(14), 0x_0000_0000_7fff_f800);

    cpu.regs_w64(13, 0);
    // addiw x14, x13, 2047
    cpu.execute_instr(0x_7ff6_871b);
    assert_eq!(cpu.regs_r64(14), 0x_0000_0000_0000_07ff);

    cpu.regs_w64(13, 0x_8000_0000);
    // addiw x13, x13, -1
    cpu.execute_instr(0x_fff6_869b);
    // addiw x14, x13, 0
    cpu.execute_instr(0x_0006_871b);
    assert_eq!(cpu.regs_r64(14), 0x_0000_0000_7fff_ffff);

    cpu.regs_w64(13, 0x_ffff_ffff_8000_0000);
    // addiw x14, x13, 2047
    cpu.execute_instr(0x_7ff6_871b);
    assert_eq!(cpu.regs_r64(14), 0x_ffff_ffff_8000_07ff);

    cpu.regs_w64(13, 0x_0000_0000_7fff_ffff);
    // addiw x14, x13, -2048
    cpu.execute_instr(0x_8006_871b);
    assert_eq!(cpu.regs_r64(14), 0x_0000_0000_7fff_f7ff);

    cpu.regs_w64(13, 0x_0000_0000_0000_0000);
    // addiw x14, x13, -1
    cpu.execute_instr(0x_fff6_871b);
    assert_eq!(cpu.regs_r64(14), 0x_ffff_ffff_ffff_ffff);

    cpu.regs_w64(13, 0x_ffff_ffff_ffff_ffff);
    // addiw x14, x13, 1
    cpu.execute_instr(0x_0016_871b);
    assert_eq!(cpu.regs_r64(14), 0x_0000_0000_0000_0000);

    cpu.regs_w64(13, 0x_ffff_ffff_ffff_ffff);
    // addiw x14, x13, -1
    cpu.execute_instr(0x_fff6_871b);
    assert_eq!(cpu.regs_r64(14), 0x_ffff_ffff_ffff_fffe);

    cpu.regs_w64(13, 0x_0000_0000_7fff_ffff);
    // addiw x14, x13, 1
    cpu.execute_instr(0x_0016_871b);
    assert_eq!(cpu.regs_r64(14), 0x_ffff_ffff_8000_0000);

    assert_eq!(cpu.get_pc(), 16 * 4);
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
    assert_eq!(cpu.get_pc(), 4);
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
    cpu.regs_w64(7, 0x_dead_beef_dead_beef);
    cpu.execute_instr(0x_0002_a383);
    // lw sign extends 32-bit word
    assert_eq!(cpu.regs_r64(7), 0xffff_ffff_a5a5_a5a5);
    assert_eq!(cpu.get_pc(), 4);
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
    assert_eq!(cpu.get_pc(), 4);
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
    assert_eq!(cpu.get_pc(), 4);
}

#[test]
// sub x1, x1, x1
fn test_instruction_sub() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(1, 0x123);
    cpu.execute_instr(0x_4010_80b3);
    assert_eq!(cpu.regs_r64(1), 0);
    assert_eq!(cpu.get_pc(), 4);
}

#[test]
// lr.w x1, (x0)
fn test_instruction_lrw() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);
    cpu.bus.write32(0x0, 0x0000_beef);
    cpu.execute_instr(0x_1000_20af);
    assert_eq!(cpu.regs_r64(1), 0x0000_beef);
    assert_eq!(cpu.get_pc(), 4);
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
    assert_eq!(cpu.get_pc(), 4);
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
    assert_eq!(cpu.get_pc(), 4);
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
    assert_eq!(cpu.get_pc(), 4);
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
    assert_eq!(cpu.get_pc(), 4);
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
    assert_eq!(cpu.get_pc(), 4);
}

// fence rw,rw
#[test]
fn test_fence() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);
    cpu.execute_instr(0x_0330_000f);
    // no effect for now
    assert_eq!(cpu.get_pc(), 4);
}

// slliw rd, rs1, uimm5
#[test]
fn test_slliw() {
    let mut cpu = RV64ICpu::default();

    cpu.regs_w64(15, 1);
    // slliw x15, x15, 0x18
    cpu.execute_instr(0x_0187_979b);
    assert_eq!(cpu.regs_r64(15), 1 << 0x18);

    cpu.regs_w64(15, -1_i64 as u64);
    // slliw x15, x15, 0x18
    cpu.execute_instr(0x_0187_979b);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_ff00_0000); // shift left 24 bits

    cpu.regs_w64(15, 0x_0000_0000_0000_00d0);
    // slliw x15, x15, 0x18
    cpu.execute_instr(0x_0187_979b);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_d000_0000);

    cpu.regs_w64(15, 0x_0000_0000_0000_0001);
    // slliw x15, x15, 0
    cpu.execute_instr(0x_0007_979b);
    assert_eq!(cpu.regs_r64(15), 0x_0000_0000_0000_0001);

    cpu.regs_w64(15, 0x_0000_0000_0000_0001);
    // slliw x15, x15, 1
    cpu.execute_instr(0x_0017_979b);
    assert_eq!(cpu.regs_r64(15), 0x_0000_0000_0000_0002);

    cpu.regs_w64(15, 0x_0000_0000_0000_0001);
    // slliw x15, x15, 7
    cpu.execute_instr(0x_0077_979b);
    assert_eq!(cpu.regs_r64(15), 0x_0000_0000_0000_0080);

    cpu.regs_w64(15, 0x_0000_0000_0000_0001);
    // slliw x15, x15, 14
    cpu.execute_instr(0x_00e7_979b);
    assert_eq!(cpu.regs_r64(15), 0x_0000_0000_0000_4000);

    cpu.regs_w64(15, 0x_0000_0000_0000_0001);
    // slliw x15, x15, 31
    cpu.execute_instr(0x_01f7_979b);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_8000_0000);

    cpu.regs_w64(15, 0x_ffff_ffff_ffff_ffff);
    // slliw x15, x15, 0
    cpu.execute_instr(0x_0007_979b);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_ffff_ffff);

    cpu.regs_w64(15, 0x_ffff_ffff_ffff_ffff);
    // slliw x15, x15, 1
    cpu.execute_instr(0x_0017_979b);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_ffff_fffe);

    cpu.regs_w64(15, 0x_ffff_ffff_ffff_ffff);
    // slliw x15, x15, 7
    cpu.execute_instr(0x_0077_979b);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_ffff_ff80);

    cpu.regs_w64(15, 0x_ffff_ffff_ffff_ffff);
    // slliw x15, x15, 14
    cpu.execute_instr(0x_00e7_979b);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_ffff_c000);

    cpu.regs_w64(15, 0x_ffff_ffff_ffff_ffff);
    // slliw x15, x15, 31
    cpu.execute_instr(0x_01f7_979b);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_8000_0000);

    cpu.regs_w64(15, 0x_0000_0000_2121_2121);
    // slliw x15, x15, 0
    cpu.execute_instr(0x_0007_979b);
    assert_eq!(cpu.regs_r64(15), 0x_0000_0000_2121_2121);

    cpu.regs_w64(15, 0x_0000_0000_2121_2121);
    // slliw x15, x15, 1
    cpu.execute_instr(0x_0017_979b);
    assert_eq!(cpu.regs_r64(15), 0x_0000_0000_4242_4242);

    cpu.regs_w64(15, 0x_0000_0000_2121_2121);
    // slliw x15, x15, 7
    cpu.execute_instr(0x_0077_979b);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_9090_9080);

    cpu.regs_w64(15, 0x_0000_0000_2121_2121);
    // slliw x15, x15, 14
    cpu.execute_instr(0x_00e7_979b);
    assert_eq!(cpu.regs_r64(15), 0x_0000_0000_4848_4000);

    cpu.regs_w64(15, 0x_0000_0000_2121_2121);
    // slliw x15, x15, 31
    cpu.execute_instr(0x_01f7_979b);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_8000_0000);

    cpu.regs_w64(15, 0x_ffff_ffff_1234_5678);
    // slliw x15, x15, 0
    cpu.execute_instr(0x_0007_979b);
    assert_eq!(cpu.regs_r64(15), 0x_0000_0000_1234_5678);

    cpu.regs_w64(15, 0x_ffff_ffff_1234_5678);
    // slliw x15, x15, 4
    cpu.execute_instr(0x_0047_979b);
    assert_eq!(cpu.regs_r64(15), 0x_0000_0000_2345_6780);

    cpu.regs_w64(15, 0x_0000_0000_9234_5678);
    // slliw x15, x15, 0
    cpu.execute_instr(0x_0007_979b);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_9234_5678);

    cpu.regs_w64(15, 0x_0000_0000_9934_5678);
    // slliw x15, x15, 4
    cpu.execute_instr(0x_0047_979b);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_9345_6780);

    assert_eq!(cpu.get_pc(), 22 * 4);
}

// bge rs1, rs2, offset13
#[test]
fn test_bge() {
    let mut cpu = RV64ICpu::default();

    cpu.regs_w64(15, -1_i64 as u64);
    // bge x15, x0, 0x1f314
    cpu.execute_instr(0x_0007_d563);
    assert_eq!(cpu.get_pc(), 4);

    cpu.regs_w64(15, 1);
    // bge x15, x0, 0x1f314 # offset13 == 0xa
    cpu.execute_instr(0x_0007_d563);
    // pc == 4
    assert_eq!(cpu.get_pc(), 4 + 0xa);
}

// Branch Less Than (Unsigned comparison)
// bltu rs1, rs2, PC+offset13
#[test]
fn test_bltu() {
    let mut cpu = RV64ICpu::default();

    // Negative case: x15 == 0, x14 == 0
    // bltu x15, x14, 8001dd46 # offset13 = 8
    cpu.execute_instr(0x_00e7_e463);
    assert_eq!(cpu.get_pc(), 4);

    // Positive case: x15 == 0, x14 == 1
    cpu.regs_w64(14, 1);
    // bltu x15, x14, 8001dd46 # offset13 = 8
    cpu.execute_instr(0x_00e7_e463);
    assert_eq!(cpu.get_pc(), 4 + 8); // PC + offset
}

#[test]
// Branch if Greater or Equal (Unsigned comparison)
// bgeu x15, x14, PC+offset13
fn test_bgeu() {
    let mut cpu = RV64ICpu::default();

    // Negative case: x15 == 0, x14 == 1
    cpu.regs_w64(14, 1);
    // bgeu x15, x14, 0x8001dda4 # offset13 = 74
    cpu.execute_instr(0x_04e7_f563);
    assert_eq!(cpu.get_pc(), 4);

    // Positive case: x15 == 0, x14 == 0
    cpu.regs_w64(14, 0);
    // bgeu x15, x14, 0x8001dda4 # offset13 = 74
    cpu.execute_instr(0x_04e7_f563);
    assert_eq!(cpu.get_pc(), 4 + 74); // PC + offset13
}

// Shift Right Logical Immidiate Word
// srliw rd, rs1, shamt5
#[test]
fn test_srliw() {
    let mut cpu = RV64ICpu::default();

    cpu.regs_w64(15, 0x_ffff_ffff_0123_4567);
    // srliw x15, x15, 0x8
    cpu.execute_instr(0x_0087_d79b);
    assert_eq!(cpu.regs_r64(15), 0x_0000_0000_0001_2345);

    cpu.regs_w64(13, 0x_ffff_ffff_8000_0000);
    // srliw x14, x13, 0x0
    cpu.execute_instr(0x_0006_d71b);
    assert_eq!(cpu.regs_r64(14), 0x_ffff_ffff_8000_0000);

    cpu.regs_w64(13, 0x_ffff_ffff_8000_0000);
    // srliw x14, x13, 0x1
    cpu.execute_instr(0x_0016_d71b);
    assert_eq!(cpu.regs_r64(14), 0x_0000_0000_4000_0000);

    cpu.regs_w64(13, 0x_ffff_ffff_ffff_ffff);
    // srliw x14, x13, 0x1f
    cpu.execute_instr(0x_01f6_d71b);
    assert_eq!(cpu.regs_r64(14), 0x_0000_0000_0000_0001);

    cpu.regs_w64(13, 0x_ffff_ffff_1234_5678);
    // srliw x14, x13, 0x0
    cpu.execute_instr(0x_0006_d71b);
    assert_eq!(cpu.regs_r64(14), 0x_0000_0000_1234_5678);

    cpu.regs_w64(13, 0x_ffff_ffff_1234_5678);
    // srliw x14, x13, 0x4
    cpu.execute_instr(0x_0046_d71b);
    assert_eq!(cpu.regs_r64(14), 0x_0000_0000_0123_4567);

    cpu.regs_w64(13, 0x_0000_0000_9234_5678);
    // srliw x14, x13, 0x0
    cpu.execute_instr(0x_0006_d71b);
    assert_eq!(cpu.regs_r64(14), 0x_ffff_ffff_9234_5678);

    assert_eq!(cpu.get_pc(), 7 * 4);
}

// Subtraction Word
// subw rd, rs1, rs2
#[test]
fn test_subw() {
    let mut cpu = RV64ICpu::default();

    fn subw(cpu: &mut RV64ICpu, result: u64, v1: u64, v2: u64) {
        cpu.regs_w64(14, v1);
        cpu.regs_w64(15, v2);
        // subw x15, x14, x15
        cpu.execute_instr(0x_40f7_07bb);
        assert_eq!(cpu.regs_r64(15), result);
    }
    subw(
        &mut cpu,
        0x_0000_0000_0000_5678,
        0x_ffff_ffff_1234_5678,
        0x_0000_0000_1234_0000,
    );

    subw(
        &mut cpu,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    );
    subw(
        &mut cpu,
        0x0000000000000000,
        0x0000000000000001,
        0x0000000000000001,
    );
    subw(
        &mut cpu,
        0xfffffffffffffffc,
        0x0000000000000003,
        0x0000000000000007,
    );
    subw(
        &mut cpu,
        0x0000000000008000,
        0x0000000000000000,
        0xffffffffffff8000,
    );
    subw(
        &mut cpu,
        0xffffffff80000000,
        0xffffffff80000000,
        0x0000000000000000,
    );
    subw(
        &mut cpu,
        0xffffffff80008000,
        0xffffffff80000000,
        0xffffffffffff8000,
    );
    subw(
        &mut cpu,
        0xffffffffffff8001,
        0x0000000000000000,
        0x0000000000007fff,
    );
    subw(
        &mut cpu,
        0x000000007fffffff,
        0x000000007fffffff,
        0x0000000000000000,
    );
    subw(
        &mut cpu,
        0x000000007fff8000,
        0x000000007fffffff,
        0x0000000000007fff,
    );
    subw(
        &mut cpu,
        0x000000007fff8001,
        0xffffffff80000000,
        0x0000000000007fff,
    );
    subw(
        &mut cpu,
        0xffffffff80007fff,
        0x000000007fffffff,
        0xffffffffffff8000,
    );
    subw(
        &mut cpu,
        0x0000000000000001,
        0x0000000000000000,
        0xffffffffffffffff,
    );
    subw(
        &mut cpu,
        0xfffffffffffffffe,
        0xffffffffffffffff,
        0x0000000000000001,
    );
    subw(
        &mut cpu,
        0x0000000000000000,
        0xffffffffffffffff,
        0xffffffffffffffff,
    );

    assert_eq!(cpu.get_pc(), 15 * 4);
}

// Shift Left Logical Immidiate
// slli rd, rs1, shamt6
#[test]
fn test_slli() {
    let mut cpu = RV64ICpu::default();

    cpu.regs_w64(15, 0x_3fff_ffff_ffff_ffff);
    // slli x14, x15, 0x2
    cpu.execute_instr(0x_0027_9713);
    assert_eq!(cpu.regs_r64(14), 0x_ffff_ffff_ffff_fffc);
    assert_eq!(cpu.get_pc(), 4);
}

// xori rd, rs1, imm12
#[test]
fn test_xori() {
    let mut cpu = RV64ICpu::default();

    cpu.regs_w64(15, 0x_0000_0000_0000_0001);
    // xori x15, x15, -2
    cpu.execute_instr(0x_ffe7_c793);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_ffff_ffff);
}

// Wait For Interrupt
// wfi
#[test]
fn test_wfi() {
    let mut cpu = RV64ICpu::default();
    // wfi
    cpu.execute_instr(0x_1050_0073);
    // no effect
    assert_eq!(cpu.get_pc(), 4);
}

// #[test]
// fn test_intermixed_instruction {
//     // TODO:
//     // compressed, 32-bit, compressed
// }
