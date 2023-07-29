use kompusim::rv64i_disasm::{disasm, u32_bin4, u32_hex4};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct InstrDecoder {
    /// Is window open or not
    window_open: bool,

    #[serde(skip)]
    instr_disasm: String,
    #[serde(skip)]
    instr_binary: String,
}

impl Default for InstrDecoder {
    fn default() -> InstrDecoder {
        InstrDecoder {
            window_open: true,
            instr_disasm: String::new(),
            instr_binary: String::new(),
        }
    }
}

impl InstrDecoder {
    pub fn open(&mut self) {
        self.window_open = true;
    }

    pub fn show_if_opened(&mut self, ctx: &egui::Context, address: u64, instruction: u32) {
        let mut open = self.window_open;
        egui::Window::new("Instruction decoder")
            .open(&mut open)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                self.show_window_content(ui, address, instruction);
            });
        self.window_open = open;
    }

    fn show_window_content(&mut self, ui: &mut egui::Ui, address: u64, instruction: u32) {
        egui::Grid::new("decode_instr_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Instruction");
                let mut instr_hex = u32_hex4(instruction);
                ui.add(egui::TextEdit::singleline(&mut instr_hex));
                self.instr_disasm = disasm(instruction, address);
                self.instr_binary = u32_bin4(instruction);
                ui.end_row();
                ui.label("Binary");
                ui.vertical(|ui| {
                    //ui.label(RichText::new("").monospace());
                    ui.monospace("31   27   23   19   15   11   7    3   ");
                    ui.monospace(&self.instr_binary);
                });
                ui.end_row();
                ui.label("Assembly");
                ui.monospace(&self.instr_disasm);
                ui.end_row();
            });
    }
}
