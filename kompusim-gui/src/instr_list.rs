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
    pub fn show(&mut self, ctx: &egui::Context) {
        let mut open = self.open;
        egui::Window::new("Instructions")
            .open(&mut open)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.show_table(ui));
            });
        self.open = open;
    }

    fn show_table(&self, ui: &mut egui::Ui) {
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
                    ui.strong("Instructions");
                });
                header.col(|ui| {
                    ui.strong("Comment");
                });
            })
            .body(|body| {
                body.rows(text_height, 100, |row_index, mut row| {
                    row.col(|ui| {
                        ui.label(row_index.to_string());
                    });
                    row.col(|ui| {
                        ui.label(text_sample(row_index));
                    });
                    row.col(|ui| {
                        ui.label(text_sample(row_index));
                    });
                    row.col(|ui| {
                        ui.add(egui::Label::new("Thousands of rows of even height").wrap(false));
                    });
                })
            });
    }
}

fn text_sample(row_index: usize) -> String {
    format!("txt {row_index}")
}
