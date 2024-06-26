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

// c.addi rd, imm6
#[test]
fn test_rvc_instr_c_addi() {
    let mut cpu = RV64ICpu::default();
    // x10 = -1
    cpu.regs_w64(10, 0x_ffff_ffff_ffff_ffff);

    // c.addi x10, 1
    cpu.execute_rvc_instr(0x_0505);
    assert_eq!(cpu.regs_r64(10), 0x_0000_0000_0000_0000);

    // c.addi x10, -1
    cpu.execute_rvc_instr(0x_157d);
    assert_eq!(cpu.regs_r64(10), 0x_ffff_ffff_ffff_ffff);

    // c.addi a0, 31
    cpu.execute_rvc_instr(0x_057d);
    assert_eq!(cpu.regs_r64(10), 30);

    // c.addi a0, -31
    cpu.execute_rvc_instr(0x_1505);
    assert_eq!(cpu.regs_r64(10), 0x_ffff_ffff_ffff_ffff);

    assert_eq!(cpu.get_pc(), 4 * 2);
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

// c.andi rd, imm6
#[test]
fn test_rvc_instr_c_andi() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(15, 0x_5555_5555_5555_5555);
    // c.and x15, 7
    cpu.execute_rvc_instr(0x_8b9d);
    assert_eq!(cpu.regs_r64(15), 5);
    assert_eq!(cpu.get_pc(), 2);
}

// c.ld rd, uoff8(rs1)
#[test]
fn test_rvc_instr_c_ld() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);

    cpu.bus.write64(0, 0x_dead_beef_baad_c0fe);
    // c.ld x15, 0(x15)
    cpu.execute_rvc_instr(0x_639c);
    assert_eq!(cpu.regs_r64(15), 0x_dead_beef_baad_c0fe);

    cpu.bus.write64(256 + 120, 0x_dead_c0de_dead_c0de);
    cpu.regs_w64(10, 256);
    // c.ld x15, 120(x10)
    cpu.execute_rvc_instr(0x_7d3c);
    assert_eq!(cpu.regs_r64(15), 0x_dead_c0de_dead_c0de);

    assert_eq!(cpu.get_pc(), 4);
}

// Store Word to memory
// c.sw rs2, 0(rs1)
#[test]
fn test_rvc_instr_c_sw() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);

    cpu.regs_w64(14, 0x_dead_beef_baad_c0fe);
    // c.sw x14, 0(x15)
    cpu.execute_rvc_instr(0x_c398);
    assert_eq!(cpu.bus.read64(0), 0x_0000_0000_baad_c0fe);

    cpu.regs_w64(14, 0x_dead_beef_baad_c0fe);
    cpu.regs_w64(15, 256);
    // c.sw x14, 12(x15)
    cpu.execute_rvc_instr(0x_c7d8);
    assert_eq!(cpu.bus.read64(12 + 256), 0x_0000_0000_baad_c0fe);

    assert_eq!(cpu.get_pc(), 4);
}

// Load Word from memory to rd
// c.lw    rd, offset7(rs1)
#[test]
fn test_rvc_lw() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);

    cpu.bus.write32(0, 0x_dead_beef);
    // c.lw x15, 0(x15)
    cpu.execute_rvc_instr(0x_439c);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_dead_beef);

    cpu.regs_w64(15, 0);
    cpu.bus.write32(92, 0x_c0de_c001);
    // c.lw x15, 92(x15)
    cpu.execute_rvc_instr(0x_4ffc);
    assert_eq!(cpu.regs_r64(15), 0x_ffff_ffff_c0de_c001);

    assert_eq!(cpu.get_pc(), 4);
}

// Bitwise And
// c.and rd, rs2
#[test]
fn test_rvc_and() {
    let mut cpu = RV64ICpu::default();

    cpu.regs_w64(13, 0x_5a5a_a5a5_5a5a_a5a5);
    cpu.regs_w64(15, 0x_ffff_0000_ffff_0000);
    // c.and x15, x13
    cpu.execute_rvc_instr(0x_8ff5);
    assert_eq!(cpu.regs_r64(15), 0x_5a5a_0000_5a5a_0000);

    assert_eq!(cpu.get_pc(), 2);
}

