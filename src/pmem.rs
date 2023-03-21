use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::result::Result;

// TODO: use generics
pub struct MemRegion {
    start: u64,
    end:   u64,
    pub m: Vec<u8>,
}

#[derive(Default)]
pub struct Pmem {
    regions: Vec<MemRegion>, // TODO: should be sorted
}

impl Pmem {
    pub fn alloc_region(&mut self, start_addr: u64, size: u64) {
        self.regions.push(MemRegion { start: start_addr,
                                      end:   start_addr + size,
                                      m:     vec![0; size as usize], })
    }

    pub fn find_mem_region(&self, start: u64, end: u64) -> Option<&MemRegion> {
        for r in &self.regions {
            // TODO: fast binary search
            // TODO: what if it crosses two regions?
            if start >= r.start && end <= r.end {
                return Some(r);
            }
        }
        None
    }

    pub fn find_mem_region_mut(&mut self, start: u64, end: u64) -> Option<&mut MemRegion> {
        for r in &mut self.regions {
            // TODO: fast binary search
            // TODO: what if it crosses two regions?
            if start >= r.start && end <= r.end {
                return Some(r);
            }
        }
        None
    }

    pub fn load_bin_file(&mut self, addr: u64, fname: &str) -> Result<(), Box<dyn Error>> {
        // TODO: check if exists
        let f_size = fs::metadata(fname)?.len();
        let mr: &mut MemRegion = self.find_mem_region_mut(addr, addr + f_size)
                                     .ok_or("region is too small")?;
        let mut f = File::open(fname)?;
        f.read(&mut mr.m[..])?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn read8(&self, addr: u64) -> u8 {
        if let Some(mr) = self.find_mem_region(addr, 1) {
            return mr.m[addr as usize];
        }
        return 0;
    }

    #[allow(dead_code)]
    pub fn write8(&mut self, addr: u64, val: u8) {
        if let Some(mr) = self.find_mem_region_mut(addr, 1) {
            mr.m[addr as usize] = val;
        }
    }

    pub fn read32(&self, addr: u64) -> u32 {
        let mr = self.find_mem_region(addr, 4).unwrap();
        let offs = (addr - mr.start) as usize;
        let b0 = mr.m[offs] as u32;
        let b1 = mr.m[offs + 1] as u32;
        let b2 = mr.m[offs + 2] as u32;
        let b3 = mr.m[offs + 3] as u32;
        // little endian
        b3 << 24 | b2 << 16 | b1 << 8 | b0
    }

    pub fn dump_hex(&self, addr: u64, size: u64) {
        let aligned_addr = addr & !0xf_u64;
        let aligned_size = (size + 16) & !0xf_u64;
        let m = &self.find_mem_region(aligned_addr, aligned_addr + aligned_size)
                     .unwrap()
                     .m;
        let mut line = String::with_capacity(size as usize + 32);
        // TODO: optimize - slow
        let mut pr_str = String::with_capacity(22);
        line.push_str(&format!("{:016x} ", aligned_addr));
        for (i, b) in m[..aligned_size as usize].iter().enumerate() {
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

#[test]
pub fn test_pmem_dump() {
    let pmem = Pmem::default();
    let b = pmem.read8(0);
    assert!(b == 0)
}

#[test]
pub fn test_read32_le() {
    let mut m = Pmem::default();
    m.alloc_region(0, 1024);
    m.write8(0, 0xef);
    m.write8(1, 0xbe);
    m.write8(2, 0xad);
    m.write8(3, 0xde);
    let v: u32 = m.read32(0);
    assert!(v == 0xdeadbeef);
}
