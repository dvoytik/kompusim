pub trait Dev {
    // addr is local to the device, i.e = PA - Device.start
    fn read8(&self, addr: u64) -> u8;
    fn write8(&mut self, addr: u64, val: u8);
    fn read32(&self, addr: u64) -> u32;
    fn read64(&self, addr: u64) -> u64;
    fn write32(&mut self, addr: u64, val: u32);
    fn write64(&mut self, addr: u64, val: u64);
}

/// Device maintains absolute physical address.
pub struct Device {
    pub start: u64,
    pub end: u64,
    pub dev: Box<dyn Dev>,
}

impl Device {
    pub fn new(d: Box<dyn Dev>, start: u64, size: u64) -> Device {
        Device {
            start,
            end: start + size,
            dev: d,
        }
    }

    pub fn read8(&self, addr: u64) -> u8 {
        self.dev.read8(addr - self.start)
    }

    pub fn write8(&mut self, addr: u64, val: u8) {
        self.dev.write8(addr - self.start, val)
    }

    pub fn read32(&self, addr: u64) -> u32 {
        self.dev.read32(addr - self.start)
    }

    pub fn read64(&self, addr: u64) -> u64 {
        self.dev.read64(addr - self.start)
    }

    pub fn write32(&mut self, addr: u64, val: u32) {
        self.dev.write32(addr - self.start, val)
    }

    pub fn write64(&mut self, addr: u64, val: u64) {
        self.dev.write64(addr - self.start, val)
    }
}
