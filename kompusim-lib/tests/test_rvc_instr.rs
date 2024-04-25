use kompusim::{bus::Bus, rv64i_cpu::RV64ICpu};

#[test]
// c.li rd, imm6
fn test_rvc_instr_c_li() {
    let mut cpu = RV64ICpu::default();
    assert_eq!(cpu.regs_r64(1), 0);
    // c.li x1, 1
    cpu.execute_rvc_instr(0x_4085);
    assert_eq!(cpu.regs_r64(1), 1);

    // c.li x10, -1
    cpu.execute_rvc_instr(0x_557d);
    assert_eq!(cpu.regs_r64(10), (-1_i64 as u64));
    assert_eq!(cpu.get_pc(), 4);
}

#[test]
// c.jr x1
fn test_rvc_instr_c_jr() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(1, 0x54);
    cpu.execute_rvc_instr(0x_8082);
    assert_eq!(cpu.get_pc(), 0x54);
}

#[test]
// c.add x1, x1
fn test_rvc_add() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(1, 0x1122_3344);
    cpu.execute_rvc_instr(0x_9086);
    assert_eq!(cpu.regs_r64(1), 0x2244_6688);
    assert_eq!(cpu.get_pc(), 2);
}

#[test]
// c.j offset
fn test_rvc_instr_c_j() {
    let mut cpu = RV64ICpu::default();
    cpu.pc_jump(0x_8000_003a);
    // c.j 80000024
    cpu.execute_rvc_instr(0x_b7ed);
    assert_eq!(cpu.get_pc(), 0x_8000_0024);
}

// c.beqz rs1, address
#[test]
fn test_rvc_instr_c_beqz() {
    let mut cpu = RV64ICpu::default();
    cpu.pc_jump(0x_8000_3700);
    // test negative branch
    cpu.regs_w64(15, 1);
    // c.beqz x15, 0x3718
    cpu.execute_rvc_instr(0x_cf81);
    assert_eq!(cpu.get_pc(), 0x_8000_3702);

    // test positive branch
    cpu.pc_jump(0x_8000_3700);
    cpu.regs_w64(15, 0);
    // c.beqz x15, 0x3718
    cpu.execute_rvc_instr(0x_cf81);
    assert_eq!(cpu.get_pc(), 0x_8000_3718);
}

// c.bnez rs1, pc_rel_address
#[test]
fn test_rvc_instr_c_bnez() {
    let mut cpu = RV64ICpu::default();

    // test negative branch
    cpu.pc_jump(0x_8000_3710);
    cpu.regs_w64(15, 0);
    // c.bnez x15, 0x3706
    cpu.execute_rvc_instr(0x_fbfd);
    assert_eq!(cpu.get_pc(), 0x_8000_3712);

    // test positive branch
    cpu.pc_jump(0x_8000_3710);
    cpu.regs_w64(15, 1);
    // c.bnez x15, 0x3706
    cpu.execute_rvc_instr(0x_fbfd);
    assert_eq!(cpu.get_pc(), 0x_8000_3706);
}

// c.addi a0,1
#[test]
fn test_rvc_instr_c_addi() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(10, 0x_1122_3344);
    cpu.execute_rvc_instr(0x_0505);
    assert_eq!(cpu.regs_r64(10), 0x_1122_3345);
    assert_eq!(cpu.get_pc(), 2);
}

// c.slli x6, 0x1f
#[test]
fn test_rvc_instr_slli() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(6, 0x1);
    cpu.execute_rvc_instr(0x_037e);
    assert_eq!(cpu.regs_r64(6), 0x1 << 0x1f);
    assert_eq!(cpu.get_pc(), 2);
}

#[test]
fn test_rvc_instr_c_lui() {
    let mut cpu = RV64ICpu::default();
    // c.lui	x21, 0x2
    cpu.execute_rvc_instr(0x_6a89);
    assert!(cpu.regs_r64(21) == 2 * 4096);
    // c.lui	x1, 0x1f
    cpu.execute_rvc_instr(0x_60fd);
    assert_eq!(cpu.regs_r64(1), 0x1f_u64 * 4096_u64);
    assert_eq!(cpu.get_pc(), 4);
}

// c.addi16sp sp, -144
#[test]
fn test_rvc_instr_c_addi16sp() {
    let mut cpu = RV64ICpu::default();
    // c.addi16sp x2, -144
    cpu.execute_rvc_instr(0x_7175);
    // -144 + 144 == 0
    assert_eq!(cpu.regs_r64(2).wrapping_add(144), 0);
    assert_eq!(cpu.get_pc(), 2);
}

