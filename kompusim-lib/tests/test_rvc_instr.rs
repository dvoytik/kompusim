use kompusim::rv64i_cpu::RV64ICpu;

#[test]
// c.li x1, 1
fn test_rvc_instr_c_li() {
    let mut cpu = RV64ICpu::default();
    assert!(cpu.regs.x[1] == 0);
    cpu.execute_rvc_instr(0x_4085);
    assert!(cpu.regs.x[1] == 1);
}

#[test]
// c.jr x1
fn test_rvc_instr_c_jr() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(1, 0x54);
    cpu.execute_rvc_instr(0x_8082);
    assert!(cpu.regs.pc == 0x54);
}

#[test]
// c.add x1, x1
fn test_rvc_add() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(1, 0x1122_3344);
    cpu.execute_rvc_instr(0x_9086);
    assert!(cpu.regs.x[1] == 0x2244_6688);
}

#[test]
// 8000003a:  b7ed  c.j 80000024
fn test_rvc_instr_c_j() {
    let mut cpu = RV64ICpu::default();
    cpu.regs.pc = 0x_8000_003a;
    cpu.execute_rvc_instr(0x_b7ed);
    assert!(cpu.regs.pc == 0x_8000_0024);
}

// c.addi a0,1
#[test]
fn test_rvc_instr_c_addi() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(10, 0x1122_3344);
    cpu.execute_rvc_instr(0x_0505);
    assert!(cpu.regs.x[10] == 0x1122_3345);
}

// c.slli x6, 0x1f
#[test]
fn test_rvc_instr_slli() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(6, 0x1);
    cpu.execute_rvc_instr(0x_037e);
    assert!(cpu.regs.x[6] == 0x1 << 0x1f);
}

#[test]
fn test_todo() {
    let mut cpu = RV64ICpu::default();
    // c.lui	x21, 0x2
    cpu.execute_rvc_instr(0x_6a89);
    assert!(cpu.regs_r64(21) == 2 * 4096);
    // c.lui	x1, 0x1f
    cpu.execute_rvc_instr(0x_60fd);
    assert_eq!(cpu.regs_r64(1), 0x1f_u64 * 4096_u64);
}

#[test]
/// Check all non-jumping RVC instructions increment PC by 2
fn test_all_rvc_instr_incr_pc_2() {
    let mut cpu = RV64ICpu::default();
    // c.jr x1
    cpu.execute_rvc_instr(0x_8082);
    // c.li x1, 1
    cpu.execute_rvc_instr(0x_4085);
    // c.add x1, x1
    cpu.execute_rvc_instr(0x_9086);
    // c.addi a0,1
    cpu.execute_rvc_instr(0x_0505);
    assert!(cpu.regs.pc == 0x_06);
    // TODO: add all instructions
}
