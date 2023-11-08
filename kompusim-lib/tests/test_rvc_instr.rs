use kompusim::rv64i_cpu::RV64ICpu;

#[test]
// c.li x1, 1
fn test_16b_instr_c_li() {
    let mut cpu = RV64ICpu::default();
    assert!(cpu.regs.x[1] == 0);
    cpu.execute_rvc_instr(0x_4085);
    assert!(cpu.regs.x[1] == 1);
}

#[test]
// c.jr x1
fn test_16b_instr_c_jr() {
    let mut cpu = RV64ICpu::default();
    cpu.regs_w64(1, 0x54);
    cpu.execute_16b_instr(0x_8082);
    assert!(cpu.regs.pc == 0x54);
}
