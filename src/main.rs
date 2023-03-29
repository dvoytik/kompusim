use std::path::PathBuf;

use clap::{arg, Parser, Subcommand};
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

#[derive(Parser)]
//#[command(name = "Kompusim")]
//#[command(author = "Dmitry Voytik <voytikd@gmail.com>")]
//#[command(about = "RISC-V ISA simulator")]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    ram: Option<u64>,

    #[arg(short, long)]
    breakpoint: Option<u64>,

    #[arg(long)]
    max_instr: Option<u64>,

    #[arg(short, long)]
    interactive: Option<bool>,

    #[arg(short, long)]
    trace: Option<bool>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Load {
        #[arg(short, long)]
        address: String,

        #[arg(short, long)]
        run: bool,

        #[arg(short, long)]
        bin: PathBuf,
    },
}

const DEF_BRK_POINT: u64 = 0x0000_0000_8000_0014;

fn main() {
    let args = Args::parse();
    let max_instr = args.max_instr.unwrap_or(u64::MAX);
    let break_point = args.breakpoint.unwrap_or(DEF_BRK_POINT);

    let ram_sz = args.ram.unwrap_or(4 * 1024);
    match &args.command {
        Some(Commands::Load { address,
                              run: _,
                              bin, }) => {
            let addr = u64::from_str_radix(address.trim_start_matches("0x"), 16).unwrap();
            let mut ram = ram::Ram::new(addr, ram_sz);
            ram.load_bin_file(addr, bin).unwrap();
            ram.dump_hex(addr, 80);

            let mut bus = bus::Bus::new();
            bus.attach_ram(ram);
            bus.attach_device(Device::new(Box::new(Uart::new("0".to_string())), 0x1001_0000, 0x20));
            let mut cpu0 = RV64ICpu::new(bus);
            cpu0.regs.pc = addr;
            cpu0.run_until(break_point, max_instr);
        }
        None => {
            println!("nothing to do")
        }
    }
}
