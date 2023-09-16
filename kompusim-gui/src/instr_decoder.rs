use kompusim::rv64i_disasm::{
    disasm, disasm_operation_name, disasm_pseudo_code, u32_bin4, u32_hex4, u64_hex4,
};

pub struct InstrDecoder {
    /// Is window open or not
    window_open: bool,

    cached_address: u64,
    cached_address_hex: String,
    cached_instruction: u32,
    cached_instr_hex: String,
    cached_instr_disasm: String,
    cached_instr_binary: String,
    cached_operation_name: String,
    cached_pseudo_code: String,
}

impl Default for InstrDecoder {
    fn default() -> InstrDecoder {
        InstrDecoder {
            window_open: true,
            cached_address: 0,
            cached_address_hex: String::new(),
            cached_instruction: 0,
            cached_instr_hex: String::new(),
            cached_instr_disasm: String::new(),
            cached_instr_binary: String::new(),
            cached_operation_name: String::new(),
            cached_pseudo_code: String::new(),
        }
    }
}

impl InstrDecoder {
    pub fn open(&mut self) {
        self.window_open = true;
    }

    pub fn show_if_opened(&mut self, ui_ctx: &egui::Context, address: u64, instruction: u32) {
        let mut open = self.window_open;
        egui::Window::new("Instruction decoder")
            .open(&mut open)
            .resizable(true)
            .default_width(400.0)
            .show(ui_ctx, |ui| {
                self.show_window_content(ui, address, instruction);
            });
        self.window_open = open;
    }

    fn show_window_content(&mut self, ui: &mut egui::Ui, address: u64, instruction: u32) {
        if address != self.cached_address || instruction != self.cached_instruction {
            self.cached_address = address;
            self.cached_address_hex = u64_hex4(address);
            self.cached_instruction = instruction;
            self.cached_instr_hex = u32_hex4(instruction);
            self.cached_instr_disasm = disasm(instruction, address);
            self.cached_instr_binary = u32_bin4(instruction);
            self.cached_operation_name = disasm_operation_name(instruction);
            self.cached_pseudo_code = disasm_pseudo_code(instruction, address);
        }
        egui::Grid::new("decode_instr_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Instruction");
                ui.add(egui::TextEdit::singleline(&mut self.cached_instr_hex));
                ui.end_row();
                ui.label("Its address");
                ui.add(egui::TextEdit::singleline(&mut self.cached_address_hex));
                ui.end_row();
                ui.label("Binary");
                ui.vertical(|ui| {
                    //ui.label(RichText::new("").monospace());
                    ui.monospace("31   27   23   19   15   11   7    3   ");
                    ui.monospace(&self.cached_instr_binary);
                });
                ui.end_row();
                ui.label("Assembly");
                ui.monospace(&self.cached_instr_disasm);
                ui.end_row();
                ui.label("Operation");
                ui.monospace(&self.cached_operation_name);
                ui.end_row();
                ui.label("Pseudo code");
                ui.monospace(&self.cached_pseudo_code);
                ui.end_row();
            });
    }
}
