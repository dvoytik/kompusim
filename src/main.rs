mod tui;

use clap::{arg, Parser, Subcommand};
use std::path::PathBuf;

use kompusim::bus;
use kompusim::device::Device;
use kompusim::ram;
use kompusim::rv64i_cpu::RV64ICpu;
use kompusim::uart::Uart;
use tui::TuiMenuCmd;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    arg_required_else_help(true),
    hide_possible_values(true)
)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    // Disasm {},
    /// Load a binary file and execute it
    Exec {
        /// Address in hex where to load the binary (e.g, 0x0000000080000000)
        #[arg(short, long)]
        load_addr: String,

        /// Path to the binary file
        #[arg(long)]
        bin: PathBuf,

        /// RAM size in KiBytes (defult 4)
        #[arg(short, long)]
        ram: Option<u64>,

        /// Breakpont - "auto" or address in hex (e.g. 0x0000000080000014)
        #[arg(short, long)]
        breakpoint: Option<String>,

        /// Maximum number of instruction before stop
        #[arg(long)]
        max_instr: Option<u64>,

        /// Run in with interactive menu, don't execute
        #[arg(short, long, action=clap::ArgAction::SetTrue)]
        interactive: Option<bool>,
    },
}

/// Convert hex str (e.g, "0x9393") to u64
fn hex_to_u64(hex_str: &str, err_msg: &str) -> u64 {
    u64::from_str_radix(hex_str.trim_start_matches("0x"), 16).expect(err_msg)
}

fn main() {
    let args = Args::parse();

    match &args.command {
        // Some(Commands::Disasm {}) => {}
        Some(Commands::Exec {
            load_addr,
            bin,
            ram,
            breakpoint,
            max_instr,
            interactive,
        }) => {
            let max_instr = max_instr.unwrap_or(u64::MAX);

            let mut break_point: Option<u64> = None;
            if let Some(breakpoint) = breakpoint {
                if breakpoint.find("auto").is_none() {
                    break_point = Some(hex_to_u64(breakpoint, "wrong hex in --breakpoint"));
                }
                // TODO: handel auto breakpoint case
            }

            let ram_sz = ram.unwrap_or(4) * 1024;

            let addr = hex_to_u64(load_addr, "wrong hex in --load_addr");
            let mut ram = ram::Ram::new(addr, ram_sz);
            ram.load_bin_file(addr, bin).unwrap();
            println!("Loaded {bin:?} at 0x{addr:x}");
            // ram.dump_hex(addr, 80);

            let mut bus = bus::Bus::new();
            bus.attach_ram(ram);
            bus.attach_device(Device::new(
                Box::new(Uart::new("0".to_string())),
                0x1001_0000,
                0x20,
            ));
            let mut cpu0 = RV64ICpu::new(bus);
            cpu0.regs.pc = addr;

            if let Some(breakpoint) = break_point {
                cpu0.add_breakpoint(breakpoint)
            }

            if interactive.unwrap_or(false) {
                cpu0.enable_tracing(true); // TODO: remove tracing from rv64i_cpu
                loop {
                    match tui::interactive_menu() {
                        TuiMenuCmd::Quit => break,
                        TuiMenuCmd::Step(n_steps) => {
                            for _ in 0..n_steps {
                                let before_regs = cpu0.get_regs().clone();
                                let pc = cpu0.get_pc();
                                tui::print_instr_listing(cpu0.get_n_instr(pc - 4, 3), pc - 4, pc);
                                let _ = cpu0.exec_continue(1);
                                let after_regs = cpu0.get_regs();
                                tui::print_changed_regs(&before_regs, after_regs);
                            }
                        }
                        TuiMenuCmd::Continue => {
                            let _ = cpu0.exec_continue(max_instr);
                        }
                        TuiMenuCmd::PrintAllRegisters => {
                            // TODO: highlight changed registers - store old state, calc diff
                            tui::print_regs(cpu0.get_regs())
                        }
                        TuiMenuCmd::PrintRegister(reg_i) => {
                            tui::print_reg(cpu0.get_regs(), reg_i);
                        }
                        TuiMenuCmd::DumpMem(addr, size) => {
                            tui::dump_mem(cpu0.get_ram(addr, size), addr, size)
                        }
                        TuiMenuCmd::ListInstr(pc_offset, n_instr) => {
                            let pc = cpu0.get_pc();
                            let start = (pc as i64 + pc_offset as i64) as u64;
                            tui::print_instr_listing(cpu0.get_n_instr(start, n_instr), start, pc);
                        }
                    }
                }
            } else {
                let _ = cpu0.exec_continue(max_instr);
            }
        }
        None => {}
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Args::command().debug_assert()
}
