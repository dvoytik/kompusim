use anstream::println;
use owo_colors::OwoColorize;
use text_io::read;

use kompusim::{
    bits::BitOps,
    rv64i_cpu::RV64IURegs,
    rv64i_disasm::{disasm, reg_idx2abi},
};

#[derive(PartialEq)]
pub enum TuiMenuCmd {
    Step,
    Continue,
    Quit,
    ToggleTracing, // Enables/disable tracing
    PrintRegister(u8),
    PrintAllRegisters,
    DumpMem(u64, u64),
}

fn print_green_line() {
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

/// Parses "print register" command, e.g. "pr x1", "pr t0"
fn parse_pr(s: &str) -> Option<u8> {
    if let Some(reg_s_i) = s.trim().find(|c: char| c.is_ascii_whitespace()) {
        let reg_s = &s[reg_s_i..].trim();
        if reg_s.starts_with('x') {
            if let Ok(reg_i) = u8::from_str_radix(reg_s.trim_start_matches("x"), 10) {
                if reg_i <= 31 {
                    return Some(reg_i);
                }
            }
        } else if reg_s.starts_with("ra") {
            return Some(1);
        } else if reg_s.starts_with("sp") {
            return Some(2);
        } else if reg_s.starts_with("gp") {
            return Some(3);
        } else if reg_s.starts_with("tp") {
            return Some(4);
        } else if reg_s.starts_with('t') {
            // t0 ... t6
            if let Ok(reg_i) = u8::from_str_radix(reg_s.trim_start_matches("t"), 10) {
                if reg_i < 3 {
                    return Some(reg_i as u8 + 5);
                } else if reg_i >= 3 && reg_i <= 6 {
                    return Some(reg_i + 25);
                }
            }
        } else if reg_s.starts_with('a') {
            // a0 ... 07
            if let Ok(reg_i) = u8::from_str_radix(reg_s.trim_start_matches("a"), 10) {
                return Some(reg_i as u8 + 10);
            }
        } else if reg_s.starts_with('s') {
            // s0, s1, s2 ... s11
            if let Ok(reg_i) = u8::from_str_radix(reg_s.trim_start_matches("s"), 10) {
                if reg_i <= 1 {
                    return Some(reg_i as u8 + 8);
                } else if reg_i >= 2 && reg_i <= 11 {
                    return Some(reg_i + 16);
                }
            }
        }
    }
    None
}

fn parse_command(l: String, enabled_tracing: bool) -> Option<TuiMenuCmd> {
    if l.len() == 0 {
        return None;
    }
    const MAX_CMD_SZ: usize = 2;
    let cmd_sz = if l.len() < MAX_CMD_SZ {
        l.len()
    } else {
        MAX_CMD_SZ
    };
    let cmd = &l[..cmd_sz];
    if l.starts_with("help") || l.starts_with("h") {
        println!(
            "q - exit Kompusim\n\
                 e - enable/disable explain mode (NOT IMPLEMENTED)\n\
                 c - continue (run until hitting a breakpoint)\n\
                 s     - step one instruction\n\
                 s <N> - step <N> instructions (NOT IMPLEMENTED)\n\
                 sa    - step automatically until a breakpoint (NOT IMPLEMENTED)\n\
                 t - toggle tracing (enabled: {enabled_tracing})\n\
                 pr     - print all registers\n\
                 pr <r> - print register <r>\n\
                 b <addr> - set breakpoint (NOT IMPLEMENTED)\n\
                 lb       - list breakpoints (NOT IMPLEMENTED)\n\
                 dm <addr> <size> - dump memory at address <addr>"
        );
        // TODO: add dm x0 <size> dump from pointer in x0
    } else if cmd.starts_with("q") {
        return Some(TuiMenuCmd::Quit);
    } else if cmd.starts_with("c") {
        return Some(TuiMenuCmd::Continue);
    } else if cmd.starts_with("s") {
        return Some(TuiMenuCmd::Step);
    } else if cmd.starts_with("t") {
        return Some(TuiMenuCmd::ToggleTracing);
    } else if cmd.starts_with("pr") {
        if let Some(reg_i) = parse_pr(&l) {
            return Some(TuiMenuCmd::PrintRegister(reg_i));
        } else {
            return Some(TuiMenuCmd::PrintAllRegisters);
        }
    } else if cmd.starts_with("dm") {
        if let Some((addr, size)) = parse_dm(&l) {
            return Some(TuiMenuCmd::DumpMem(align16(addr), align16_nonzero(size)));
        } else {
            println!("format shoud be: dm <hex_addr> <size>. Example:\ndm 0x00001234 1024");
        }
    } else {
        println!("unrecognized command");
    }
    None
}

pub fn interactive_menu(enabled_tracing: bool) -> TuiMenuCmd {
    let selected_option = loop {
        print_green_line();
        print!("command (h for Help): ");
        let l: String = read!("{}\n");
        if let Some(valid_menu_opt) = parse_command(l, enabled_tracing) {
            break valid_menu_opt;
        }
    };
    print_green_line();
    selected_option
}

fn reg_hex(v: u64) -> String {
    format!(
        "{:04x}_{:04x}_{:04x}_{:04x}",
        v.bits(63, 48),
        v.bits(47, 32),
        v.bits(31, 16),
        v.bits(15, 0)
    )
}

/// Print one register
pub fn print_reg(regs: &RV64IURegs, reg_i: u8) {
    println!("{}", reg2str(regs, reg_i));
    let reg_i = reg_i as usize;
    // TODO: bits
    // TODO: octal
    println!("Unsigned: {}", regs.x[reg_i] as u64);
    println!("Signed:   {}", regs.x[reg_i] as i64);
}

pub fn print_regs(regs: &RV64IURegs) {
    println!(
        " x1 (ra): {} | x17 (a7):  {}",
        reg_hex(regs.x[1]),
        reg_hex(regs.x[17])
    );
    println!(
        " x2 (sp): {} | x18 (s2):  {}",
        reg_hex(regs.x[2]),
        reg_hex(regs.x[18])
    );
    println!(
        " x3 (gp): {} | x19 (s3):  {}",
        reg_hex(regs.x[3]),
        reg_hex(regs.x[19])
    );
    println!(
        " x4 (tp): {} | x20 (s4):  {}",
        reg_hex(regs.x[4]),
        reg_hex(regs.x[20])
    );
    println!(
        " x5 (t0): {} | x21 (s5):  {}",
        reg_hex(regs.x[5]),
        reg_hex(regs.x[21])
    );
    println!(
        " x6 (t1): {} | x22 (s6):  {}",
        reg_hex(regs.x[6]),
        reg_hex(regs.x[22])
    );
    println!(
        " x7 (t2): {} | x23 (s7):  {}",
        reg_hex(regs.x[7]),
        reg_hex(regs.x[23])
    );
    println!(
        " x8 (s0): {} | x24 (s8):  {}",
        reg_hex(regs.x[8]),
        reg_hex(regs.x[24])
    );
    println!(
        " x9 (s1): {} | x25 (s9):  {}",
        reg_hex(regs.x[9]),
        reg_hex(regs.x[25])
    );
    println!(
        "x10 (a0): {} | x26 (s10): {}",
        reg_hex(regs.x[10]),
        reg_hex(regs.x[26])
    );
    println!(
        "x11 (a1): {} | x27 (s11): {}",
        reg_hex(regs.x[11]),
        reg_hex(regs.x[27])
    );
    println!(
        "x12 (a2): {} | x28 (t3):  {}",
        reg_hex(regs.x[12]),
        reg_hex(regs.x[28])
    );
    println!(
        "x13 (a3): {} | x29 (t4):  {}",
        reg_hex(regs.x[13]),
        reg_hex(regs.x[29])
    );
    println!(
        "x14 (a4): {} | x30 (t5):  {}",
        reg_hex(regs.x[14]),
        reg_hex(regs.x[30])
    );
    println!(
        "x15 (a5): {} | x31 (t6):  {}",
        reg_hex(regs.x[15]),
        reg_hex(regs.x[31])
    );
    println!("x16 (a6): {} |", reg_hex(regs.x[16]));
    println!("      pc: {} |", reg_hex(regs.pc));
}

fn reg2str(regs: &RV64IURegs, ri: u8) -> String {
    if ri == 0 {
        return "x0 (zero)".to_string();
    }
    let reg_val = regs.x[ri as usize];
    let r_abi = reg_idx2abi(ri);
    format!("x{ri} ({r_abi}): {}", reg_hex(reg_val))
}

pub fn print_changed_regs(before_regs: &RV64IURegs, after_regs: &RV64IURegs) {
    print_green_line();
    for i in 1..31 {
        if before_regs.x[i] != after_regs.x[i] {
            println!(
                "{} → {}",
                reg2str(before_regs, i as u8),
                reg2str(after_regs, i as u8)
            )
        }
    }

    // How has PC changed
    let jump = after_regs.pc - before_regs.pc;
    if jump == 0 {
        println!("PC: 0x{:x} ↩", after_regs.pc);
    } else {
        let sign = if jump > 0 { '+' } else { '-' };
        println!(
            "PC: 0x{0:x} {sign} 0x{jump:x} = 0x{1:x}",
            before_regs.pc, after_regs.pc
        );
    }
}

fn print_instr(instr: Option<u32>, addr: u64, instr_current: bool) {
    if instr.is_none() {
        return;
    }
    let instr = instr.unwrap();
    let cur_char = if instr_current { '→' } else { ' ' };
    let s = format!(
        "{} 0x{addr:08x} | 0x{instr:08x} | {}",
        cur_char,
        disasm(instr, addr)
    );
    if instr_current {
        println!("{}", s.bold().green());
    } else {
        println!("{}", s);
    }
}

pub fn print_3instr(
    prev_instr: Option<u32>,
    cur_instr: Option<u32>,
    next_instr: Option<u32>,
    addr: u64,
) {
    print_instr(prev_instr, addr - 4, false);
    print_instr(cur_instr, addr, true);
    print_instr(next_instr, addr + 4, false);
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
            break;
        }
        if i > 0 && i % 16 == 0 {
            line.push_str(&format!("| {} |\n", pr_str));
            line.push_str(&format!("{:016x} ", aligned_addr + i));
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

    assert!(parse_pr("pr x1") == Some(1));
    assert!(parse_pr("pr s11") == Some(27));
    assert!(parse_pr("pr	    x15 ") == Some(15));
    assert!(parse_pr("pr x32") == None);

    assert!(parse_command("".to_string(), true) == None);
    assert!(parse_command("c".to_string(), true) == Some(TuiMenuCmd::Continue));
    assert!(
        parse_command("dm 0x800000c0 16".to_string(), true)
            == Some(TuiMenuCmd::DumpMem(0x800000c0, 16))
    );

    assert!(reg_hex(0x1234_5678_9abc_def0) == "1234_5678_9abc_def0".to_string());
    assert!(reg_hex(0x1234) == "0000_0000_0000_1234".to_string());
    assert!(reg_hex(0xF234_0000_0000_0000) == "f234_0000_0000_0000".to_string());
}
