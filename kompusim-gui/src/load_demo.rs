pub struct LoadDemo {
    /// Is window open or not
    pub window_open: bool,
    demos: Vec<DemoImage>,
    selected_demo: usize,
}

impl Default for LoadDemo {
    fn default() -> LoadDemo {
        let demos = vec![
            DemoImage {
                name: "Hello world",
                descriptoin: "Bare metall program printing Hello World to UART",
                load_address: 0x0000000080000000,
                breakpoint: 0x0000000080000014,
                image: include_bytes!("../assets/test_bins/uart_hello_world.bin"),
            }, // TODO: changes this:
            DemoImage {
                name: "Empty binary",
                descriptoin: "One zero byte",
                load_address: 0x0000000080000000,
                breakpoint: 0x0000000080000014,
                image: b"\x00",
            },
        ];
        LoadDemo {
            window_open: true,
            demos,
            selected_demo: 0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct DemoImage {
    name: &'static str,
    descriptoin: &'static str,
    pub load_address: u64,
    pub breakpoint: u64,
    pub image: &'static [u8],
}

impl LoadDemo {
    pub fn open(&mut self) {
        self.window_open = true;
    }

    pub fn show_pick_demo(&mut self, ui_ctx: &egui::Context) -> Option<DemoImage> {
        let mut loaded_demo = false;
        if self.window_open {
            let mut window_opened = self.window_open;
            egui::Window::new("Load demo")
                .open(&mut window_opened)
                .resizable(true)
                .default_width(500.0)
                .show(ui_ctx, |ui| {
                    egui::Grid::new("load_demo_grid")
                        .num_columns(1)
                        .min_col_width(600.0)
                        .striped(true)
                        .show(ui, |ui| {
                            egui::ComboBox::from_label("").width(400.0).show_index(
                                ui,
                                &mut self.selected_demo,
                                self.demos.len(),
                                |i| self.demos[i].name,
                            );
                            ui.end_row();
                            ui.label(self.demos[self.selected_demo].descriptoin);
                            ui.end_row();
                            ui.label(format!(
                                "Load address: 0x{:x}",
                                self.demos[self.selected_demo].load_address
                            ));
                            ui.end_row();
                            ui.label(format!(
                                "Breakpoint: 0x{:x}",
                                self.demos[self.selected_demo].breakpoint
                            ));
                            ui.end_row();
                            if ui.button("Load").clicked() {
                                self.window_open = false;
                                loaded_demo = true;
                            }
                            ui.end_row();
                        });
                });
            if self.window_open {
                self.window_open = window_opened;
            }
        }
        if loaded_demo {
            Some(self.demos[self.selected_demo])
        } else {
            None
        }
    }
}
