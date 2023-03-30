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
    pub end:   u64, // end physical address
    pub m:     Vec<u8>,
}

impl Ram {
    pub fn new(start: u64, size: u64) -> Ram {
        Ram { start,
              end: start + size as u64,
              m: vec![0; size as usize] } // TODO: is it lazy allocation?
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
        let offset = addr - self.start;
        let f_size = fs::metadata(fname)?.len();
        if offset + f_size > self.m.len() as u64 {
            return Err(Box::new(RamError { details: "size is wrong".to_string(), }));
        }
        let mut f = File::open(fname)?;
        f.read(&mut self.m[offset as usize..])?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn dump_hex(&self, addr: u64, size: u64) {
        let aligned_addr = addr & !0xf_u64;
        let aligned_size = (size + 16) & !0xf_u64;
        let mut line = String::with_capacity(size as usize + 32);
        // TODO: optimize - slow
        let mut pr_str = String::with_capacity(22);
        line.push_str(&format!("{:016x} ", aligned_addr));
        for (i, b) in self.m[..aligned_size as usize].iter().enumerate() {
            let i = i as u64;
            if i == size {
                if i % 16 != 0 {
                    let mid_blank = if i % 16 < 8 { 1 } else { 0 };
                    let left_blanks = mid_blank + 3 * (16 - (i % 16));
                    line.push_str(&format!("{:1$}", " ", left_blanks as usize));
                }
                line.push_str(&format!("| {} |\n", pr_str));
                line.push_str(&format!("{:016x} ", aligned_addr + i + 16));
                break;
            }
            if i > 0 && i % 16 == 0 {
                line.push_str(&format!("| {} |\n", pr_str));
                line.push_str(&format!("{:016x} ", aligned_addr + i + 16));
                pr_str.clear();
            }
            if i % 8 == 0 {
                line.push_str(" ");
            }
            line.push_str(&format!("{:02x} ", b));
            pr_str.push(if *b >= 0x20 && *b <= 0x7e {
                            *b as char
                        } else {
                            '.'
                        })
        }
        println!("{}", line);
    }
}
