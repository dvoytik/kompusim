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

    pub fn show_if_opened(&mut self, ctx: &egui::Context, regs: &RV64IURegs) {
        if self.window_open {
            let mut window_opened = self.window_open;
            egui::Window::new("Base Unprivileged Integer Registers")
                .open(&mut window_opened)
                .resizable(false)
                .default_width(500.0)
                .show(ctx, |ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        egui::Grid::new("base_regs_grid0")
                            .num_columns(3)
                            //.min_col_width(600.0)
                            .striped(true)
                            .show(ui, |ui| {
                                for i in 0..=15 {
                                    grid_row_reg(ui, regs, i as u8);
                                }
                            });
                        egui::Grid::new("base_regs_grid1")
                            .num_columns(3)
                            //.min_col_width(600.0)
                            .striped(true)
                            .show(ui, |ui| {
                                for i in 16..=31 {
                                    grid_row_reg(ui, regs, i as u8);
                                }
                                ui.label(format!("pc"));
                                ui.label("");
                                ui.label(reg_hex(regs.pc));
                                ui.end_row();
                            });
                    });
                });
            if self.window_open {
                self.window_open = window_opened;
            }
        }
    }
}

/// show a register in the grid
fn grid_row_reg(ui: &mut egui::Ui, regs: &RV64IURegs, ri: u8) {
    ui.label(format!("x{ri}"));
    ui.label(reg_idx2abi(ri));
    ui.label(reg_hex(regs.x[ri as usize]));
    ui.end_row();
}
