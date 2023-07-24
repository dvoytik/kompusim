use eframe::{self, glow::Context};
use egui::Modifiers;

use crate::{
    base_uregs::BaseURegs,
    console::Console,
    instr_decoder::InstrDecoder,
    instr_list::InstrList,
    load_demo::LoadDemo,
    sim::Simulator,
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
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Start simulator thread
        //thread
        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            let app: KompusimApp = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            set_all_fonts_size(&cc.egui_ctx, app.font_delta as f32 * 0.5);
            return app;
        }
        Default::default()
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Shortcuts
            let organize_windows_shortcut =
                egui::KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, egui::Key::O);
            if ui.input_mut(|i| i.consume_shortcut(&organize_windows_shortcut)) {
                ctx.memory_mut(|mem| mem.reset_areas());
            }

            let inc_fonts_shortcut =
                egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::PlusEquals);
            if ui.input_mut(|i| i.consume_shortcut(&inc_fonts_shortcut)) {
                increase_all_fonts(ctx, font_delta);
            }
            let dec_fonts_shortcut = egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::Minus);
            if ui.input_mut(|i| i.consume_shortcut(&dec_fonts_shortcut)) {
                decrease_all_fonts(ctx, font_delta);
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
                        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                        _frame.close();
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
                    // hack to make menus oneliners
                    ui.set_min_width(*font_delta as f32 * 10.0 + 150.0);
                    if ui
                        .add(
                            egui::Button::new("Increase font")
                                .shortcut_text(ui.ctx().format_shortcut(&inc_fonts_shortcut)),
                        )
                        .clicked()
                    {
                        increase_all_fonts(ctx, font_delta);
                        ui.close_menu();
                    }
                    if ui
                        .add(
                            egui::Button::new("Decrease font")
                                .shortcut_text(ui.ctx().format_shortcut(&dec_fonts_shortcut)),
                        )
                        .clicked()
                    {
                        decrease_all_fonts(ctx, font_delta);
                        ui.close_menu();
                    }
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

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::warn_if_debug_build(ui);
        });

        match status_control.show_if_opened(ctx, sim.get_state(), sim.get_num_exec_instr()) {
            None => {}
            Some(StatusControlCmd::Run) => sim.carry_on(),
            Some(StatusControlCmd::Stop) => {
                todo!()
            }
            Some(StatusControlCmd::Step) => {
                sim.step();
            }
        }
        base_uregs.show_if_opened(ctx, sim.get_regs());
        instr_list.show_if_opened(ctx, sim.disasm_at_pc());
        decode_instr.show(ctx);
        if let Some(demo_image) = load_demo.show_pick_demo(ctx) {
            sim.load_image(
                demo_image.load_address,
                demo_image.image,
                demo_image.breakpoint,
            )
        }
        console.show(ctx, sim.console_recv());

        egui::Window::new("Settings")
            .open(show_settings)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| ctx.settings_ui(ui));
            });
    }
}

fn increase_all_fonts(ctx: &egui::Context, font_delta: &mut i32) {
    if *font_delta <= 50 {
        *font_delta += 1;
        set_all_fonts_size(ctx, 0.5);
    }
}

fn decrease_all_fonts(ctx: &egui::Context, font_delta: &mut i32) {
    if *font_delta >= -5 {
        *font_delta -= 1;
        set_all_fonts_size(ctx, -0.5);
    }
}

fn set_all_fonts_size(ctx: &egui::Context, font_delta: f32) {
    let mut style: egui::Style = (*ctx.style()).clone();
    for (_, v) in style.text_styles.iter_mut() {
        v.size += font_delta;
    }
    ctx.set_style(style);
}
