use crate::device::Device;
use crate::ram::Ram;
use core::fmt;
use std::error::Error;

#[derive(Debug)]
struct BusError {
    details: String,
}

impl fmt::Display for BusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for BusError {
    fn description(&self) -> &str {
        &self.details
    }
}

// TODO: use generics
enum BusAgent {
    Ram(Ram),
    Device(Device),
}

impl BusAgent {
    pub fn read8(&self, addr: u64) -> u8 {
        match self {
            BusAgent::Ram(ram) => ram.read8(addr),
            BusAgent::Device(dev) => dev.read8(addr),
        }
    }

    pub fn read32(&self, addr: u64) -> u32 {
        match self {
            BusAgent::Ram(ram) => ram.read32(addr),
            BusAgent::Device(dev) => dev.read32(addr),
        }
    }

    pub fn read64(&self, addr: u64) -> u64 {
        match self {
            BusAgent::Ram(ram) => ram.read64(addr),
            BusAgent::Device(dev) => dev.read64(addr),
        }
    }

    pub fn write8(&mut self, addr: u64, val: u8) {
        match self {
            BusAgent::Ram(ram) => ram.write8(addr, val),
            BusAgent::Device(dev) => dev.write8(addr, val),
        }
    }

    pub fn write32(&mut self, addr: u64, val: u32) {
        match self {
            BusAgent::Ram(ram) => ram.write32(addr, val),
            BusAgent::Device(dev) => dev.write32(addr, val),
        }
    }

    pub fn write64(&mut self, addr: u64, val: u64) {
        match self {
            BusAgent::Ram(ram) => ram.write64(addr, val),
            BusAgent::Device(dev) => dev.write64(addr, val),
        }
    }

    pub fn get_ram(&self, addr: u64, size: u64) -> Option<&[u8]> {
        match self {
            BusAgent::Device(_) => None,
            BusAgent::Ram(ram) => ram.get_ram(addr, size),
        }
    }
}

struct AddrRegion {
    start: u64,
    end: u64,
    agent: BusAgent,
}

#[derive(Default)]
pub struct Bus {
    regions: Vec<AddrRegion>, // TODO: should be sorted
    ram_start: u64,
}

impl Bus {
    pub fn new() -> Bus {
        Default::default()
    }

    #[allow(dead_code)]
    pub fn new_with_ram(start: u64, size: u64) -> Bus {
        let ram = Ram::new(start, start + size);
        let mut bus = Bus::new();
        bus.attach_ram(ram);
        bus
    }

    pub fn attach_ram(&mut self, ram: Ram) {
        // for now and for simplicyt let's support only one RAM region
        assert!(self.ram_start == 0);
        self.ram_start = ram.start;
        self.regions.push(AddrRegion {
            start: ram.start,
            end: ram.end,
            agent: BusAgent::Ram(ram),
        });
    }

    pub fn attach_device(&mut self, dev: Device) {
        // TODO: insert in sorted order - search optimization
        if self
            .find_addr_region(dev.start, dev.end - dev.start)
            .is_some()
        {
            panic!("address region is occupied")
        }
        self.regions.push(AddrRegion {
            start: dev.start,
            end: dev.end,
            agent: BusAgent::Device(dev),
        });
    }

    fn find_addr_region(&self, start: u64, size: u64) -> Option<&AddrRegion> {
        let end = start + size;
        // TODO: fast binary search
        // TODO: what if it crosses two regions?
        self.regions
            .iter()
            .find(|&r| start >= r.start && end <= r.end)
    }

    fn find_addr_region_mut(&mut self, start: u64, end: u64) -> Option<&mut AddrRegion> {
        // TODO: fast binary search
        // TODO: what if it crosses two regions?
        self.regions
            .iter_mut()
            .find(|r| start >= r.start && end <= r.end)
    }

    /// Read byte
    /// TODO: implement exception logic - return Error
    pub fn read8(&self, addr: u64) -> u8 {
        if let Some(ar) = self.find_addr_region(addr, 1) {
            ar.agent.read8(addr)
        } else {
            panic!("ERR: read8 bus fault @ 0x{addr:x}");
        }
    }

