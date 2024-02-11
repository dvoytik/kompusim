// trick with mod and use to disable rustfmt for the following defines
#[rustfmt::skip]
mod csr_defines {
pub const MHARTID:  u16 = 0xf14; // Machine Hardware Thread ID
pub const MSCRATCH: u16 = 0x340; // Machine Scratch register for machine trap handlers.
}
use csr_defines::*;

#[derive(Default)]
pub struct Csrs {
    mscratch: u64,
}

impl Csrs {
    pub fn new() -> Csrs {
        Csrs { mscratch: 0 }
    }
    /// Read 64 bit
    pub fn r64(&self, csr_a: u16) -> u64 {
        match csr_a {
            MHARTID => 0, // current cpu id
            MSCRATCH => self.mscratch,
            _ => panic!("not implemented"),
        }
    }
    /// Write 64 bit
    pub fn w64(&mut self, csr_a: u16, val: u64) {
        match csr_a {
            MHARTID => (), // ignore
            MSCRATCH => self.mscratch = val,
            _ => panic!("not implemented"),
        }
    }
}
