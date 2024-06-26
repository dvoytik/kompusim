use crate::{
    alu::Imm,
    rvc_dec::{rv64c_decode_instr, COpcode},
};

pub fn disasm_rvc_operation_name(instr: u16) -> String {
    match rv64c_decode_instr(instr) {
        COpcode::CNOP => "Compressed No Operation".to_string(),
        COpcode::CADDI { .. } => "Compressed Immediate Add".to_string(),
        COpcode::CLUI { .. } => "Compressed Load Upper Immediate".to_string(),
        COpcode::ADDI16SP { .. } => "Add Immediate to Stack Pointer (x2)".to_string(),
        COpcode::CSLLI { .. } => "Compressed Shift Left Logical Immediate".to_string(),
        COpcode::CSRLI { .. } => "Compressed Shift Right Logical Immediate".to_string(),
        COpcode::CLI { .. } => "Compressed Load Immediate".to_string(),
        COpcode::CJR { .. } => "Compressed Jump Register".to_string(),
        COpcode::CADD { .. } => "Compressed Add".to_string(),
        COpcode::CADDW { .. } => "Compressed Add Word".to_string(),
        COpcode::CSUBW { .. } => "Compressed Subtract Word".to_string(),
        COpcode::COR { .. } => "Compressed bitwise Or".to_string(),
        COpcode::CAND { .. } => "Compressed bitwise And".to_string(),
        COpcode::CANDI { .. } => "Compressed bitwise AND Immediate".to_string(),
        COpcode::CJ { .. } => "Compressed Jump".to_string(),
        COpcode::BEQZ { .. } => "Compressed Branch Equal Zero".to_string(),
        COpcode::BNEZ { .. } => "Compressed Branch Not Equal Zero".to_string(),
        COpcode::SDSP { .. } => "Compressed Store Doubleword at Stack Pointer".to_string(),
        COpcode::LDSP { .. } => "Compressed Load Doubleword at Stack Pointer".to_string(),
        COpcode::LD { .. } => "Compressed Load Doubleword from memory".to_string(),
        COpcode::SW { .. } => "Compressed Store Word to memory".to_string(),
        COpcode::SD { .. } => "Compressed Store Double-word to memory".to_string(),
        COpcode::LW { .. } => "Compressed Load Word to memory".to_string(),
        COpcode::ADDIW { .. } => "Compressed Add Immediate Word".to_string(),
        COpcode::ADDI4SPN { .. } => {
            "Compressed Add Immediate * 4 to Stack Pointer (x2)".to_string()
        }
        COpcode::MV { .. } => "Compressed Move".to_string(),

        COpcode::Hint => "HINT (NOP)".to_string(),
        COpcode::Uknown => "Unknown RVC instruction".to_string(),
        COpcode::Reserved => "Reserved RVC instruction".to_string(),
    }
}

pub fn disasm_rvc_pseudo_code(instr: u16) -> String {
    match rv64c_decode_instr(instr) {
        COpcode::CNOP => "".to_string(),
        COpcode::CADDI { imm6, rd } => format!("x{rd} = x{rd} + 0x{imm6:x}"),
        COpcode::CLUI { rd, imm6 } => format!("x{rd} = 0x{imm6:x} << 12"),
        COpcode::ADDI16SP { imm6 } => format!("x2 = x2 {:+}", imm6.0 << 4),
        COpcode::CSLLI { uimm6, rd } => format!("x{rd} = x{rd} << {uimm6}"),
        COpcode::CSRLI { shamt6, rd } => format!("x{rd} = x{rd} >> {shamt6}"),
        COpcode::CLI { imm6, rd } => format!("x{rd} = {imm6:x}"),
        COpcode::CJR { rs1 } => format!("PC = x{rs1}"),
        COpcode::CADD { rd, rs2 } => format!("x{rd} = x{rd} + x{rs2}"),
        COpcode::CADDW { rd, rs2 } => {
            format!("x{rd}[31:0] = x{rd}[31:0] + x{rs2}[31:0]; sign extend")
        }
        COpcode::CSUBW { rd, rs2 } => {
            format!("x{rd}[31:0] = x{rd}[31:0] - x{rs2}[31:0]; sign extend")
        }
        COpcode::COR { rd, rs2 } => format!("x{rd} = x{rd} | x{rs2}"),
        COpcode::CAND { rd, rs2 } => format!("x{rd} = x{rd} & x{rs2}"),
        COpcode::CANDI { imm6, rd } => format!("x{rd} = x{rd} & 0x{imm6:x}"),
        COpcode::CJ { imm12 } => format!("PC = PC + {:x}", imm12),
        COpcode::BEQZ { imm9, rs1 } => format!("if x{rs1} == 0 then PC = PC {:+}", imm9.0),
        COpcode::BNEZ { imm9, rs1 } => format!("if x{rs1} != 0 then PC = PC {:+}", imm9.0),
        COpcode::SDSP { uimm6, rs2 } => format!("mem64[x2 {:+}] = x{rs2}", uimm6 << 3),
        COpcode::LDSP { uimm6, rd } => format!("x{rd} = mem64[x2 {:+}]", uimm6 << 3),
        COpcode::LD { uoff8, rs1, rd } => format!("x{rd} = mem64[x{rs1} + {uoff8}]"),
        COpcode::SW { uoff7, rs1, rs2 } => format!("mem32[x{rs1} + {uoff7}] = x{rs2}"),
        COpcode::SD { uoff8, rs1, rs2 } => format!("mem64[x{rs1} + {uoff8}] = x{rs2}"),
        COpcode::LW { uoff7, rs1, rd } => {
            format!("x{rd}[31:0] = mem32[x{rs1} + {uoff7}]; sign extend")
        }
        COpcode::ADDIW { rd, uimm6 } => format!("x{rd} = x{rd} + {uimm6}"),
        COpcode::ADDI4SPN { uimm8, rd } => format!("x{rd} = x2 + {uimm8} * 4"),
        COpcode::MV { rd, rs2 } => format!("x{rd} = x{rs2}"),

        COpcode::Hint => "HINT (NOP)".to_string(),
        COpcode::Uknown => "Unknown RVC instruction".to_string(),
        COpcode::Reserved => "Reserved RVC instruction".to_string(),
    }
}

