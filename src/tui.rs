use anstream::println;
use owo_colors::OwoColorize;
use text_io::read;

use kompusim::{
    rv64i_cpu::RV64IURegs,
    rv64i_disasm::{disasm, reg_idx2abi},
};

pub enum TuiMenuOpt {
    Step,
    Continue,
    Quit,
    ToggleTracing, // Enables/disable tracing
    PrintRegisters,
    DumpMem(u64, u64),
}

fn green_line() {
    println!(
        "{}",
        "─────────────────────────────────────────────────────────────"
            .green()
            .bold()
    );
}

/// Parses string "dm 0x10000 1024" to (0x10000, 1024)
fn parse_dm(s: &str) -> Option<(u64, u64)> {
    if let Some(addr_i) = s.trim().find(" ") {
        let s = &s[addr_i + 1..].trim();
        if let Some(size_i) = s.find(" ") {
            let addr_str = &s[..size_i].trim();
            let size_str = &s[size_i + 1..].trim();
            if let Ok(addr) = u64::from_str_radix(addr_str.trim_start_matches("0x"), 16) {
                if let Ok(size) = size_str.parse() {
                    return Some((addr, size));
                }
            }
        }
    }
    None
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
                 s <N> - step <N> instructions (NOT IMPLEMENTED)\n\
                 sa    - step automatically until a breakpoint (NOT IMPLEMENTED)\n\
                 t - toggle tracing (enabled: {enabled_tracing})\n\
                 pr - print registers\n\
                 b <addr> - set breakpoint (NOT IMPLEMENTED)\n\
                 lb       - list breakpoints (NOT IMPLEMENTED)\n\
                 dm <addr> <size> - dump memory at address <addr>"
            );
            // TODO: add dm x0 <size> dump from pointer in x0
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
        } else if l.contains("dm") {
            if let Some((addr, size)) = parse_dm(&l) {
                break TuiMenuOpt::DumpMem(align16(addr), align16_nonzero(size));
            } else {
                println!("format shoud be: dm <hex_addr> <size>. Example:\ndm 0x00001234 1024");
            }
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

fn reg2str(regs: &RV64IURegs, ri: u8) -> String {
    if ri == 0 {
        return "x0 (zero)".to_string();
    }
    let r_abi = reg_idx2abi(ri);
    format!(
        "x{ri} ({r_abi}): 0x{0:016x} | b'{0:064b}",
        regs.x[ri as usize]
    )
}

pub fn print_changed_regs(before_regs: &RV64IURegs, after_regs: &RV64IURegs) {
    for i in 1..31 {
        if before_regs.x[i] != after_regs.x[i] {
            println!(
                "{}\n   ↓\n{}",
                reg2str(before_regs, i as u8),
                reg2str(after_regs, i as u8)
            )
        }
    }

    // How has PC changed
    let jump = after_regs.pc - before_regs.pc;
    if jump == 0 {
        println!("PC: {} ↩", after_regs.pc);
    } else {
        let sign = if jump > 0 { '+' } else { '-' };
        println!(
            "PC: 0x{0:x} {sign} 0x{jump:x} = 0x{1:x}",
            before_regs.pc, after_regs.pc
        );
    }
}

pub fn print_instr(instr: u32, addr: u64) {
    println!(
        "PC: 0x{addr:08x} | I: 0x{instr:08x} | {}",
        disasm(instr, addr)
    );
}

#[inline(always)]
pub fn align16(n: u64) -> u64 {
    n & !0xf_u64
}

#[inline(always)]
pub fn align16_nonzero(n: u64) -> u64 {
    let n = n & !0xf_u64;
    if n == 0 {
        16
    } else {
        n
    }
}

pub fn dump_mem(m: Option<&[u8]>, addr: u64, size: u64) {
    if let None = m {
        println!("Wrong address or size");
        return;
    }
    let m = m.unwrap();
    let aligned_addr = align16(addr);
    let aligned_size = align16_nonzero(size);
    let mut line = String::with_capacity(size as usize + 32);
    // TODO: optimize - slow
    let mut pr_str = String::with_capacity(22);
    line.push_str(&format!("{:016x} ", aligned_addr));
    for (i, b) in m[..aligned_size as usize].iter().enumerate() {
        let i = i as u64;
        if i == size {
            if i % 16 != 0 {
                let mid_blank = if i % 16 < 8 { 1 } else { 0 };
                let left_blanks = mid_blank + 3 * (16 - (i % 16));
                line.push_str(&format!("{:1$}", " ", left_blanks as usize));
            }
            line.push_str(&format!("| {} |\n", pr_str));
            line.push_str(&format!("{:016x} ", aligned_addr + i + 16));
            break;
        }
        if i > 0 && i % 16 == 0 {
            line.push_str(&format!("| {} |\n", pr_str));
            line.push_str(&format!("{:016x} ", aligned_addr + i + 16));
            pr_str.clear();
        }
        if i % 8 == 0 {
            line.push_str(" ");
        }
        line.push_str(&format!("{:02x} ", b));
        pr_str.push(if *b >= 0x20 && *b <= 0x7e {
            *b as char
        } else {
            '.'
        })
    }
    println!("{}", line);
}

#[test]
fn test_tui_dm() {
    assert!(parse_dm("dm 0x1000 1000") == Some((0x1000, 1000)));
    assert!(parse_dm("dm 0x0 1") == Some((0x0, 1)));
    assert!(parse_dm("dm  0x2000   3000") == Some((0x2000, 3000)));
    assert!(parse_dm("dm 	 0x4000 	  10") == Some((0x4000, 10)));
}
