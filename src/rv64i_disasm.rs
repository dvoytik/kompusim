// Disassembler

use crate::rv64i_dec::*;

pub fn disasm(instr: u32) -> String {
    match i_opcode(instr) {
        OPC_SYSTEM => {
            let Instr::System {
                csr,
                rs1,
                funct3,
                rd,
            } = dec_opc_system(instr);

            match funct3 {
                F3_SYSTEM_CSRRS => {
                    // TODO: conver csr to string name, e.g. "mhartid"
                    format!("csrrs x{rd}, {csr:x}, 0x{rs1:x}")
                }
                _ => "uknown SYSTEM instruction".to_string(),
            }
        }
        _ => "todo".to_string(),
    }
}
