use eframe::egui::scroll_area::ScrollAreaOutput;
use egui_extras::{Column, TableBuilder};
use egui_notify::Toasts;
use rusqlite::Connection;
use std::time::Duration;

use crate::{create_db, insert_new_patient, DB_URL};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Carelog {
    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    #[serde(skip)]
    state: AppState,
    #[serde(skip)]
    current_screen: Screen,
}

pub struct AppState {
    conn: Connection,
    first_name: String,
    last_name: String,
    phone_number: String,
    date_of_birth: chrono::NaiveDate,
    address: String,
    cb_patient_relation: bool,
    relative_name: String,
    toasts: Toasts,
}

impl Default for AppState {
    fn default() -> Self {
        let conn = Connection::open(DB_URL).expect("Error with the DB");
        let _ = create_db(&conn).expect("Error with the DB");

        Self {
            conn: conn,
            first_name: Default::default(),
            last_name: Default::default(),
            phone_number: Default::default(),
            date_of_birth: chrono::Local::now().date_naive(),
            address: Default::default(),
            cb_patient_relation: Default::default(),
            relative_name: Default::default(),
            toasts: Default::default(),
        }
    }
}
// ! TODO - Make the Screens independent
#[derive(PartialEq)]
pub enum Screen {
    NewPatientCreation,
    FindPatientHistory,
}

impl Screen {
    fn title(&self) -> &'static str {
        match self {
            Screen::NewPatientCreation => "➕ New Patient",
            Screen::FindPatientHistory => "🔍 Find Patient",
        }
    }
}

impl Default for Carelog {
    fn default() -> Self {
        Self {
            value: 2.7,
            state: AppState {
                ..Default::default()
            },
            current_screen: Screen::NewPatientCreation,
        }
    }
}

impl Carelog {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_zoom_factor(1.2);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
    fn render_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                // Navigation buttons
                let mut nav_button = |ui: &mut egui::Ui, screen: Screen| {
                    let selected = self.current_screen == screen;
                    let response =
                        ui.selectable_label(selected, screen.title())
                            .on_hover_text(match screen {
                                Screen::NewPatientCreation => "Create a new patient record",
                                Screen::FindPatientHistory => "Search and view patient records",
                            });

                    if response.clicked() {
                        self.current_screen = screen;
                    }
                };

                // Scroll area for navigation items
                egui::ScrollArea::vertical().show(ui, |ui| {
                    nav_button(ui, Screen::NewPatientCreation);
                    nav_button(ui, Screen::FindPatientHistory);
                });

                // Version information at the bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("Carelog ");
                        ui.colored_label(ui.visuals().weak_text_color(), "Alpha");
                    });
                });
            });
    }

    fn render_new_patient(&mut self, ui: &mut egui::Ui) {
        ui.heading("New Patient Information");
        ui.add_space(20.0);
        eframe::egui::Grid::new("patient_form_grid")
            .num_columns(2)
            .show(ui, |ui| {
                egui::Grid::new("patient_form_grid")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::strong("Patient ID".into()));

                        // Retrieve the ID from DB to be used
                        let id = &mut self.state.conn.last_insert_rowid().clone().to_string();

                        ui.label(egui::RichText::strong(id.into()).raised());
                        ui.end_row();
                        // ------------
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::strong("First Name".into()));
                            ui.label(egui::RichText::new("*").color(egui::Color32::RED));
                        });
                        ui.add(
                            egui::TextEdit::singleline(&mut self.state.first_name)
                                .hint_text("Suyog"),
                        );
                        ui.end_row();
                        // ------------
                        ui.label(egui::RichText::strong("Last Name".into()));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.state.last_name)
                                .hint_text("Diggikar"),
                        );
                        ui.end_row();
                        // ------------
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::strong("Mobile Number".into()));
                            ui.label(egui::RichText::new("*").color(egui::Color32::RED));
                        });
                        ui.add(
                            egui::TextEdit::singleline(&mut self.state.phone_number)
                                .hint_text("100"),
                        );
                        ui.end_row();
                        // -----------
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::strong("Date of birth".into()));
                        });
                        ui.add(egui_extras::DatePickerButton::new(
                            &mut self.state.date_of_birth,
                        ));
                        ui.end_row();
                        // -----------
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::strong("Address".into()));
                        });
                        ui.add(
                            egui::TextEdit::multiline(&mut self.state.address).hint_text(
                                "Harshal Residency, Near Gawade Petrol Pump, Chinchwad - 411019",
                            ),
                        );
                        ui.end_row();
                    });
            });

        ui.checkbox(
            &mut self.state.cb_patient_relation,
            "Relative of an existing patient",
        );

        if self.state.cb_patient_relation {
            ui.add_space(10.0);
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

        ui.add_space(20.0);

        if ui.button("Create patient").highlight().clicked()
            && !self.state.first_name.is_empty()
            && !self.state.phone_number.is_empty()
        {
            insert_new_patient(
                &self.state.conn,
                &self.state.first_name,
                &self.state.last_name,
                &self.state.phone_number,
                &self.state.date_of_birth,
                &self.state.address,
            )
            .unwrap_or_else(|_| {
                self.state
                    .toasts
                    .error("Failed to create a new patient")
                    .duration(Some(Duration::from_secs(7)));
            });

            self.state
                .toasts
                .success("Created a new patient")
                .duration(Some(Duration::from_secs(7)));
        };
    }

    // todo render patient search
    fn render_find_patient(&mut self, ui: &mut egui::Ui) {
        ui.heading("Find Patient History");
        ui.add_space(20.0);

        // Add your patient search functionality here
        ui.horizontal(|ui| {
            ui.label("Search: ");
            let mut search_text = String::new();
            ui.text_edit_singleline(&mut search_text);
            if ui.button("Search").clicked() {
                // Implement search functionality
                todo!()
            }
        });

        // Display search results
        display_related_patients(ui);
    }
}

impl eframe::App for Carelog {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Toast Init
        self.state.toasts.show(ctx);
        self.render_sidebar(ctx);

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
            egui::Frame::none().show(ui, |ui| match self.current_screen {
                Screen::NewPatientCreation => self.render_new_patient(ui),
                Screen::FindPatientHistory => self.render_find_patient(ui),
            });
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
