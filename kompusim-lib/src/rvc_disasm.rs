use crate::rvc_dec::{decode_rvc_instr, COpcode};

pub fn disasm_rvc_operation_name(instr: u16) -> String {
    match decode_rvc_instr(instr) {
        COpcode::CLI { .. } => "Compressed Load Immediate".to_string(),
        COpcode::CJR { .. } => "Compressed Jump Register".to_string(),
        COpcode::CADD { .. } => "Compressed Add".to_string(),

        COpcode::Uknown => "Unknown RVC instruction".to_string(),
    }
}

pub fn disasm_rvc_pseudo_code(instr: u16) -> String {
    match decode_rvc_instr(instr) {
        COpcode::CLI { imm6, rd } => format!("x{rd} = {}", imm6),
        COpcode::CJR { rs1 } => format!("PC = x{rs1}"),
        COpcode::CADD { rd, rs2 } => format!("x{rd} = x{rd} + x{rs2}"),

        COpcode::Uknown => "Unknown RVC instruction".to_string(),
    }
}

/// Returns used registers indexes of a 16 compressed instruction (rs1, rs2, rd)
pub fn disasm_rvc_get_used_regs(instr: u16) -> (Option<u8>, Option<u8>, Option<u8>) {
    match decode_rvc_instr(instr) {
        COpcode::CLI { rd, .. } => (None, None, Some(rd)),
        COpcode::CJR { rs1 } => (Some(rs1), None, None),
        COpcode::CADD { rd, rs2 } => (Some(rd), Some(rs2), Some(rd)),

        COpcode::Uknown => (None, None, None),
    }
}

pub fn disasm_rvc(c_instr: u16, _instr_addr: u64) -> String {
    match decode_rvc_instr(c_instr) {
        COpcode::CLI { imm6, rd } => format!("c.li x{rd}, {imm6}"),
        COpcode::CJR { rs1 } => format!("c.jr x{rs1}"),
        COpcode::CADD { rd, rs2 } => format!("c.add x{rd}, x{rs2}"),

        COpcode::Uknown => "Unknown RVC instruction".to_string(),
    }
}

#[test]
fn test_disasm_rvc_cli() {
    assert_eq!(disasm_rvc(0x_4085, 0x0), "c.li x1, 1".to_owned());
    assert_eq!(disasm_rvc(0x_517d, 0x0), "c.li x2, -1".to_string());
    assert_eq!(disasm_rvc(0x_8082, 0x0), "c.jr x1".to_string());
}
