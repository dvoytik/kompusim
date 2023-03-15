use rv64i_cpu::RV64ICpu;

mod csr;
mod pmem;
mod rv64i_cpu;

const LOAD_ADDRESS: u64 = 0x0000_0000_8000_0000;

fn main() {
    let mut pmem = pmem::Pmem::default();
    pmem.alloc_region(LOAD_ADDRESS, 4 * 1024);

    pmem.load_bin_file(
        LOAD_ADDRESS,
        "tests/uart_hello_world/out/uart_hello_world.bin",
    )
    .unwrap();
    pmem.dump_hex(LOAD_ADDRESS, 80);

    let mut cpu0 = RV64ICpu::new(pmem);
    cpu0.regs.pc = LOAD_ADDRESS;
    cpu0.run_until(0x0000000080000008);
    //cpu0.run_until(0x0000000080000014);
}
