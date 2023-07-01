pub struct LoadDemo {
    /// Is window open or not
    window_open: bool,
    demos: Vec<Demo>,
    selected_demo: usize,
}

impl Default for LoadDemo {
    fn default() -> LoadDemo {
        let mut demos = Vec::new();
        demos.push(Demo {
            name: "Hello world",
            descriptoin: "Bare metall program printing Hello World to UART",
            load_address: 0x0000000080000000,
            breakpoint: 0x0000000080000014,
            image: include_bytes!("../assets/test_bins/uart_hello_world.bin"),
        });
        // TODO: changes this:
        demos.push(Demo {
            name: "Hello world",
            descriptoin: "Bare metall program printing Hello World to UART",
            load_address: 0x0000000080000000,
            breakpoint: 0x0000000080000014,
            image: demos[0].image,
        });
        LoadDemo {
            window_open: true,
            demos,
            selected_demo: 0,
        }
    }
}

struct Demo {
    name: &'static str,
    descriptoin: &'static str,
    load_address: u64,
    breakpoint: u64,
    image: &'static [u8],
}

impl LoadDemo {
    pub fn open(&mut self) {
        self.window_open = true;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<&'static [u8]> {
        let mut loaded_demo = false;
        if self.window_open {
            let mut window_opened = self.window_open;
            egui::Window::new("Load demo")
                .open(&mut window_opened)
                .resizable(true)
                .default_width(500.0)
                .show(ctx, |ui| {
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
            Some(self.demos[self.selected_demo].image)
        } else {
            None
        }
    }
}
