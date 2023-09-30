use std::path::PathBuf;

use clap::{arg, Parser, Subcommand};
use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::{alpha0, digit1},
    sequence::tuple,
    IResult,
};

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    arg_required_else_help(false),
    hide_possible_values(true)
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<CmdLCommand>,
}

#[derive(Subcommand)]
pub enum CmdLCommand {
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
        #[arg(long)]
        ram: Option<String>,

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

/// Parses strings like "100MB" and returns size in bytes
pub fn parse_size_with_suffix(input: &str) -> Option<u64> {
    // Converts "KiB", "MB" to 1024, 1024*1024, etc.
    fn suffix_to_multiplier(input: &str) -> Option<u64> {
        let suffix_is_mb: IResult<&str, &str> = alt((
            tag_no_case("kib"),
            tag_no_case("kb"),
            tag_no_case("k"),
            tag_no_case("mib"),
            tag_no_case("mb"),
            tag_no_case("m"),
            tag_no_case("gib"),
            tag_no_case("gb"),
            tag_no_case("g"),
        ))(input);
        if let Ok((remainder, sfx)) = suffix_is_mb {
            if !remainder.is_empty() {
                return None;
            }
            let multiplier = match sfx.chars().next().unwrap() {
                'k' | 'K' => 1024,
                'm' | 'M' => 1024 * 1024,
                'g' | 'G' => 1024 * 1024 * 1024,
                _ => return None,
            };
            return Some(multiplier);
        }
        None
    }
    let digit_alpha: IResult<&str, (&str, &str)> = tuple((digit1, alpha0))(input);
    if let Ok((remainder, (digits, suffix_alpha))) = digit_alpha {
        // forbid strange characters at the end
        if !remainder.is_empty() {
            return None;
        }
        let Ok(number) = digits.parse::<u64>() else {
            return None;
        };
        if suffix_alpha.is_empty() {
            return Some(number);
        } else {
            if let Some(multiplier) = suffix_to_multiplier(suffix_alpha) {
                return number.checked_mul(multiplier);
            }
        }
    }
    None
}

#[test]
fn test_parse_size_in_bytes() {
    assert_eq!(parse_size_with_suffix("1"), Some(1));
    assert_eq!(parse_size_with_suffix("9999"), Some(9999));
    // Kilobyte
    assert_eq!(parse_size_with_suffix("333k"), Some(333 * 1024));
    // Megabyte
    assert_eq!(parse_size_with_suffix("1mb"), Some(1024 * 1024));
    assert_eq!(parse_size_with_suffix("100Mib"), Some(100 * 1024 * 1024));
    assert_eq!(parse_size_with_suffix("100MiB"), Some(100 * 1024 * 1024));
    // Gigabyte
    assert_eq!(
        parse_size_with_suffix("100gb"),
        Some(100 * 1024 * 1024 * 1024)
    );

    // assert_eq!(parse_size_with_suffix("1_0"), Some(10)); // TODO

    // Negative
    assert!(parse_size_with_suffix("-1").is_none());
    assert!(parse_size_with_suffix("-1+MiB").is_none());
    assert!(parse_size_with_suffix("1+MiB").is_none());
    assert!(parse_size_with_suffix("1Megs").is_none());
    assert!(parse_size_with_suffix("1gi").is_none());
    assert!(parse_size_with_suffix("MiB1").is_none());
    assert!(parse_size_with_suffix(".1M").is_none());
    assert!(parse_size_with_suffix("0.001M").is_none());
    assert_ne!(parse_size_with_suffix("1mb"), Some(1024 * 1025));

    // Overflow is handled
    assert!(parse_size_with_suffix("100000000000000GB").is_none())
}
