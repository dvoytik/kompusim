use kompusim::rv64i_cpu::RV64ICpu;

#[test]
// c.li x1, 1
fn test_16b_instr_c_li() {
    let mut cpu = RV64ICpu::default();
    assert!(cpu.regs.x[1] == 0);
    cpu.execute_16b_instr(0x_4085);
    assert!(cpu.regs.x[1] == 1);
}
