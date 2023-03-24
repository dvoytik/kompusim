use device::Device;
use rv64i_cpu::RV64ICpu;
use uart::Uart;

mod alu;
mod bits;
mod bus;
mod csr;
mod device;
mod ram;
mod rv64i_cpu;
mod uart;

const LOAD_ADDRESS: u64 = 0x0000_0000_8000_0000;

fn main() {
    let mut ram = ram::Ram::new(LOAD_ADDRESS, 4 * 1024);
    ram.load_bin_file(LOAD_ADDRESS,
                      "tests/uart_hello_world/out/uart_hello_world.bin")
       .unwrap();
    ram.dump_hex(LOAD_ADDRESS, 80);

    let mut bus = bus::Bus::new();
    bus.attach_ram(ram);
    bus.attach_device(Device::new(Box::new(Uart::new()), 0x1001_0000, 0x100));
    let mut cpu0 = RV64ICpu::new(bus);
    cpu0.regs.pc = LOAD_ADDRESS;
    cpu0.run_until(0x000000008000002c);
    // cpu0.run_until(0x0000000080000014);
}
