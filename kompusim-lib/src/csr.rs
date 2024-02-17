// trick with mod and use to disable rustfmt for the following defines
#[rustfmt::skip]
mod csr_defines {
pub const MTVEC: u16    = 0x305; // Machine trap-handler base address.
pub const MSCRATCH: u16 = 0x340; // Machine Scratch register for machine trap handlers.
pub const MHARTID:u16   = 0xf14; // Machine Hardware Thread ID
}
pub use csr_defines::*;

#[derive(Default)]
pub struct Csrs {
    /// Machine trap-handler base address.
    mtvec: u64,
    /// Machine Scratch register for machine trap handlers.
    mscratch: u64,
}

impl Csrs {
    pub fn new() -> Csrs {
        Csrs {
            mscratch: 0,
            mtvec: 0,
        }
    }
    /// Read 64 bit
    pub fn r64(&self, csr_a: u16) -> u64 {
        match csr_a {
            MTVEC => self.mtvec,
            MHARTID => 0, // current cpu id
            MSCRATCH => self.mscratch,
            _ => panic!("CSR: 0x{csr_a:x} is not implemented"),
        }
    }
    /// Write 64 bit
    pub fn w64(&mut self, csr_a: u16, val: u64) {
        match csr_a {
            MTVEC => self.mtvec = val,
            MHARTID => (), // ignore
            MSCRATCH => self.mscratch = val,
            _ => panic!("CSR: 0x{csr_a:x} is not implemented"),
        }
    }
}
