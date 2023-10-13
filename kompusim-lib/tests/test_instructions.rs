use kompusim::bus::Bus;
use kompusim::rv64i_cpu::RV64ICpu;

#[test]
fn test_instruction_csrrs() {
    let mut cpu = RV64ICpu::default();
    // pollute x5
    cpu.regs.x[5] = 1;
    // csrrs  x5, mhartid, zero
    cpu.execute_instr(0xf14022f3);
    assert!(cpu.regs.x[5] == 0);
}

#[test]
fn test_instruction_bne() {
    let mut cpu = RV64ICpu::default();
    cpu.regs.x[5] = 1;
    // BNE t0, x0, 0x10
    cpu.execute_instr(0x00029863);
    assert!(cpu.regs.pc == 0x10);
}

#[test]
fn test_instruction_lui() {
    let mut cpu = RV64ICpu::default();
    cpu.regs.x[5] = 0x123;
    // lui x5, 0x10010
    cpu.execute_instr(0x100102b7);
    assert!(cpu.regs.x[5] == 0x10010000);
}

#[test]
fn test_instruction_auipc() {
    let mut cpu = RV64ICpu::default();
    cpu.regs.pc = 0x100;
    cpu.regs.x[10] = 0x123;
    // auipc x10, 0x0
    cpu.execute_instr(0x00000517);
    assert!(cpu.regs.x[10] == 0x100);
}

#[test]
// addi x10, x10, 52
fn test_instruction_addi() {
    let mut cpu = RV64ICpu::default();
    cpu.regs.x[10] = 0x123;
    cpu.execute_instr(0x03450513);
    assert!(cpu.regs.x[10] == 0x123 + 52);
}

#[test]
// jal ra, 80000018
fn test_instruction_jal() {
    let mut cpu = RV64ICpu::default();
    cpu.regs.x[5] = 1;
    cpu.regs.pc = 0x80000010;
    cpu.execute_instr(0x008000ef);
    assert!(cpu.regs.pc == 0x80000018);
}

#[test]
// jalr x0, 0x0(x1)
fn test_instruction_jalr() {
    let mut cpu = RV64ICpu::default();
    cpu.regs.x[1] = 0x20;
    cpu.execute_instr(0x00008067);
    assert!(cpu.regs.pc == 0x20);
}

#[test]
// lbu x6, 0x0(x10)
fn test_instruction_lbu() {
    let mut bus = Bus::new_with_ram(0x0000_0000_8000_0000, 4 * 1024);
    bus.write8(0x00000000_8000_003c, 0x48);
    let mut cpu = RV64ICpu::new(bus);

    cpu.regs_w64(6, 0xa5a5a5a5_a5a5_a5a5);
    cpu.regs_w64(10, 0x00000000_8000_003c);
    cpu.execute_instr(0x00054303);
    assert!(cpu.regs_r64(6) == 0x48);
}

// TODO: lb test
#[test]
fn test_instruction_lb() {}

#[test]
// lw x7, 0x0(x5)
fn test_instruction_lw() {
    let mut bus = Bus::new_with_ram(0x00000000_0000_0000, 4 * 1024);
    bus.write32(0x00000000_0000_0000, 0xa5a5_a5a5);
    let mut cpu = RV64ICpu::new(bus);
    cpu.regs_w64(7, 0xdead_beef_dead_beef);
    cpu.execute_instr(0x0002a383);
    // lw sign extends 32-bit word
    assert!(cpu.regs_r64(7) == 0xffff_ffff_a5a5_a5a5);
}

#[test]
// sw x6, 0x0(x5)
fn test_instruction_sw() {
    let bus = Bus::new_with_ram(0x00000000_0000_0000, 4 * 1024);
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
// blt x7, x0, -4
fn test_instruction_blt() {
    let mut cpu = RV64ICpu::default();
    cpu.regs.pc = 0x4;

    // less
    cpu.regs.x[7] = -1_i64 as u64;
    cpu.execute_instr(0xfe03cee3);
    // pc = 0x4 - 4
    assert!(cpu.regs.pc == 0x0);

    // equal
    cpu.regs.x[7] = 0;
    cpu.execute_instr(0xfe03cee3);
    // pc = 0x0 + 4
    assert!(cpu.regs.pc == 0x4);
}

#[test]
fn registers_writes() {
    let mut cpu = RV64ICpu::default();
    // test sign extension
    cpu.regs_wi32(1, 0x_8000_0000);
    assert!(cpu.regs_r64(1) == 0xffff_ffff_8000_0000);
}

// #[test]
// fn test_intermixed_instruction {
//     // TODO:
//     // compressed, 32-bit, compressed
// }
