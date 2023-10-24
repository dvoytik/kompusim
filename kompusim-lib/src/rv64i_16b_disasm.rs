use crate::rv64i_16b_dec::{decode_16b_instr, COpcode};

pub fn disasm_16b(c_instr: u16, instr_addr: u64) -> String {
    match decode_16b_instr(c_instr) {
        COpcode::CLI { imm6, rd } => format!("c.li x{rd}, 0x{:x}", imm6),

        COpcode::Uknown => "Unknown 16b instruction".to_string(),
    }
}
