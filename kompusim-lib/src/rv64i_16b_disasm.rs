use crate::rv64i_16b_dec::{decode_16b_instr, COpcode};

pub fn disasm_16b(c_instr: u16, _instr_addr: u64) -> String {
    match decode_16b_instr(c_instr) {
        COpcode::CLI { imm6, rd } => format!("c.li x{rd}, {imm6}"),

        COpcode::Uknown => "Unknown 16b instruction".to_string(),
    }
}

#[test]
fn test_disasm_16b_cli() {
    assert_eq!(disasm_16b(0x_4085, 0x0), "c.li x1, 1".to_owned());
    assert_eq!(disasm_16b(0x_517d, 0x0), "c.li x2, -1".to_owned())
}
