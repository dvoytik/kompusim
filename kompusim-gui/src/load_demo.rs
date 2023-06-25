pub struct LoadDemo {
    /// Is window open or not
    window_open: bool,
    test_bin1: &'static [u8],
}

impl Default for LoadDemo {
    fn default() -> LoadDemo {
        let bare_metal_uart_hello_world =
            include_bytes!("../assets/test_bins/uart_hello_world.bin");
        LoadDemo {
            window_open: true,
            test_bin1: bare_metal_uart_hello_world,
        }
    }
}

impl LoadDemo {
    pub fn open(&mut self) {
        self.window_open = true;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<&'static [u8]> {
        let mut chose_demo1 = false;
        if self.window_open {
            let mut window_opened = self.window_open;
            egui::Window::new("Load demo")
                .open(&mut window_opened)
                .resizable(true)
                .default_width(400.0)
                .show(ctx, |ui| {
                    egui::Grid::new("load_demo_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Bare metal hello world with UART");
                            if ui.button("Load").clicked() {
                                self.window_open = false;
                                chose_demo1 = true;
                            }
                            ui.end_row();
                            ui.label("Linux kernel (unimplemented)");
                            if ui.button("Load").clicked() {
                                self.window_open = false;
                            }
                            ui.end_row();
                        });
                });
            if self.window_open {
                self.window_open = window_opened;
            }
        }
        if chose_demo1 {
            Some(self.test_bin1)
        } else {
            None
        }
    }
}
