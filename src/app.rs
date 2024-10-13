use egui::{scroll_area::ScrollAreaOutput, Color32};
use egui_extras::{Column, TableBuilder};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Carelog {
    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    #[serde(skip)]
    state: AppState,
}
#[derive(Default)]
pub struct AppState {
    id: u64,
    first_name: String,
    last_name: String,
    phone_number: String,
    cb_patient_relation: bool,
    relative_name: String,
}

// ! TODO - Make the Screens independent
// pub enum Screen {
//     NewPatientCreation,
//     FindPatientHistory,
// }

impl Default for Carelog {
    fn default() -> Self {
        Self {
            value: 2.7,
            state: AppState::default(),
        }
    }
}

impl Carelog {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for Carelog {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                    credits(ui);
                    egui::widgets::global_theme_preference_buttons(ui);
                });
                egui::warn_if_debug_build(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("New Patient Information");
            ui.add_space(20.0);

            ui.checkbox(
                &mut self.state.cb_patient_relation,
                "Relative of an existing patient",
            );

            if self.state.cb_patient_relation {
                egui::Grid::new("cb_patient_relation_grid")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::strong("Relative's Name".into()));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.state.relative_name)
                                .hint_text("Sujit Bhor"),
                        );
                        ui.end_row();
                    });
            }
            ui.add_space(10.0);

            egui::Grid::new("patient_form_grid")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label(egui::RichText::strong("Patient ID".into()));
                    ui.label(
                        egui::RichText::strong("1".into())
                            .color(Color32::YELLOW)
                            .background_color(Color32::BLACK)
                            .raised(),
                    );
                    ui.end_row();

                    ui.label(egui::RichText::strong("First Name".into()));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.state.first_name).hint_text("Suyog"),
                    );
                    ui.end_row();

                    ui.label(egui::RichText::strong("Last Name".into()));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.state.last_name).hint_text("Diggikar"),
                    );
                    ui.end_row();

                    ui.label(egui::RichText::strong("Mobile Number".into()));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.state.phone_number).hint_text("100"),
                    );
                    ui.end_row();
                });

            ui.add_space(20.0);
            ui.button("Create patient").highlight();
        });
    }
}

fn credits(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label(" © 2024 Sakshat Shinde");
    });
}

fn display_related_patients(ui: &mut egui::Ui) -> ScrollAreaOutput<()> {
    let related_patient_info = TableBuilder::new(ui)
        .column(Column::auto().resizable(true))
        .column(Column::remainder())
        .striped(true)
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.heading("Patient Name");
            });
            header.col(|ui| {
                ui.heading("Patient Id");
            });
        })
        .body(|mut body| {
            body.row(5.0, |mut row| {
                row.col(|ui| {
                    ui.label("Suyog Diggikar");
                });
                row.col(|ui| {
                    ui.label("1");
                });
            });
            body.row(5.0, |mut row| {
                row.col(|ui| {
                    ui.label("Sujit Bhor");
                });
                row.col(|ui| {
                    ui.label("2");
                });
            });
        });

    return related_patient_info;
}
