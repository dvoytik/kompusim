use crate::sim::SimState;

pub enum StatusControlCmd {
    Run,
    Stop,
    Step,
    //Autostep
}

pub struct StatusControl {
    /// Is window open or not
    window_open: bool,
}

impl Default for StatusControl {
    fn default() -> Self {
        StatusControl { window_open: true }
    }
}

impl StatusControl {
    pub fn open(&mut self) {
        self.window_open = true;
    }

    pub fn show_if_opened(
        &mut self,
        ctx: &egui::Context,
        sim_state: SimState,
    ) -> Option<StatusControlCmd> {
        let mut command: Option<StatusControlCmd> = None;
        if self.window_open {
            let mut window_opened = self.window_open;
            egui::Window::new("Simulator Status/Control")
                .open(&mut window_opened)
                .resizable(true)
                .default_width(500.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Run").clicked() {
                            // TODO:
                            command = Some(StatusControlCmd::Run);
                        }
                        ui.add_enabled_ui(false, |ui| {
                            if ui.button("Stop").clicked() {
                                // TODO:
                            }
                        });
                        ui.add_enabled_ui(false, |ui| {
                            if ui.button("Step").clicked() {
                                self.window_open = false;
                            }
                        });
                    });
                    egui::Grid::new("load_demo_grid")
                        .num_columns(2)
                        //.min_col_width(600.0)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Simulator state: ");
                            ui.label(format!("{:?}", sim_state));
                            ui.end_row();
                            ui.label("RAM: ");
                            ui.label("TODO");
                            ui.end_row();
                            ui.label("Devices: ");
                            ui.label("TODO");
                            ui.end_row();
                        });
                });
            if self.window_open {
                self.window_open = window_opened;
            }
        }
        command
    }
}
