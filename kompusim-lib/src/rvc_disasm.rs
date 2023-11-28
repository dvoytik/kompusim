use crate::{
    alu::Imm,
    rvc_dec::{decode_rvc_instr, COpcode},
};

pub fn disasm_rvc_operation_name(instr: u16) -> String {
    match decode_rvc_instr(instr) {
        COpcode::CNOP => "Compressed No Operation".to_string(),
        COpcode::CADDI { .. } => "Comprossed Immediate Add".to_string(),
        COpcode::CLI { .. } => "Compressed Load Immediate".to_string(),
        COpcode::CJR { .. } => "Compressed Jump Register".to_string(),
        COpcode::CADD { .. } => "Compressed Add".to_string(),
        COpcode::CJ { .. } => "Compressed Jump".to_string(),

        COpcode::Uknown => "Unknown RVC instruction".to_string(),
    }
}

pub fn disasm_rvc_pseudo_code(instr: u16) -> String {
    match decode_rvc_instr(instr) {
        COpcode::CNOP => "".to_string(),
        COpcode::CADDI { imm6, rd } => format!("x{rd} = x{rd} + {imm6:x}"),
        COpcode::CLI { imm6, rd } => format!("x{rd} = {imm6:x}"),
        COpcode::CJR { rs1 } => format!("PC = x{rs1}"),
        COpcode::CADD { rd, rs2 } => format!("x{rd} = x{rd} + x{rs2}"),
        COpcode::CJ { imm12 } => format!("PC = PC + {:x}", imm12),

        COpcode::Uknown => "Unknown RVC instruction".to_string(),
    }
}

/// Returns used registers indexes of a 16 compressed instruction (rs1, rs2, rd)
pub fn disasm_rvc_get_used_regs(instr: u16) -> (Option<u8>, Option<u8>, Option<u8>) {
    match decode_rvc_instr(instr) {
        COpcode::CNOP => (None, None, None),
        COpcode::CADDI { rd, .. } => (None, None, Some(rd)),
        COpcode::CLI { rd, .. } => (None, None, Some(rd)),
        COpcode::CJR { rs1 } => (Some(rs1), None, None),
        COpcode::CADD { rd, rs2 } => (Some(rd), Some(rs2), Some(rd)),
        COpcode::CJ { .. } => (None, None, None),

        COpcode::Uknown => (None, None, None),
    }
}

pub fn disasm_rvc(c_instr: u16, instr_addr: u64) -> String {
    match decode_rvc_instr(c_instr) {
        COpcode::CNOP => "nop".to_string(),
        COpcode::CADDI { imm6, rd } => format!("c.addi x{rd}, {imm6}"),
        COpcode::CLI { imm6, rd } => format!("c.li x{rd}, {imm6}"),
        COpcode::CJR { rs1 } => format!("c.jr x{rs1}"),
        COpcode::CADD { rd, rs2 } => format!("c.add x{rd}, x{rs2}"),
        COpcode::CJ { imm12 } => format!("c.j {:x}", instr_addr.add_i12(imm12)),

        COpcode::Uknown => "Unknown RVC instruction".to_string(),
    }
}

#[test]
fn test_disasm_rvc_cli() {
    assert_eq!(disasm_rvc(0x_4085, 0x0), "c.li x1, 1".to_owned());
    assert_eq!(disasm_rvc(0x_517d, 0x0), "c.li x2, -1".to_string());
    assert_eq!(disasm_rvc(0x_8082, 0x0), "c.jr x1".to_string());
    assert_eq!(disasm_rvc(0x_9086, 0x0), "c.add x1, x1".to_string());
    assert_eq!(disasm_rvc(0x_a001, 0x0), "c.j 0".to_string());
    assert_eq!(disasm_rvc(0x_b7ed, 0x8000003a), "c.j 80000024".to_string());
    assert_eq!(disasm_rvc(0x_0505, 0x0), "c.addi x10, 1".to_string());
}
