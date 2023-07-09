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

    pub fn show_if_opened(&mut self, ctx: &egui::Context) {
        if self.window_open {
            let mut window_opened = self.window_open;
            egui::Window::new("Base Unprivileged Integer Registers")
                .open(&mut window_opened)
                .resizable(false)
                .default_width(500.0)
                .show(ctx, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        egui::Grid::new("base_regs_grid0")
                            .num_columns(3)
                            //.min_col_width(600.0)
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("x1");
                                ui.label("ra");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x2");
                                ui.label("sp");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x3");
                                ui.label("gp");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x4");
                                ui.label("tp");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x5");
                                ui.label("t0");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x6");
                                ui.label("t1");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x7");
                                ui.label("t2");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x8");
                                ui.label("s0");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x9");
                                ui.label("s1");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x10");
                                ui.label("a0");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x11");
                                ui.label("a1");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x12");
                                ui.label("a2");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x13");
                                ui.label("a3");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x14");
                                ui.label("a4");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x15");
                                ui.label("a5");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();

                                ui.label("x16");
                                ui.label("a6");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();
                            });
                        egui::Grid::new("base_regs_grid1")
                            .num_columns(3)
                            //.min_col_width(600.0)
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("x17");
                                ui.label("a7");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();
                                ui.label("x18");
                                ui.label("s2");
                                ui.label("0000_0000_0000_0000");
                                ui.end_row();
                                ui.label("x19");
                                ui.label("s3");
                                ui.label("0000_0000_0000_0000");
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
