use eframe::egui::scroll_area::ScrollAreaOutput;
use eframe::egui::{Align, Layout};
use egui_extras::{Column, TableBuilder};
use egui_notify::Toasts;
use rusqlite::Connection;
use std::time::Duration;

use crate::{
    create_db, download_licenses_data, extract_number_from_brackets,
    helper_avaliable_patients_in_db, insert_new_patient, insert_patient_data, is_license_valid,
    DB_URL,
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Carelog {
    #[serde(skip)]
    state: AppState,
    #[serde(skip)]
    current_screen: Screen,
}

pub struct AppState {
    machine_uid: String,
    id: u32,
    conn: Connection,
    search_text: String,
    first_name: String,
    last_name: String,
    phone_number: String,
    date_of_birth: chrono::NaiveDate,
    visit_date: chrono::NaiveDate,
    address: String,
    cb_patient_relation: bool,
    relative_name: String,
    current_patient: String,
    diagnosis_overview: String,
    detailed_notes: String,
    medical_test_info: String,
    toasts: Toasts,
    is_license_valid: bool,
    customer: String,
}

impl Default for AppState {
    fn default() -> Self {
        let conn = Connection::open(DB_URL).expect("Failed to connect DB");
        let _ = create_db(&conn).expect("Failed to create or initialize the DB");

        let m_id = machine_uid::get().unwrap();

        let license_data = download_licenses_data();
        let (valid, licensee) = is_license_valid(m_id.as_str(), license_data.as_str());

        Self {
            machine_uid: m_id,
            id: Default::default(),
            conn: conn,
            first_name: Default::default(),
            last_name: Default::default(),
            phone_number: Default::default(),
            date_of_birth: chrono::Local::now().date_naive(),
            visit_date: chrono::Local::now().date_naive(),
            address: Default::default(),
            cb_patient_relation: Default::default(),
            relative_name: Default::default(),
            toasts: Default::default(),
            search_text: Default::default(),
            current_patient: Default::default(),
            diagnosis_overview: Default::default(),
            detailed_notes: Default::default(),
            medical_test_info: Default::default(),
            is_license_valid: valid,
            customer: licensee,
        }
    }
}

#[derive(PartialEq)]
pub enum Screen {
    NewPatientCreation,
    FindPatientHistory,
    NewCase,
    About,
}

impl Screen {
    fn title(&self) -> &'static str {
        match self {
            Screen::NewPatientCreation => "➕ New Patient",
            Screen::FindPatientHistory => "🔍 New Case",
            Screen::NewCase => "🔍 NewCase",
            Screen::About => "💊 About",
        }
    }
}

