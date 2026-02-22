use crate::game_state::{AppState, MyGameApp};
use eframe::egui;

impl eframe::App for MyGameApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| match self.state {
            AppState::Menu => {
                self.show_menu(ui);
            }
            AppState::Learning {
                image_index,
                start_time,
            } => {
                let elapsed = ctx.input(|i| i.time) - start_time;

                if elapsed >= self.settings.learning_time {
                    // Transition logic
                    if image_index < self.settings.num_pictures - 1 {
                        self.state = AppState::Learning {
                            image_index: image_index + 1,
                            start_time: ctx.input(|i| i.time),
                        };
                    } else {
                        self.state = AppState::Question { question_index: 0 };
                    }
                }

                self.show_learning(ui, image_index);
                ctx.request_repaint(); // Keep the timer ticking!
            }
            AppState::Question { question_index } => {
                self.show_questions(ui, question_index);
            }
            AppState::Results => {
                self.show_results(ui);
            }
        });
    }
}

impl MyGameApp {
    // Note: We don't necessarily need &egui::Context here because
    // we can get it from ui.ctx() if needed.
    fn show_menu(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.heading("LOGO");
            ui.add_space(20.0);

            if ui.button("Start Game").clicked() {
                self.create_dataset();
                // Change the state to trigger the next "page"
                self.state = AppState::Learning {
                    image_index: 0,
                    start_time: ui.ctx().input(|i| i.time),
                };
            }
        });
    }

    fn show_learning(&mut self, ui: &mut egui::Ui, idx: usize) {
        // 1. Récupérer le sample actuel
        if let Some(sample) = self.dataset.get(idx) {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading(format!(
                    "Mémorisez ce profil ({} / {})",
                    idx + 1,
                    self.dataset.len()
                ));
                ui.add_space(20.0);

                // Afficher l'image avec une taille fixe (ex: 200px)
                ui.image(&sample.img_path);

                ui.add_space(20.0);

                // 3. Afficher les informations
                ui.label(egui::RichText::new(&sample.name).size(30.0).strong());
                ui.label(egui::RichText::new(&sample.position).size(20.0).italics());

                ui.add_space(40.0);

                // 4. Bouton pour passer à la suite
                if ui.button("Suivant ➡").clicked() {
                    if idx + 1 < self.dataset.len() {
                        self.state = AppState::Learning {
                            image_index: idx + 1,
                            start_time: ui.ctx().input(|i| i.time),
                        };
                    } else {
                        // Si fini, on passe aux questions
                        self.state = AppState::Question { question_index: 0 };
                    }
                }
            });
        }
    }

    fn show_questions(&mut self, ui: &mut egui::Ui, idx: usize) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.heading("LOGO");
            ui.add_space(20.0);

            ui.heading(String::from("Question ") + &(idx + 1).to_string());
        });
    }

    fn show_results(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.heading("LOGO");
            ui.add_space(20.0);

            if ui.button("Start Game").clicked() {
                // Change the state to trigger the next "page"
                self.state = AppState::Menu;
            }
        });
    }
}
