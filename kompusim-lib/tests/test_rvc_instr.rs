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
/// Check all non-jumping RVC instructions increment PC by 2
fn test_all_rvc_instr_incr_pc_2() {
    let mut cpu = RV64ICpu::default();
    // c.jr x1
    cpu.execute_rvc_instr(0x_8082);
    // c.li x1, 1
    cpu.execute_rvc_instr(0x_4085);
    // c.add x1, x1
    cpu.execute_rvc_instr(0x_9086);
    assert!(cpu.regs.pc == 0x_04);
    // TODO: add all instructions
}