/// Returns used registers indexes of a 16 compressed instruction (rs1, rs2, rd)
pub fn disasm_rvc_get_used_regs(instr: u16) -> (Option<u8>, Option<u8>, Option<u8>) {
    match rv64c_decode_instr(instr) {
        COpcode::CNOP => (None, None, None),
        COpcode::CADDI { rd, .. } => (Some(rd), None, Some(rd)),
        COpcode::CLUI { rd, .. } => (None, None, Some(rd)),
        COpcode::ADDI16SP { .. } => (Some(2), None, Some(2)),
        COpcode::CSLLI { rd, .. } => (Some(rd), None, Some(rd)),
        COpcode::CSRLI { rd, .. } => (Some(rd), None, Some(rd)),
        COpcode::CLI { rd, .. } => (None, None, Some(rd)),
        COpcode::CJR { rs1 } => (Some(rs1), None, None),
        COpcode::CADD { rd, rs2 } => (Some(rd), Some(rs2), Some(rd)),
        COpcode::CADDW { rd, rs2 } => (Some(rd), Some(rs2), Some(rd)),
        COpcode::CSUBW { rd, rs2 } => (Some(rd), Some(rs2), Some(rd)),
        COpcode::COR { rd, rs2 } => (Some(rd), Some(rs2), Some(rd)),
        COpcode::CAND { rd, rs2 } => (Some(rd), Some(rs2), Some(rd)),
        COpcode::CANDI { rd, .. } => (Some(rd), None, Some(rd)),
        COpcode::CJ { .. } => (None, None, None),
        COpcode::BEQZ { rs1, .. } => (Some(rs1), None, None),
        COpcode::BNEZ { rs1, .. } => (Some(rs1), None, None),
        COpcode::SDSP { rs2, .. } => (Some(2), Some(rs2), None),
        COpcode::LDSP { rd, .. } => (Some(2), None, Some(rd)),
        COpcode::LD { rs1, rd, .. } => (Some(rs1), None, Some(rd)),
        COpcode::SW { rs1, rs2, .. } => (Some(rs1), Some(rs2), None),
        COpcode::SD { rs1, rs2, .. } => (Some(rs1), Some(rs2), None),
        COpcode::LW { rs1, rd, .. } => (Some(rs1), None, Some(rd)),
        COpcode::ADDIW { rd, .. } => (Some(rd), None, Some(rd)),
        COpcode::ADDI4SPN { rd, .. } => (Some(2), None, Some(rd)),
        COpcode::MV { rd, rs2 } => (None, Some(rs2), Some(rd)),

        COpcode::Hint => (None, None, None),
        COpcode::Uknown => (None, None, None),
        COpcode::Reserved => (None, None, None),
    }
}