    #[allow(dead_code)]
    pub fn write8(&mut self, addr: u64, val: u8) {
        if let Some(ar) = self.find_addr_region_mut(addr, 1) {
            ar.agent.write8(addr, val)
        } else {
            panic!("ERR: write8 bus fault: 0x{addr:x}");
        }
    }

    // Little Endian 32 bit read
    pub fn read32(&self, addr: u64) -> u32 {
        if let Some(ar) = self.find_addr_region(addr, 4) {
            ar.agent.read32(addr)
        } else {
            panic!("ERR: read32 bus fault @ 0x{addr:x}");
        }
    }

    pub fn read64(&self, addr: u64) -> u64 {
        if let Some(ar) = self.find_addr_region(addr, 8) {
            ar.agent.read64(addr)
        } else {
            panic!("ERR: read64 bus fault @ 0x{addr:x}");
        }
    }

    #[allow(dead_code)]
    pub fn write32(&mut self, addr: u64, val: u32) {
        if let Some(ar) = self.find_addr_region_mut(addr, 4) {
            ar.agent.write32(addr, val)
        } else {
            panic!("ERR: write32 bus fault @ 0x{addr:x}");
        }
    }

    pub fn write64(&mut self, addr: u64, val: u64) {
        if let Some(ar) = self.find_addr_region_mut(addr, 8) {
            ar.agent.write64(addr, val)
        } else {
            panic!("ERR: write64 bus fault @ 0x{addr:x}");
        }
    }

    pub fn get_ram(&self, addr: u64, size: u64) -> Option<&[u8]> {
        if let Some(ar) = self.find_addr_region(addr, size) {
            ar.agent.get_ram(addr, size)
        } else {
            None
        }
    }

    pub fn set_ram_sz(&mut self, ram_sz: u64) {
        // find RAM region
        if let Some(ar) = self.find_addr_region_mut(self.ram_start, 4) {
            if let BusAgent::Ram(ram) = &mut ar.agent {
                ram.resize(ram_sz);
                return;
            }
        }
        panic!("Could not find RAM region")
    }

    pub fn load_image(&mut self, addr: u64, image: &'static [u8]) -> Result<(), Box<dyn Error>> {
        if let Some(ar) = self.find_addr_region_mut(addr, image.len() as u64) {
            if let BusAgent::Ram(ram) = &mut ar.agent {
                return ram.load_image(addr, image);
            }
        }
        Err(Box::new(BusError {
            details: "No suitable RAM address region".to_string(),
        }))
    }

    /// Loads a binary file image into ram
    pub fn load_file(
        &mut self,
        addr: u64,
        file_path: &std::path::PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(ar) = self.find_addr_region_mut(addr, 8) {
            if let BusAgent::Ram(ram) = &mut ar.agent {
                return ram.load_bin_file(addr, file_path);
            }
        }
        Err(Box::new(BusError {
            details: "No suitable RAM address region".to_string(),
        }))
    }
}

#[test]
pub fn test_ram_read_write() {
    let mut bus = Bus::new_with_ram(0, 4 * 1024);
    assert!(bus.read8(0) == 0);
    bus.write8(1, 0x55);
    assert!(bus.read8(1) == 0x55)
}

#[test]
pub fn test_read32_le() {
    let mut bus = Bus::new_with_ram(0, 4 * 1024);
    bus.write8(0, 0xef);
    bus.write8(1, 0xbe);
    bus.write8(2, 0xad);
    bus.write8(3, 0xde);
    let v: u32 = bus.read32(0);
    assert!(v == 0xdeadbeef);
}

#[test]
pub fn test_write32_le() {
    let mut bus = Bus::new_with_ram(0, 4 * 1024);
    bus.write32(0, 0x_dead_beef);
    assert!(bus.read8(0) == 0xef);
    assert!(bus.read8(1) == 0xbe);
    assert!(bus.read8(2) == 0xad);
    assert!(bus.read8(3) == 0xde);
}

#[test]
fn test_load_image_from_static() {
    static BIN: &[u8] = &[0x55; 1024];
    let mut bus = Bus::new_with_ram(0, 4 * 1024);
    bus.load_image(0x4, BIN).unwrap();
    assert!(bus.read32(0x4) == 0x5555_5555);
    assert!(bus.read32(0x0) == 0x0000_0000);
}
