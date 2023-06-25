use kompusim::rv64i_disasm::{disasm, u32_bin4};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct InstrDecoder {
    /// Is window open or not
    window_open: bool,

    instr_hex: String,
    #[serde(skip)]
    instr_disasm: String,
    #[serde(skip)]
    instr_binary: String,
}

impl Default for InstrDecoder {
    fn default() -> InstrDecoder {
        InstrDecoder {
            window_open: true,
            instr_hex: String::with_capacity(16),
            instr_disasm: String::new(),
            instr_binary: String::new(),
        }
    }
}

impl InstrDecoder {
    pub fn open(&mut self) {
        self.window_open = true;
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let mut open = self.window_open;
        egui::Window::new("Instruction decoder")
            .open(&mut open)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                self.show_window_content(ui);
            });
        self.window_open = open;
    }

    fn show_window_content(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("decode_instr_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Instruction");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.instr_hex).hint_text("instruction in hex"),
                );
                if response.changed() || self.instr_disasm.len() == 0 {
                    let instr = hex_to_u32(&self.instr_hex);
                    self.instr_disasm = disasm(instr, 0x0); // TODO: add address
                    self.instr_binary = u32_bin4(instr);
                }
                ui.end_row();
                ui.label("Binary");
                ui.vertical(|ui| {
                    //ui.label(RichText::new("").monospace());
                    ui.monospace("31   27   23   19   15   11   7    3   ");
                    ui.monospace(&self.instr_binary);
                });
                ui.end_row();
                ui.label("Assembly");
                ui.label(&self.instr_disasm);
                ui.end_row();
            });
    }
}

/// Convert hex str (e.g, "0x9393") to u32
fn hex_to_u32(hex_str: &str) -> u32 {
    u32::from_str_radix(hex_str.trim_start_matches("0x"), 16).unwrap_or_default()
}
