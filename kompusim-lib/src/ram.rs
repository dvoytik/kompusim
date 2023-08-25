use core::fmt;
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

use crate::bits::BitOps;

#[derive(Debug)]
struct RamError {
    details: String,
}

impl fmt::Display for RamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for RamError {
    fn description(&self) -> &str {
        &self.details
    }
}

// TODO: consider DS Rope, crate "bytes"
#[derive(Default)]
pub struct Ram {
    // TODO: do we need start and end here?
    pub start: u64, // start physical address
    pub end: u64,   // end physical address
    pub m: Vec<u8>,
}

impl Ram {
    pub fn new(start: u64, size: u64) -> Ram {
        Ram {
            start,
            end: start + size as u64,
            m: vec![0; size as usize],
        } // TODO: is it lazy allocation?
    }

    pub fn read8(&self, addr: u64) -> u8 {
        let offs = (addr - self.start) as usize;
        self.m[offs]
    }

    pub fn write8(&mut self, addr: u64, val: u8) {
        let offs = (addr - self.start) as usize;
        self.m[offs] = val
    }

    // Little Endian 32-bit read
    pub fn read32(&self, addr: u64) -> u32 {
        let offs = (addr - self.start) as usize;
        let b0 = self.m[offs] as u32;
        let b1 = self.m[offs + 1] as u32;
        let b2 = self.m[offs + 2] as u32;
        let b3 = self.m[offs + 3] as u32;
        // little endian
        b3 << 24 | b2 << 16 | b1 << 8 | b0
    }

    // Little Endian 32-bit write
    pub fn write32(&mut self, addr: u64, val: u32) {
        let offs = (addr - self.start) as usize;
        self.m[offs] = val.bits(7, 0) as u8;
        self.m[offs + 1] = val.bits(15, 8) as u8;
        self.m[offs + 2] = val.bits(23, 16) as u8;
        self.m[offs + 3] = val.bits(31, 24) as u8;
    }

    pub fn load_bin_file(&mut self, addr: u64, fname: &PathBuf) -> Result<(), Box<dyn Error>> {
        // TODO: check if exists
        assert!(addr >= self.start && addr <= self.end);
        let offset = (addr - self.start) as usize;
        let f_size = fs::metadata(fname)?.len() as usize;
        if offset + f_size > self.m.len() {
            return Err(Box::new(RamError {
                details: "size is wrong".to_string(),
            }));
        }
        let mut f = File::open(fname)?;
        f.read_exact(&mut self.m[offset..offset + f_size])?;
        Ok(())
    }

    pub fn load_image(&mut self, addr: u64, bin: &'static [u8]) -> Result<(), Box<dyn Error>> {
        assert!(addr >= self.start && addr <= self.end);
        let offset = addr - self.start;
        let bin_size = bin.len() as u64;
        if offset + bin_size > self.m.len() as u64 {
            return Err(Box::new(RamError {
                details: "size is wrong".to_string(),
            }));
        }
        for i in 0..bin.len() {
            self.m[offset as usize + i] = bin[i];
        }
        Ok(())
    }

    pub fn get_ram(&self, addr: u64, size: u64) -> Option<&[u8]> {
        if addr < self.start || addr > self.end {
            return None;
        }
        if addr + size > self.end {
            return None;
        }
        let offs = (addr - self.start) as usize;
        Some(&self.m[offs..offs + size as usize])
    }
}