// c.sdsp x8, 128(x2)
#[test]
fn test_rv_instr_c_sdsp() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);
    // SP points at 0
    assert_eq!(cpu.regs_r64(2), 0x0000_0000_0000_0000);
    cpu.regs_w64(8, 0xdead_c0de_dead_c0de);
    // c.sdsp x8, 128(x2)
    cpu.execute_rvc_instr(0x_e122);
    assert_eq!(cpu.bus.read64(128), 0xdead_c0de_dead_c0de);
    assert_eq!(cpu.get_pc(), 2);
}

// c.ldsp rd, offset(x2)
#[test]
fn test_rvc_instr_ldsp() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);
    cpu.bus.write64(8, 0x_dead_beef_dead_beef);
    // SP/x2 points at 0
    // c.ldsp x8, 8(x2)
    cpu.execute_rvc_instr(0x_6422);
    assert_eq!(cpu.regs_r64(8), 0x_dead_beef_dead_beef);
    assert_eq!(cpu.get_pc(), 2);
}

// c.addi4spn
#[test]
fn test_rvc_instr_c_addi4spn() {
    let mut cpu = RV64ICpu::default();
    // c.addi4spn x8,x2,144
    cpu.execute_rvc_instr(0x_0900);
    assert_eq!(cpu.regs_r64(8), 144);
    assert_eq!(cpu.get_pc(), 2);
}

// c.mv rd, rs
#[test]
fn test_rvc_instr_c_mv() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(11, 0x0123_4567_89ab_cdef);
    assert_eq!(cpu.regs_r64(18), 0x0000_0000_0000_0000);
    // c.mv x18, x11
    cpu.execute_rvc_instr(0x_892e);
    assert_eq!(cpu.regs_r64(18), 0x0123_4567_89ab_cdef);
    assert_eq!(cpu.get_pc(), 2);
}

// c.addiw rd, uimm6
#[test]
fn test_rvc_instr_c_addiw() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(15, 0x_1234_abcd_0000_0000);
    // c.addiw x15, 0
    cpu.execute_rvc_instr(0x_2781);
    assert_eq!(cpu.regs_r64(15), 0x_0000_0000_0000_0000);
    assert_eq!(cpu.get_pc(), 2);
}

// c.or rd, rs2
#[test]
fn test_rvc_instr_c_or() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(15, 0x_5555_5555_5555_5555);
    cpu.regs_w64(14, 0x_aaaa_aaaa_aaaa_aaaa);
    // c.or x15, x14
    cpu.execute_rvc_instr(0x_8fd9);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_ffff_ffff);
    assert_eq!(cpu.get_pc(), 2);
}

#[test]
/// Check all non-jumping RVC instructions increment PC by 2
fn test_all_rvc_instr_incr_pc_2() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);
    // c.or x15, x14
    cpu.execute_rvc_instr(0x_8fd9);
    // c.addiw x15, 0
    cpu.execute_rvc_instr(0x_2781);
    // c.bnez x15, 0x3706
    cpu.execute_rvc_instr(0x_fbfd);
    // make next c.beqz not to branch
    cpu.regs_w64(15, 1);
    // c.beqz x15, 0x3718
    cpu.execute_rvc_instr(0x_cf81);
    // c.sdsp x8, 128(x2)
    cpu.execute_rvc_instr(0x_e122);
    // c.ldsp x8, 8(x2)
    cpu.execute_rvc_instr(0x_6422);
    // c.addi4spn	x8,x2,144
    cpu.execute_rvc_instr(0x_0900);
    // c.li x1, 1
    cpu.execute_rvc_instr(0x_4085);
    // c.add x1, x1
    cpu.execute_rvc_instr(0x_9086);
    // c.addi a0,1
    cpu.execute_rvc_instr(0x_0505);
    // c.slli x6, 0x1f
    cpu.execute_rvc_instr(0x_037e);
    // c.lui x1, 0x1f
    cpu.execute_rvc_instr(0x_60fd);
    // c.addi16sp x2, -144
    cpu.execute_rvc_instr(0x_7175);
    // c.mv x18, x11
    cpu.execute_rvc_instr(0x_892e);
    // todo
    assert_eq!(cpu.get_pc(), 28);
    // TODO: add all instructions
}