pub fn disasm_rvc(c_instr: u16, instr_addr: u64) -> String {
    match rv64c_decode_instr(c_instr) {
        COpcode::CNOP => "nop".to_string(),
        COpcode::CADDI { imm6, rd } => format!("c.addi x{rd}, {imm6}"),
        COpcode::CLUI { rd, imm6 } => format!("c.lui x{rd}, 0x{imm6:x}"),
        COpcode::ADDI16SP { imm6 } => format!("c.addi16sp x2, {:+}", (imm6.0 as i16) << 4),
        COpcode::CSLLI { uimm6, rd } => format!("c.slli x{rd}, 0x{uimm6:x}"),
        COpcode::CSRLI { shamt6, rd } => format!("c.srli x{rd}, 0x{shamt6:x}"),
        COpcode::CLI { imm6, rd } => format!("c.li x{rd}, {imm6}"),
        COpcode::CJR { rs1 } => format!("c.jr x{rs1}"),
        COpcode::CADD { rd, rs2 } => format!("c.add x{rd}, x{rs2}"),
        COpcode::CADDW { rd, rs2 } => format!("c.addw x{rd}, x{rs2}"),
        COpcode::CSUBW { rd, rs2 } => format!("c.subw x{rd}, x{rs2}"),
        COpcode::COR { rd, rs2 } => format!("c.or x{rd}, x{rs2}"),
        COpcode::CAND { rd, rs2 } => format!("c.and x{rd}, x{rs2}"),
        COpcode::CANDI { imm6, rd } => format!("c.andi x{rd}, {imm6}"),
        COpcode::CJ { imm12 } => format!("c.j {:x}", instr_addr.add_i12(imm12)),
        COpcode::BEQZ { imm9, rs1 } => format!("c.beqz x{rs1}, 0x{:x}", instr_addr.add_i9(imm9)),
        COpcode::BNEZ { imm9, rs1 } => format!("c.bnez x{rs1}, 0x{:x}", instr_addr.add_i9(imm9)),
        COpcode::SDSP { uimm6, rs2 } => format!("c.sdsp x{rs2}, {}(x2)", uimm6 << 3),
        COpcode::LDSP { uimm6, rd } => format!("c.ldsp x{rd}, {}(x2)", uimm6 << 3),
        COpcode::LD { uoff8, rs1, rd } => format!("c.ld x{rd}, {uoff8}(x{rs1})"),
        COpcode::SW { uoff7, rs1, rs2 } => format!("c.sw x{rs2}, {uoff7}(x{rs1})"),
        COpcode::SD { uoff8, rs1, rs2 } => format!("c.sd x{rs2}, {uoff8}(x{rs1})"),
        COpcode::LW { uoff7, rs1, rd } => format!("c.lw x{rd}, {uoff7}(x{rs1})"),
        COpcode::ADDIW { uimm6, rd } => format!("c.addiw x{rd}, {uimm6}"),
        COpcode::ADDI4SPN { uimm8, rd } => format!("c.addi4spn x{rd}, x2, {}", uimm8 << 2),
        COpcode::MV { rd, rs2 } => format!("c.mv x{rd}, x{rs2}"),

        COpcode::Hint => "HINT (NOP)".to_string(),
        COpcode::Uknown => "Unknown RVC instruction".to_string(),
        COpcode::Reserved => "Reserved RVC instruction".to_string(),
    }
}

#[test]
fn test_disasm_rvc_cli() {
    assert_eq!(disasm_rvc(0x_4085, 0x0), "c.li x1, 1".to_owned());
    assert_eq!(disasm_rvc(0x_517d, 0x0), "c.li x2, -1".to_string());
    assert_eq!(disasm_rvc(0x_8082, 0x0), "c.jr x1".to_string());
    assert_eq!(disasm_rvc(0x_9086, 0x0), "c.add x1, x1".to_string());
    assert_eq!(disasm_rvc(0x_9fb9, 0x0), "c.addw x15, x14".to_string());
    assert_eq!(disasm_rvc(0x_9f99, 0x0), "c.subw x15, x14".to_string());
    assert_eq!(disasm_rvc(0x_037e, 0x0), "c.slli x6, 0x1f".to_string());
    assert_eq!(disasm_rvc(0x_9301, 0x0), "c.srli x14, 0x20".to_string());
    assert_eq!(disasm_rvc(0x_a001, 0x0), "c.j 0".to_string());
    assert_eq!(disasm_rvc(0x_b7ed, 0x8000003a), "c.j 80000024".to_string());
    assert_eq!(disasm_rvc(0x_0505, 0x0), "c.addi x10, 1".to_string());
    assert_eq!(disasm_rvc(0x_60fd, 0x0), "c.lui x1, 0x1f");
    assert_eq!(disasm_rvc(0x_7175, 0x0), "c.addi16sp x2, -144");
    assert_eq!(disasm_rvc(0x_e122, 0x0), "c.sdsp x8, 128(x2)");
    assert_eq!(disasm_rvc(0x_6422, 0x0), "c.ldsp x8, 8(x2)");
    assert_eq!(disasm_rvc(0x_0900, 0x0), "c.addi4spn x8, x2, 144");
    assert_eq!(disasm_rvc(0x_892e, 0x0), "c.mv x18, x11");
    assert_eq!(disasm_rvc(0x_cf81, 0x_8000_3700), "c.beqz x15, 0x80003718");
    assert_eq!(disasm_rvc(0x_fbfd, 0x_8000_3710), "c.bnez x15, 0x80003706");
    assert_eq!(disasm_rvc(0x_2781, 0x0), "c.addiw x15, 0");
    assert_eq!(disasm_rvc(0x_8fd9, 0x0), "c.or x15, x14");
    assert_eq!(disasm_rvc(0x_8ff5, 0x0), "c.and x15, x13");
    assert_eq!(disasm_rvc(0x_8b9d, 0x0), "c.andi x15, 7");
    assert_eq!(disasm_rvc(0x_7d3c, 0x0), "c.ld x15, 120(x10)");
    assert_eq!(disasm_rvc(0x_c7d8, 0x0), "c.sw x14, 12(x15)");
    assert_eq!(disasm_rvc(0x_fff8, 0x0), "c.sd x14, 248(x15)");
    assert_eq!(disasm_rvc(0x_4ffc, 0x0), "c.lw x15, 92(x15)");
}
