use std::iter::zip;

use egui::Color32;
use egui_extras::TableRow;
use egui_extras::{Column, TableBuilder};
use kompusim::rv64i_disasm::{disasm, instr_hex, u64_hex4};
use kompusim::rvc_dec::instr_is_rvc;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct InstrList {
    /// Is window open or not
    open: bool,
    font_size: usize,
    /// User setting - start address of instructions
    #[serde(skip)]
    user_start_addr: u64,
    /// User setting - number of instructions
    #[serde(skip)]
    user_num_instr: u64,
    #[serde(skip)]
    instr_cache: InstrCache,
}

impl Default for InstrList {
    fn default() -> InstrList {
        InstrList {
            open: true,
            font_size: 0,
            user_start_addr: crate::sim::DEFAULT_START_ADDRESS,
            user_num_instr: 64,
            instr_cache: InstrCache::default(),
        }
    }
}

impl InstrList {
    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn show_if_opened(
        &mut self,
        ui_ctx: &egui::Context,
        instructions: (&Vec<u8>, u64),
        pc: u64,
    ) {
        let mut open = self.open;
        egui::Window::new("Instructions")
            .open(&mut open)
            .resizable(true)
            .default_width(400.0)
            .show(ui_ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.show_table(ui, instructions, pc));
            });
        self.open = open;
    }

    /// instructions - (instructions_array, start_addres)
    fn show_table(&mut self, ui: &mut egui::Ui, instructions: (&Vec<u8>, u64), pc: u64) {
        self.instr_cache
            .update_cache(instructions.1, instructions.0);
        // update view window of instructions:
        if pc > 8 + self.instr_cache.start_address + self.instr_cache.instructions.len() as u64 {
            self.user_start_addr = pc - 8;
        }
        if pc < self.instr_cache.start_address {
            self.user_start_addr = pc - 8;
        }

        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::initial(100.0).at_least(40.0).clip(false))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
            .column(Column::remainder())
            .min_scrolled_height(1.0);

        table
            .header(40.0, |mut header| {
                header.col(|ui| {
                    ui.strong("");
                });
                header.col(|ui| {
                    ui.strong("Address");
                });
                header.col(|ui| {
                    ui.strong("Instruction");
                });
                header.col(|ui| {
                    ui.strong("Mnemonic");
                });
            })
            .body(|body| {
                body.rows(text_height, self.instr_cache.disasm.len(), |mut row| {
                    let row_index = row.index();
                    let (instr_addr, addr_hex, instr_hex, instr_mnemonic) =
                        self.instr_cache.get_disasm(row_index);
                    if pc == instr_addr {
                        highlight_col(&mut row, "➡", addr_hex, instr_hex, instr_mnemonic);
                    } else {
                        row.col(|ui| {
                            ui.label("");
                        });
                        row.col(|ui| {
                            ui.label(addr_hex);
                        });
                        row.col(|ui| {
                            ui.label(instr_hex);
                        });
                        row.col(|ui| {
                            ui.label(instr_mnemonic);
                        });
                    }
                })
            });
    }

    pub fn get_start_addr(&self) -> u64 {
        self.user_start_addr
    }

    pub fn get_num_instr(&self) -> u64 {
        self.user_num_instr
    }
}

fn highlight_col(row: &mut TableRow<'_, '_>, s1: &str, s2: &str, s3: &str, s4: &str) {
    // TODO: change for white background
    let color = Color32::YELLOW;
    row.col(|ui| {
        ui.colored_label(color, s1);
    });
    row.col(|ui| {
        ui.colored_label(color, s2);
    });
    row.col(|ui| {
        ui.colored_label(color, s3);
    });
    row.col(|ui| {
        ui.colored_label(color, s4);
    });
}

/// Instruction Cache to optimizing rendering the instruction list
#[derive(Default)]
struct InstrCache {
    instructions: Vec<u8>,
    start_address: u64,
    /// (instr_addr, instr_addr_hex_str, instr_hex_str, instr_mnemonic_str)
    disasm: Vec<(u64, String, String, String)>,
}

impl InstrCache {
    fn update_cache(&mut self, start_addr: u64, new_instructions: &Vec<u8>) {
        // compare against cached instructions
        if new_instructions.len() == self.instructions.len()
            && zip(&self.instructions, new_instructions)
                .all(|(old_byte, new_byte)| *old_byte == *new_byte)
        {
            return;
        }
        // keep it for debuggin unnecessary cache updates
        println!("UI: Updating instruction cache");
        self.start_address = start_addr;
        self.instructions.resize(new_instructions.len(), 0);
        self.instructions.copy_from_slice(new_instructions);
        self.disasm = Vec::with_capacity(new_instructions.len());

        let instr_iter = InstrCacheIterator {
            instr_bytes: new_instructions,
            curr_addr: start_addr,
            curr_byte: 0,
        };
        for (instr_addr, instr) in instr_iter {
            let addr_hex = u64_hex4(instr_addr);
            let instr_hex = instr_hex(instr);
            let instr_mnemonic = disasm(instr, instr_addr);
            self.disasm
                .push((instr_addr, addr_hex, instr_hex, instr_mnemonic));
        }
    }

    fn get_disasm(&self, index: usize) -> (u64, &str, &str, &str) {
        let (instr_addr, ref addr_hex, ref instr_hex, ref instr_mnemonic) = self.disasm[index];
        (instr_addr, addr_hex, instr_hex, instr_mnemonic)
    }
}

struct InstrCacheIterator<'a> {
    instr_bytes: &'a Vec<u8>,
    curr_addr: u64,
    curr_byte: usize,
}

impl<'a> Iterator for InstrCacheIterator<'a> {
    type Item = (u64, u32); // (instruction_address, instruction_32b_or_16b)
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_byte >= self.instr_bytes.len() {
            return None;
        }
        if self.instr_is_32b() {
            if self.curr_byte + 4 >= self.instr_bytes.len() {
                return None;
            }
            let ret_val = (self.curr_addr, self.get_instr());
            self.curr_addr += 4;
            self.curr_byte += 4;
            Some(ret_val)
        } else {
            // 16b compressed instruction
            let ret_val = (self.curr_addr, self.get_rvc_instr());
            self.curr_addr += 2;
            self.curr_byte += 2;
            Some(ret_val)
        }
    }
}

impl<'a> InstrCacheIterator<'a> {
    fn get_instr(&self) -> u32 {
        // TODO: this might fail due to cut 32b instruction
        (self.instr_bytes[self.curr_byte] as u32)
            | (self.instr_bytes[self.curr_byte + 1] as u32) << 8
            | (self.instr_bytes[self.curr_byte + 2] as u32) << 16
            | (self.instr_bytes[self.curr_byte + 3] as u32) << 24
    }

    // 16-bit compressed
    fn get_rvc_instr(&self) -> u32 {
        (self.instr_bytes[self.curr_byte] as u32)
            | (self.instr_bytes[self.curr_byte + 1] as u32) << 8
    }

    fn instr_is_32b(&self) -> bool {
        !instr_is_rvc(self.instr_bytes[self.curr_byte] as u32)
    }
}
