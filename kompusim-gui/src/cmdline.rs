use std::path::PathBuf;

use clap::{arg, Parser, Subcommand};

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
