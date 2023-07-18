use crate::sim::DisasmInstructionLine;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct InstrList {
    /// Is window open or not
    open: bool,
    font_size: usize,
}

impl Default for InstrList {
    fn default() -> InstrList {
        InstrList {
            open: true,
            font_size: 0,
        }
    }
}

impl InstrList {
    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn show_if_opened(
        &mut self,
        ctx: &egui::Context,
        instructions: &Vec<DisasmInstructionLine>,
    ) {
        let mut open = self.open;
        egui::Window::new("Instructions")
            .open(&mut open)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.show_table(ui, instructions));
            });
        self.open = open;
    }

    fn show_table(&self, ui: &mut egui::Ui, instructions: &Vec<DisasmInstructionLine>) {
        use egui_extras::{Column, TableBuilder};

        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::initial(100.0).at_least(40.0).clip(false))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
            .column(Column::remainder())
            .min_scrolled_height(1.0);

        table
            .header(40.0, |mut header| {
                header.col(|ui| {
                    ui.strong("");
                });
                header.col(|ui| {
                    ui.strong("Address");
                });
                header.col(|ui| {
                    ui.strong("Instruction");
                });
                header.col(|ui| {
                    ui.strong("Mnemonic");
                });
            })
            .body(|body| {
                // for instruction_row in instructions {
                //     println!("hi")
                //     body.row()
                // }
                body.rows(text_height, instructions.len(), |row_index, mut row| {
                    let mark = instructions[row_index].0;
                    let addr_hex = &instructions[row_index].1;
                    let instr_hex = &instructions[row_index].2;
                    let instr_mnemonic = &instructions[row_index].3;
                    row.col(|ui| {
                        ui.label(mark.unwrap_or(""));
                    });
                    row.col(|ui| {
                        ui.label(addr_hex);
                    });
                    row.col(|ui| {
                        ui.label(instr_hex);
                    });
                    row.col(|ui| {
                        ui.label(instr_mnemonic);
                    });
                })
            });
    }
}
