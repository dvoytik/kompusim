use anstream::println;
use owo_colors::OwoColorize;
use text_io::read;

use kompusim::rv64i_cpu::RV64IURegs;

pub enum TuiMenuOpt {
    Step,
    Continue,
    Quit,
    ToggleTracing, // Enables/disable tracing
    PrintRegisters,
}

fn green_line() {
    println!(
        "{}",
        "─────────────────────────────────────────────────────────────"
            .green()
            .bold()
    );
}

pub fn interactive_menu(enabled_tracing: bool) -> TuiMenuOpt {
    let selected_option = loop {
        green_line();
        print!("command (h for Help): ");
        let l: String = read!("{}\n");
        if l.contains("help") || l.contains("h") {
            println!(
                "q - exit Kompusim\n\
                 c - continue (run until hitting a breakpoint)\n\
                 s     - step one instruction\n\
                 s <N> - step <N> instructions\n\
                 t - toggle tracing (enabled: {enabled_tracing})\n\
                 pr - print registers\n\
                 b <addr> - set breakpoint\n\
                 lb       - list breakpoints\n\
                 dm <addr|reg> - dump memory at address <addr>"
            );
        } else if l.contains("q") {
            break TuiMenuOpt::Quit;
        } else if l.contains("c") {
            break TuiMenuOpt::Continue;
        } else if l.contains("s") {
            break TuiMenuOpt::Step;
        } else if l.contains("t") {
            break TuiMenuOpt::ToggleTracing;
        } else if l.contains("pr") {
            break TuiMenuOpt::PrintRegisters;
        } else {
            println!("unrecognized command");
        }
    };
    green_line();
    selected_option
}

pub fn print_regs(regs: &RV64IURegs) {
    println!(" x1 (ra): {:016x} | {0:064b}", regs.x[1]);
    println!(" x2 (sp): {:016x} | {0:064b}", regs.x[2]);
    println!(" x3 (gp): {:016x} | {0:064b}", regs.x[3]);
    println!(" x4 (tp): {:016x} | {0:064b}", regs.x[4]);
    println!(" x5 (t0): {:016x} | {0:064b}", regs.x[5]);
    println!(" x6 (t1): {:016x} | {0:064b}", regs.x[6]);
    println!(" x7 (t2): {:016x} | {0:064b}", regs.x[7]);
    println!(" x8 (s0): {:016x} | {0:064b}", regs.x[8]);
    println!(" x9 (s1): {:016x} | {0:064b}", regs.x[9]);
    println!("x10 (a0): {:016x} | {0:064b}", regs.x[10]);
    println!("x11 (a1): {:016x} | {0:064b}", regs.x[11]);
    println!("x12 (a2): {:016x} | {0:064b}", regs.x[12]);
    println!("x13 (a3): {:016x} | {0:064b}", regs.x[13]);
    println!("      pc: {:016x} | {0:064b}", regs.pc)
}
