#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::KompusimApp;
mod base_uregs;
pub mod cmdline;
mod console;
mod instr_decoder;
mod instr_list;
mod load_demo;
mod sim;
mod status_control;
