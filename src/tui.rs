use text_io::read;

pub enum TuiMenuOpt {
    Step,
    Continue,
    Quit,
}

pub fn interactive_menu(enabled_tracing: bool) -> TuiMenuOpt {
    let selected_option = loop {
        println!("------------------------------------------------------------");
        print!("command (h for Help): ");
        let l: String = read!("{}\n");
        if l.contains("help") || l.contains("h") {
            println!("s - step one instruction\nc - continue (until breakpoint or infinitely)\nq \
                      - exit Kompusim\nt - toggle tracing (enabled: {enabled_tracing})\nb <addr> \
                      - set breakpoint\nlb - list breakpoints\ndm <addr> - dump memory at \
                      address <addr>");
        } else if l.contains("c") {
            break TuiMenuOpt::Continue;
        } else if l.contains("s") {
            break TuiMenuOpt::Step;
        } else if l.contains("q") {
            break TuiMenuOpt::Quit;
        } else {
            println!("unrecognized command");
        }
    };
    println!("------------------------------------------------------------");
    selected_option
}