impl Default for Carelog {
    fn default() -> Self {
        Self {
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

    fn render_license_warning(&self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let rect = ui.max_rect();

        // Draw the semi-transparent overlay
        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_black_alpha(190));

        // Calculate the center position, accounting for the warning box size
        let warning_width = 400.0;
        let warning_height = 200.0;
        let pos = egui::pos2(
            rect.center().x - (warning_width / 2.0),
            rect.center().y - (warning_height / 2.0),
        );

        // Create a frame for the warning message
        egui::Window::new("License Warning")
            .fixed_pos(pos)
            .fixed_size([warning_width, warning_height])
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // Warning header
                    ui.colored_label(
                        egui::Color32::RED,
                        egui::RichText::new("⚠ Subscription Ended")
                            .heading()
                            .strong()
                            .size(30.0),
                    );

                    // Warning messages
                    ui.label(
                        egui::RichText::new("Seems like you are no longer subscribed to Carelog")
                            .color(egui::Color32::WHITE)
                            .size(16.0),
                    );

                    ui.add_space(20.0);

                    // Center the purchase button
                    ui.with_layout(
                        egui::Layout::top_down_justified(egui::Align::Center),
                        |ui| {
                            let label = egui::RichText::new("Purchase Subscription")
                                .strong()
                                .size(16.0);

                            if ui
                                .add(egui::Button::new(label).min_size(egui::vec2(200.0, 40.0)))
                                .clicked()
                            {
                                ui.ctx().output_mut(|o| {
                                    o.open_url = Some(egui::output::OpenUrl {
                                        url: "https://sakshat.pages.dev/contact/".to_string(),
                                        new_tab: true, // Opens in a new tab
                                    });
                                });
                            }
                        },
                    );

                    ui.add_space(20.0);

                    ui.separator();
                    ui.label(
                        egui::RichText::new("Your Machine")
                            .monospace()
                            .color(egui::Color32::WHITE),
                    );

                    ui.colored_label(ui.visuals().warn_fg_color, &self.state.machine_uid);
                    ui.separator();
                });
            });
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
                                Screen::FindPatientHistory => "Search patient",
                                Screen::NewCase => "Case Details",
                                Screen::About => "About",
                            });

                    if response.clicked() {
                        self.current_screen = screen;
                    }
                };

                // Scroll area for navigation items
                egui::ScrollArea::vertical().show(ui, |ui| {
                    nav_button(ui, Screen::NewPatientCreation);
                    nav_button(ui, Screen::FindPatientHistory);
                    nav_button(ui, Screen::About)
                });

                // Version information at the bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.horizontal(|ui| {
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

        if ui
            .add_sized(
                [100.0, 24.0],
                eframe::egui::Button::new("Save").fill(ui.visuals().selection.bg_fill),
            )
            .clicked()
            && !self.state.first_name.is_empty()
            && !self.state.phone_number.is_empty()
        {
            match insert_new_patient(
                &self.state.conn,
                &self.state.first_name,
                &self.state.last_name,
                &self.state.phone_number,
                &self.state.date_of_birth,
                &self.state.address,
            ) {
                Ok(_) => {
                    self.state
                        .toasts
                        .success("Created a new patient")
                        .duration(Some(Duration::from_secs(7)));
                }

                Err(e) => {
                    self.state
                        .toasts
                        .error("Failed to create a new patient")
                        .duration(Some(Duration::from_secs(7)));

                    log::error!("{}", e);
                }
            }
        };
    }

    fn render_find_patient(&mut self, ui: &mut egui::Ui) {
        // Fetch the list of all patients from the database
        let list_of_patients = match helper_avaliable_patients_in_db(&self.state.conn) {
            Ok(patients) => patients,
            Err(e) => {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("❌ Failed to load patients").color(egui::Color32::RED),
                );

                log::error!("{}", e);
                return;
            }
        };

        // Header section
        ui.vertical(|ui| {
            ui.heading("New Case");
            ui.add_space(20.0);
        });

        // Search input section with styling
        eframe::egui::Grid::new("search_form_grid")
            .num_columns(2)
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                ui.label(eframe::egui::RichText::strong("Search Patient:".into()));
                ui.add(
                    eframe::egui::TextEdit::singleline(&mut self.state.search_text)
                        .desired_width(250.0)
                        .hint_text("Enter patient's name..."),
                );
                ui.end_row();
            });

        ui.add_space(10.0);

        // Filter suggestions based on search text
        let suggestions: Vec<String> = list_of_patients
            .iter()
            .filter(|patient| {
                patient
                    .to_lowercase()
                    .contains(&self.state.search_text.to_lowercase())
            })
            .cloned()
            .collect();

        // Display filtered results section
        if !self.state.search_text.is_empty() {
            if !suggestions.is_empty() {
                eframe::egui::ScrollArea::vertical()
                    // .max_height(400.0)
                    .show(ui, |ui| {
                        ui.add_space(10.0);
                        eframe::egui::Grid::new("search_results_grid")
                            .num_columns(1)
                            .spacing([10.0, 10.0])
                            .show(ui, |ui| {
                                for suggestion in suggestions {
                                    // Create a card-like effect with consistent sizing
                                    let card_frame = eframe::egui::Frame::none()
                                        .outer_margin(eframe::egui::vec2(0.0, 4.0))
                                        .inner_margin(eframe::egui::vec2(8.0, 8.0))
                                        .fill(ui.style().visuals.extreme_bg_color)
                                        .rounding(4.0)
                                        .stroke(
                                            ui.style().visuals.widgets.noninteractive.bg_stroke,
                                        );

                                    card_frame.show(ui, |ui| {
                                        ui.set_min_width(ui.available_width() - 20.0);
                                        ui.horizontal(|ui| {
                                            ui.with_layout(
                                                eframe::egui::Layout::left_to_right(
                                                    egui::Align::Center,
                                                ),
                                                |ui| {
                                                    ui.label("👤 ");
                                                    ui.strong(&suggestion);
                                                    ui.with_layout(
                                                        eframe::egui::Layout::right_to_left(
                                                            eframe::egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            if ui
                                                                .add_sized(
                                                                    [100.0, 24.0],
                                                                    eframe::egui::Button::new(
                                                                        "Create Case 📋",
                                                                    )
                                                                    .fill(
                                                                        ui.visuals()
                                                                            .selection
                                                                            .bg_fill,
                                                                    ),
                                                                )
                                                                .clicked()
                                                            {
                                                                self.state.current_patient =
                                                                    suggestion.clone();
                                                                self.state
                                                                    .toasts
                                                                    .success(format!(
                                                                        "Creating case for {}",
                                                                        suggestion
                                                                    ))
                                                                    .duration(Some(
                                                                        Duration::from_secs(3),
                                                                    ));

                                                                self.state.id = match  extract_number_from_brackets(
                                                                        &suggestion,
                                                                    ){
                                                                        Some(n) => n,
                                                                        None => 0,
                                                                    };

                                                                self.current_screen =
                                                                    Screen::NewCase;
                                                            }
                                                        },
                                                    );
                                                },
                                            );
                                        });
                                    });
                                    ui.end_row();
                                }
                            });
                    });
            } else {
                ui.label(
                    egui::RichText::new("ℹ Patient not found")
                        .color(ui.visuals().error_fg_color)
                        .strong(),
                );
            }
        } else {
            ui.label(
                egui::RichText::new("ℹ Please enter a name")
                    .color(ui.visuals().warn_fg_color)
                    .strong(),
            );
        }
    }

    fn render_new_case(&mut self, ui: &mut egui::Ui) {
        // Header section
        ui.vertical(|ui| {
            ui.heading("New Case");
            ui.set_width(ui.available_width());
            ui.add_space(20.0);
            eframe::egui::Grid::new("patient_info_grid")
                .num_columns(2)
                // .spacing([10.0, 10.0])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::strong("Patient".into()));
                    });

                    ui.label(
                        egui::RichText::strong(self.state.current_patient.clone().into())
                            .color(ui.visuals().hyperlink_color),
                    );

                    ui.end_row();
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::strong("Vist Date".into()));
                    });

                    ui.add(egui_extras::DatePickerButton::new(
                        &mut self.state.visit_date,
                    ));

                    ui.end_row();
                });

            ui.add_space(20.0);

            eframe::egui::ScrollArea::vertical()
                .id_salt("case_scroll_area")
                .auto_shrink([true, true])
                .show(ui, |ui| {
                    ui.label(egui::RichText::strong("Diagnosis Overview".into()));
                    ui.add_space(10.0);
                    ui.add(
                        eframe::egui::TextEdit::multiline(&mut self.state.diagnosis_overview)
                            .desired_rows(4)
                            .desired_width(ui.available_width() - 20.0),
                    );

                    ui.add_space(20.0);

                    ui.label(egui::RichText::strong("Detailed Notes".into()));
                    ui.add_space(10.0);
                    ui.add(
                        eframe::egui::TextEdit::multiline(&mut self.state.detailed_notes)
                            .desired_rows(10)
                            .desired_width(ui.available_width() - 20.0),
                    );

                    ui.add_space(20.0);

                    ui.label(egui::RichText::strong("Medical Tests".into()));
                    ui.add_space(10.0);
                    ui.add(
                        eframe::egui::TextEdit::multiline(&mut self.state.medical_test_info)
                            .desired_rows(4)
                            .desired_width(ui.available_width() - 20.0),
                    );

                    ui.add_space(10.0);
                    if ui
                        .add_sized(
                            [100.0, 24.0],
                            eframe::egui::Button::new("Save").fill(ui.visuals().selection.bg_fill),
                        )
                        .clicked()
                    {
                        match insert_patient_data(
                            &self.state.conn,
                            &self.state.id,
                            &self.state.diagnosis_overview,
                            &self.state.detailed_notes,
                            &self.state.medical_test_info,
                        ) {
                            Ok(_) => {
                                self.state
                                    .toasts
                                    .success("Saved patient data")
                                    .duration(Some(Duration::from_secs(7)));
                            }

                            Err(e) => {
                                self.state
                                    .toasts
                                    .error("Failed to save patient data")
                                    .duration(Some(Duration::from_secs(7)));

                                log::error!("{}", e);
                            }
                        }
                    };
                });
        });
    }

    fn render_about(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                // Title Section
                ui.add_space(20.0);
                ui.heading("Carelog");
                ui.colored_label(ui.visuals().warn_fg_color, "Alpha");
                ui.add_space(8.0);

                // Version Info
                ui.monospace(format!("Version {}", "0.1.0"));
                ui.add_space(8.0);

                ui.separator();
                ui.add_space(8.0);

                // Country of Origin
                ui.monospace("Made in India ❤");
                ui.add_space(8.0);

                // Technology Stack
                ui.monospace("Built with Rust & SQLite");
                ui.add_space(8.0);

                ui.separator();
                ui.add_space(8.0);

                ui.monospace("Machine");
                ui.monospace(&self.state.machine_uid);
                ui.add_space(8.0);

                ui.separator();
                ui.add_space(8.0);
                ui.monospace("License");
                if self.state.is_license_valid {
                    ui.label(
                        egui::RichText::new("Valid").color(egui::Color32::from_rgb(0, 128, 0)),
                    );
                    ui.label(egui::RichText::new(&self.state.customer).italics());
                } else {
                    ui.monospace(
                        egui::RichText::new("Invalid").color(egui::Color32::from_rgb(255, 0, 0)),
                    );
                }
                ui.add_space(8.0);
                ui.separator();
            });

            ui.add_space(10.0); // Add space at the end for better separation
        });
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
        // Only show sidebar if license is valid
        if self.state.is_license_valid {
            self.render_sidebar(ctx);
        }

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
            if !self.state.is_license_valid {
                self.render_license_warning(ctx, ui);
            } else {
                // Regular app content

                egui::Frame::none().show(ui, |ui| match self.current_screen {
                    Screen::NewPatientCreation => self.render_new_patient(ui),
                    Screen::FindPatientHistory => self.render_find_patient(ui),
                    Screen::NewCase => self.render_new_case(ui),
                    Screen::About => self.render_about(ui),
                });
            }
        });
    }
}

fn credits(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label(" © 2024 Sakshat Shinde");
    });
}

#[allow(dead_code)]
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
