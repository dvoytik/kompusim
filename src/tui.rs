use text_io::read;
use anstream::println;
use owo_colors::OwoColorize;

pub enum TuiMenuOpt {
    Step,
    Continue,
    Quit,
    ToggleTracing, // Enables/disable tracing
}

fn green_line() {
    println!("{}", "─────────────────────────────────────────────────────────────".green().bold());
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
                 s - step one instruction\n\
                 t - toggle tracing (enabled: {enabled_tracing})\n\
                 pr - print registers\n\
                 b <addr> - set breakpoint\n\
                 lb       - list breakpoints\n\
                 dm <addr|reg> - dump memory at address <addr>");
        } else if l.contains("q") {
            break TuiMenuOpt::Quit;
        } else if l.contains("c") {
            break TuiMenuOpt::Continue;
        } else if l.contains("s") {
            break TuiMenuOpt::Step;
        } else if l.contains("t") {
            break TuiMenuOpt::ToggleTracing;
        } else {
            println!("unrecognized command");
        }
    };
    green_line();
    selected_option
}
