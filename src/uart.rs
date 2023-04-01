use crate::device::DevIO;

pub struct Uart {
    id: String,
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
    pub fn new(id: String) -> Uart {
        Uart { id }
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

    fn write32(&mut self, addr: u64, val: u32) {
        match addr {
            TXDATA => {
                let byte = (val & 0xff) as u8;
                let byte_ascii = byte as char;
                println!("UART-{0} output: hex: 0x{byte:02x}, ascii: {byte_ascii}",
                         self.id);
            }
            _ => panic!("DBG: Uart: register {addr:x} write not implemented"),
        };
    }
}
