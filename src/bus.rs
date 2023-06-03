use crate::device::Device;
use crate::ram::Ram;

// TODO: use generics
enum BusAgent {
    RAM(Ram),
    Device(Device),
}

impl BusAgent {
    pub fn read8(&self, addr: u64) -> u8 {
        match self {
            BusAgent::RAM(ram) => ram.read8(addr),
            BusAgent::Device(dev) => dev.read8(addr),
        }
    }

    pub fn read32(&self, addr: u64) -> u32 {
        match self {
            BusAgent::RAM(ram) => ram.read32(addr),
            BusAgent::Device(dev) => dev.read32(addr),
        }
    }

    pub fn write8(&mut self, addr: u64, val: u8) {
        match self {
            BusAgent::RAM(ram) => ram.write8(addr, val),
            BusAgent::Device(dev) => dev.write8(addr, val),
        }
    }

    pub fn write32(&mut self, addr: u64, val: u32) {
        match self {
            BusAgent::RAM(ram) => ram.write32(addr, val),
            BusAgent::Device(dev) => dev.write32(addr, val),
        }
    }

    pub fn get_ram(&self, addr: u64, size: u64) -> Option<&[u8]> {
        match self {
            BusAgent::Device(_) => None,
            BusAgent::RAM(ram) => ram.get_ram(addr, size),
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
        // TODO: insert in sorted order - search optimization
        self.regions.push(AddrRegion {
            start: ram.start,
            end: ram.end,
            agent: BusAgent::RAM(ram),
        });
    }

    pub fn attach_device(&mut self, dev: Device) {
        // TODO: insert in sorted order - search optimization
        if let Some(_) = self.find_addr_region(dev.start, dev.start - dev.end) {
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
        for r in &self.regions {
            // TODO: fast binary search
            // TODO: what if it crosses two regions?
            if start >= r.start && end <= r.end {
                return Some(r);
            }
        }
        None
    }

    fn find_addr_region_mut(&mut self, start: u64, end: u64) -> Option<&mut AddrRegion> {
        for r in &mut self.regions {
            // TODO: fast binary search
            // TODO: what if it crosses two regions?
            if start >= r.start && end <= r.end {
                return Some(r);
            }
        }
        None
    }

    /// Read byte
    /// TODO: implement exception logic - return Error
    pub fn read8(&self, addr: u64) -> u8 {
        if let Some(ar) = self.find_addr_region(addr, 1) {
            ar.agent.read8(addr)
        } else {
            panic!("DBG: read8 buf fault: 0x{addr:x}");
        }
    }

    #[allow(dead_code)]
    pub fn write8(&mut self, addr: u64, val: u8) {
        if let Some(ar) = self.find_addr_region_mut(addr, 1) {
            ar.agent.write8(addr, val)
        } else {
            panic!("DBG: write8 bus fault: 0x{addr:x}");
        }
    }

    // Little Endian 32 bit read
    pub fn read32(&self, addr: u64) -> u32 {
        if let Some(ar) = self.find_addr_region(addr, 4) {
            ar.agent.read32(addr)
        } else {
            // TODO: is this bus fault or we return 0xffff_ffff?
            0xffff_ffff
        }
    }

    #[allow(dead_code)]
    pub fn write32(&mut self, addr: u64, val: u32) {
        if let Some(ar) = self.find_addr_region_mut(addr, 4) {
            ar.agent.write32(addr, val)
        } else {
            panic!("DBG: write32 bus fault: 0x{addr:x}");
        }
    }

    pub fn get_ram(&self, addr: u64, size: u64) -> Option<&[u8]> {
        if let Some(ar) = self.find_addr_region(addr, size) {
            ar.agent.get_ram(addr, size)
        } else {
            None
        }
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
