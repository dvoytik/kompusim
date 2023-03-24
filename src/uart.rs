use crate::device::DevIO;

pub struct Uart {
    // txdata: u32, // 0x00
    // rxdata: u32, // 0x04
    // txctrl: u32, // 0x08
    // rxctrl: u32, // 0x0c
    // ie:     u32, // 0x10
    // ip:     u32, // 0x14
    // div:    u32, // 0x18
}

const TXDATA: u64 = 0x00;

impl Uart {
    pub fn new() -> Uart {
        Uart {}
    }
}

// addr is local to the device, i.e bus_address - base_address
impl DevIO for Uart {
    fn read8(&self, _addr: u64) -> u8 {
        panic!("DBG: Uart: read8 is not supported")
    }

    fn write8(&mut self, _addr: u64, _val: u8) {
        panic!("DBG: Uart: write8 is not supported")
    }

    fn read32(&self, addr: u64) -> u32 {
        match addr {
            TXDATA => 0x0000_0000, // full always is 0, data is alway 0x00 on read
            _ => panic!("DBG: Uart: register read not implemented"),
        }
    }
}
