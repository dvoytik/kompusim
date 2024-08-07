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
        ui_ctx: &egui::Context,
        sim_state: SimState,
        num_exec_instr: u64,
    ) -> Option<StatusControlCmd> {
        if !self.window_open {
            return None;
        }
        let mut command: Option<StatusControlCmd> = None;
        let mut window_opened = self.window_open;
        egui::Window::new("Simulator Status/Control")
            .open(&mut window_opened)
            .resizable(true)
            .default_width(500.0)
            .show(ui_ctx, |ui| {
                ui.horizontal(|ui| {
                    let (run_btn_en, stop_btn_en, step_btn_en) = match sim_state {
                        SimState::Initializing => (false, false, false),
                        SimState::InitializedReady => (true, false, true),
                        SimState::Running => (false, true, true),
                        SimState::Stopped => (true, false, true),
                        SimState::StoppedBreakpoint => (true, false, true),
                    };
                    ui.add_enabled_ui(run_btn_en, |ui| {
                        if ui.button("Run").clicked() {
                            command = Some(StatusControlCmd::Run);
                        }
                    });
                    ui.add_enabled_ui(stop_btn_en, |ui| {
                        if ui.button("Stop").clicked() {
                            command = Some(StatusControlCmd::Stop);
                        }
                    });
                    ui.add_enabled_ui(step_btn_en, |ui| {
                        if ui.button("Step").clicked() {
                            command = Some(StatusControlCmd::Step);
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
                        ui.label("Executed instructons: ");
                        ui.label(format!("{num_exec_instr}"));
                        ui.end_row();
                        ui.label("Devices: ");
                        ui.label("TODO");
                        ui.end_row();
                    });
            });
        self.window_open = window_opened;
        command
    }
}
