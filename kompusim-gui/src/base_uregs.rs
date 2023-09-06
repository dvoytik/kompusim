use kompusim::{
    rv64i_cpu::RV64IURegs,
    rv64i_disasm::{reg_hex, reg_idx2abi},
};

/// Base Unprivileged Integer Registers
pub struct BaseURegs {
    /// Is window open or not
    window_open: bool,
}

impl Default for BaseURegs {
    fn default() -> Self {
        Self { window_open: true }
    }
}

impl BaseURegs {
    pub fn open(&mut self) {
        self.window_open = true;
    }

    pub fn show_if_opened(&mut self, ui_ctx: &egui::Context, regs: &RV64IURegs, instr: u32) {
        if !self.window_open {
            return;
        }
        // TODO: chaching?
        let used_regs = kompusim::rv64i_disasm::disasm_get_used_regs(instr);

        let mut window_opened = self.window_open;
        egui::Window::new("Base Unprivileged Integer Registers")
            .open(&mut window_opened)
            .resizable(false)
            .default_width(500.0)
            .show(ui_ctx, |ui| {
                ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    egui::Grid::new("base_regs_grid0")
                        .num_columns(3)
                        //.min_col_width(600.0)
                        .striped(true)
                        .show(ui, |ui| {
                            for i in 0..=15 {
                                let reg_i = i as u8;
                                grid_row_reg(ui, regs, reg_i, reg_hi_color(reg_i, used_regs));
                            }
                        });
                    egui::Grid::new("base_regs_grid1")
                        .num_columns(3)
                        //.min_col_width(600.0)
                        .striped(true)
                        .show(ui, |ui| {
                            for i in 16..=31 {
                                let reg_i = i as u8;
                                grid_row_reg(ui, regs, reg_i, reg_hi_color(reg_i, used_regs));
                            }
                            ui.label("pc".to_string());
                            ui.label("");
                            ui.label(reg_hex(regs.pc));
                            ui.end_row();
                        });
                });
            });
        self.window_open = window_opened;
    }
}

/// input (read_reg_idx, read_reg_idx, write_reg_idx)
fn reg_hi_color(
    reg_idx: u8,
    in_out_regs: (Option<u8>, Option<u8>, Option<u8>),
) -> Option<egui::Color32> {
    if let Some(write_reg) = in_out_regs.2 {
        if write_reg == reg_idx {
            return Some(egui::Color32::RED); // TODO: configured color
        }
    }
    if let Some(read_reg0) = in_out_regs.0 {
        if read_reg0 == reg_idx {
            return Some(egui::Color32::GREEN); // TODO: configured color
        }
    }
    if let Some(read_reg1) = in_out_regs.1 {
        if read_reg1 == reg_idx {
            return Some(egui::Color32::GREEN); // TODO: configured color
        }
    }
    // egui::Color32::YELLOW;
    None
}

/// show a register in the grid
fn grid_row_reg(ui: &mut egui::Ui, regs: &RV64IURegs, ri: u8, hi_color: Option<egui::Color32>) {
    let reg_name = format!("x{ri}");
    let reg_abi_name = reg_idx2abi(ri);
    let reg_hex_val = reg_hex(regs.x[ri as usize]);
    if let Some(color) = hi_color {
        ui.colored_label(color, reg_name);
        ui.colored_label(color, reg_abi_name);
        ui.colored_label(color, reg_hex_val);
    } else {
        ui.label(reg_name);
        ui.label(reg_abi_name);
        ui.label(reg_hex_val);
    }
    ui.end_row();
}