// Add Word (32-bit) with sign extension
// c.addw rd, rs2
#[test]
fn test_rvc_addw() {
    let mut cpu = RV64ICpu::default();

    fn test_addw(cpu: &mut RV64ICpu, res: u64, r1: u64, r2: u64) {
        cpu.regs_w64(15, r1);
        cpu.regs_w64(14, r2);
        // c.addw x15, x14
        cpu.execute_rvc_instr(0x_9fb9);
        assert_eq!(cpu.regs_r64(15), res);
    }
    test_addw(&mut cpu, 2, 1, 1);

    test_addw(
        &mut cpu,
        0xffffffffffff8000,
        0x0000000000000000,
        0xffffffffffff8000,
    );
    test_addw(&mut cpu, 0xffffffff80000000, 0xffffffff80000000, 0x00000000);
    test_addw(
        &mut cpu,
        0x000000007fff8000,
        0xffffffff80000000,
        0xffffffffffff8000,
    );
    test_addw(
        &mut cpu,
        0x0000000000007fff,
        0x0000000000000000,
        0x0000000000007fff,
    );
    test_addw(
        &mut cpu,
        0x000000007fffffff,
        0x000000007fffffff,
        0x0000000000000000,
    );
    test_addw(
        &mut cpu,
        0xffffffff80007ffe,
        0x000000007fffffff,
        0x0000000000007fff,
    );

    test_addw(
        &mut cpu,
        0xffffffff80007fff,
        0xffffffff80000000,
        0x0000000000007fff,
    );
    test_addw(
        &mut cpu,
        0x000000007fff7fff,
        0x000000007fffffff,
        0xffffffffffff8000,
    );

    test_addw(
        &mut cpu,
        0xffffffffffffffff,
        0x0000000000000000,
        0xffffffffffffffff,
    );
    test_addw(
        &mut cpu,
        0x0000000000000000,
        0xffffffffffffffff,
        0x0000000000000001,
    );
    test_addw(
        &mut cpu,
        0xfffffffffffffffe,
        0xffffffffffffffff,
        0xffffffffffffffff,
    );
    test_addw(
        &mut cpu,
        0xffffffff80000000,
        0x0000000000000001,
        0x000000007fffffff,
    );
    assert_eq!(cpu.get_pc(), 13 * 2);
}

// Shift Right Logical Immidiate
// c.srli rd, 0x20
#[test]
fn test_rvc_srli() {
    let mut cpu = RV64ICpu::default();

    cpu.regs_w64(14, 0x_1234_5678_9abc_def0);
    // c.srli x14, 0x20
    cpu.execute_rvc_instr(0x_9301);
    assert_eq!(cpu.regs_r64(14), 0x_0000_0000_1234_5678);
    // c.srli x14, 0x20
    cpu.execute_rvc_instr(0x_9301);
    assert_eq!(cpu.regs_r64(14), 0x_0000_0000_0000_0000);
    assert_eq!(cpu.get_pc(), 4);
}

// Subtract Word (32-bit) with sign extension
// c.subw rd, rs2
#[test]
fn test_rvc_subw() {
    let mut cpu = RV64ICpu::default();

    fn test_subw(cpu: &mut RV64ICpu, res: u64, r1: u64, r2: u64) {
        cpu.regs_w64(15, r1);
        cpu.regs_w64(14, r2);
        // c.subw x15, x14
        cpu.execute_rvc_instr(0x_9f99);
        assert_eq!(cpu.regs_r64(15), res);
    }
    test_subw(&mut cpu, 0, 1, 1);

    test_subw(
        &mut cpu,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    );
    test_subw(
        &mut cpu,
        0x0000000000000000,
        0x0000000000000001,
        0x0000000000000001,
    );
    test_subw(
        &mut cpu,
        0xfffffffffffffffc,
        0x0000000000000003,
        0x0000000000000007,
    );
    test_subw(
        &mut cpu,
        0x0000000000008000,
        0x0000000000000000,
        0xffffffffffff8000,
    );
    test_subw(
        &mut cpu,
        0xffffffff80000000,
        0xffffffff80000000,
        0x0000000000000000,
    );
    test_subw(
        &mut cpu,
        0xffffffff80008000,
        0xffffffff80000000,
        0xffffffffffff8000,
    );
    test_subw(
        &mut cpu,
        0xffffffffffff8001,
        0x0000000000000000,
        0x0000000000007fff,
    );
    test_subw(
        &mut cpu,
        0x000000007fffffff,
        0x000000007fffffff,
        0x0000000000000000,
    );
    test_subw(
        &mut cpu,
        0x000000007fff8000,
        0x000000007fffffff,
        0x0000000000007fff,
    );
    test_subw(
        &mut cpu,
        0x000000007fff8001,
        0xffffffff80000000,
        0x0000000000007fff,
    );
    test_subw(
        &mut cpu,
        0xffffffff80007fff,
        0x000000007fffffff,
        0xffffffffffff8000,
    );
    test_subw(
        &mut cpu,
        0x0000000000000001,
        0x0000000000000000,
        0xffffffffffffffff,
    );
    test_subw(
        &mut cpu,
        0xfffffffffffffffe,
        0xffffffffffffffff,
        0x0000000000000001,
    );
    test_subw(
        &mut cpu,
        0x0000000000000000,
        0xffffffffffffffff,
        0xffffffffffffffff,
    );
}

// Store Double-word to memory
// c.sd rs2, uoffset8(rs1)
#[test]
fn test_rvc_instr_c_sd() {
    let bus = Bus::new_with_ram(0x0000_0000_0000_0000, 4 * 1024);
    let mut cpu = RV64ICpu::new(bus);

    cpu.regs_w64(14, 0x_dead_beef_baad_c0fe);
    // c.sd x14, 0(x15)
    cpu.execute_rvc_instr(0x_e398);
    assert_eq!(cpu.bus.read64(0), 0x_dead_beef_baad_c0fe);

    cpu.regs_w64(14, 0x_dead_beef_baad_c0fe);
    cpu.regs_w64(15, 256);
    // c.sd x14, 248(x15)
    cpu.execute_rvc_instr(0x_fff8);
    assert_eq!(cpu.bus.read64(248 + 256), 0x_dead_beef_baad_c0fe);

    assert_eq!(cpu.get_pc(), 4);
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
