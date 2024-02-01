use eframe::{self, glow::Context};
use egui::Modifiers;

use crate::{
    base_uregs::BaseURegs,
    cmdline::{parse_size_with_suffix, CmdLCommand},
    console::Console,
    instr_decoder::InstrDecoder,
    instr_list::InstrList,
    load_demo::LoadDemo,
    sim::{Simulator, DEFAULT_MEM_SZ},
    status_control::{StatusControl, StatusControlCmd},
};

/// Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct KompusimApp {
    /// delta (font_delta * 0.5) for the default font size for all text styles
    font_delta: i32,
    show_settings: bool,
    #[serde(skip)]
    status_control: StatusControl,
    #[serde(skip)]
    base_uregs: BaseURegs,
    instr_list: InstrList,
    #[serde(skip)]
    decode_instr: InstrDecoder,
    console: Console,
    #[serde(skip)] // this how you opt-out of serialization of a member
    load_demo: LoadDemo,
    #[serde(skip)]
    sim: Simulator,
}

impl Default for KompusimApp {
    fn default() -> Self {
        Self {
            show_settings: false,
            font_delta: 0,
            status_control: StatusControl::default(),
            instr_list: InstrList::default(),
            base_uregs: BaseURegs::default(),
            decode_instr: InstrDecoder::default(),
            load_demo: LoadDemo::default(),
            console: Console::default(),
            sim: Simulator::new(),
        }
    }
}

impl KompusimApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, cmdl_args: Option<CmdLCommand>) -> Self {
        // Start simulator thread
        //thread
        // Load previous app state (if any).
        let mut app = if let Some(storage) = cc.storage {
            let app: KompusimApp = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            app
        } else {
            Default::default()
        };

        if let Some(cmdl_cmd) = cmdl_args {
            let CmdLCommand::Exec {
                load_addr,
                bin,
                ram,
                breakpoint,
                ..
            } = cmdl_cmd;
            if let Some(ref ram) = ram {
                if let Some(ram_sz) = parse_size_with_suffix(ram) {
                    app.sim.set_ram_sz(ram_sz);
                } else {
                    eprintln!(
                        "Ram size is wrong format. Using default size: {}",
                        DEFAULT_MEM_SZ
                    );
                }
            }
            if let Some(breakpoint) = breakpoint {
                let breakpoint = u64::from_str_radix(breakpoint.trim_start_matches("0x"), 16)
                    .expect("Breakpoint address is wrong format");
                app.sim.add_breakpoint(breakpoint);
            }
            println!("Got command line: execute: execute {bin:?} @ {load_addr}, RAM: {ram:?}");
            let load_addr = u64::from_str_radix(load_addr.trim_start_matches("0x"), 16)
                .expect("Load address is wrong format");
            app.sim.load_bin_file(load_addr, bin);
            // Do not show windows that doesn't make sense to show:
            app.load_demo.window_open = false;
        }
        app
    }
}

impl eframe::App for KompusimApp {
    /// Called by the frame work to save state periodically and before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// called before shutdown
    fn on_exit(&mut self, _gl: Option<&Context>) {
        self.sim.stop();
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ui_ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            show_settings,
            font_delta,
            status_control,
            base_uregs,
            instr_list,
            decode_instr,
            load_demo,
            console,
            sim,
        } = self;

        // The top panel is for the menu bar:
        egui::TopBottomPanel::top("top_panel").show(ui_ctx, |ui| {
            // Shortcuts
            let organize_windows_shortcut =
                egui::KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, egui::Key::O);
            if ui.input_mut(|i| i.consume_shortcut(&organize_windows_shortcut)) {
                ui_ctx.memory_mut(|mem| mem.reset_areas());
            }

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    // hack to make menus oneliners
                    ui.set_min_width(*font_delta as f32 * 10.0 + 150.0);
                    ui.add_enabled_ui(false, |ui| {
                        if ui.button("Load binary (unimplemented)").clicked() {
                            ui.close_menu();
                        }
                    });
                    if ui.button("Load demo...").clicked() {
                        load_demo.open();
                        ui.close_menu();
                    }
                    ui.add_enabled_ui(false, |ui| {
                        if ui.button("Settings").clicked() {
                            *show_settings = true;
                            ui.close_menu();
                        }
                    });
                    if ui.button("Quit").clicked() {
                        ui_ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Run", |ui| {
                    // hack to make menus oneliners
                    ui.set_min_width(*font_delta as f32 * 10.0 + 150.0);
                    ui.add_enabled_ui(true, |ui| {
                        if ui.button("Run/Continue").clicked() {
                            sim.carry_on();
                            ui.close_menu();
                        }
                    });
                    ui.add_enabled_ui(false, |ui| {
                        if ui.button("Step (unimplemented)").clicked() {
                            ui.close_menu();
                        }
                    });
                });
                ui.menu_button("Windows", |ui| {
                    // hack to make menus oneliners
                    ui.set_min_width(*font_delta as f32 * 10.0 + 150.0);
                    if ui.button("Status/Control").clicked() {
                        status_control.open();
                        ui.close_menu();
                    }
                    if ui.button("Instruction list").clicked() {
                        instr_list.open();
                        ui.close_menu();
                    }
                    if ui.button("Registers (base unpriv)").clicked() {
                        base_uregs.open();
                        ui.close_menu();
                    }
                    if ui.button("Instruction decoder").clicked() {
                        decode_instr.open();
                        ui.close_menu();
                    }
                    if ui.button("Console").clicked() {
                        console.open();
                        ui.close_menu();
                    }
                    ui.add_enabled_ui(false, |ui| {
                        if ui.button("Memory (unimplemented)").clicked() {
                            ui.close_menu();
                        }
                    });
                });
                ui.menu_button("View", |ui| {
                    if ui
                        .add(
                            egui::Button::new("Organize windows").shortcut_text(
                                ui.ctx().format_shortcut(&organize_windows_shortcut),
                            ),
                        )
                        .clicked()
                    {
                        ui.ctx().memory_mut(|mem| mem.reset_areas());
                        ui.close_menu();
                    }
                    egui::gui_zoom::zoom_menu_buttons(ui);
                });
                ui.menu_button("Help", |ui| {
                    ui.add_enabled_ui(false, |ui| {
                        if ui.button("About").clicked() {
                            ui.close_menu();
                        }
                    });
                });
            });
        });

        egui::CentralPanel::default().show(ui_ctx, |ui| {
            egui::warn_if_debug_build(ui);
        });

        match status_control.show_if_opened(ui_ctx, sim.get_state(), sim.get_num_exec_instr()) {
            None => {}
            Some(StatusControlCmd::Run) => sim.carry_on(),
            Some(StatusControlCmd::Stop) => {
                todo!()
            }
            Some(StatusControlCmd::Step) => {
                sim.step();
            }
        }
        let cur_instr = sim.get_cur_instr();
        base_uregs.show_if_opened(ui_ctx, sim.get_regs(), cur_instr);
        let pc = sim.get_regs().pc;
        instr_list.show_if_opened(
            ui_ctx,
            sim.get_instructions(instr_list.get_start_addr(), instr_list.get_num_instr()),
            pc,
        );
        decode_instr.show_if_opened(ui_ctx, sim.get_regs().pc, cur_instr);

        if let Some(demo_image) = load_demo.show_pick_demo(ui_ctx) {
            sim.load_image(
                demo_image.load_address,
                demo_image.image,
                demo_image.breakpoint,
            )
        }

        console.show(ui_ctx, sim.console_recv());

        egui::Window::new("Settings")
            .open(show_settings)
            .show(ui_ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| ui_ctx.settings_ui(ui));
            });
    }
}
