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
                self.generate_questions();
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
        if let Some(question) = self.questions.get(idx) {
            let sample_index = question.sample_index;
            let choices = question.choices.clone();
            let correct_index = question.correct_index;
            let total = self.questions.len();

            if let Some(sample) = self.dataset.get(sample_index) {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(format!("Question {} / {}", idx + 1, total));
                    ui.add_space(10.0);

                    ui.image(&sample.img_path);

                    ui.add_space(15.0);
                    ui.heading("Qui est cette personne ?");
                    ui.add_space(15.0);

                    let mut answered: Option<bool> = None;
                    let btn_size = egui::vec2(160.0, 36.0);
                    egui::Grid::new("choices_grid")
                        .num_columns(2)
                        .spacing([10.0, 10.0])
                        .show(ui, |ui| {
                            for (i, choice) in choices.iter().enumerate() {
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new(choice).size(15.0))).clicked() {
                                    answered = Some(i == correct_index);
                                }
                                if i % 2 == 1 {
                                    ui.end_row();
                                }
                            }
                        });

                    if let Some(correct) = answered {
                        if correct {
                            self.score += 1;
                        }
                        if idx + 1 < total {
                            self.state = AppState::Question { question_index: idx + 1 };
                        } else {
                            self.state = AppState::Results;
                        }
                    }
                });
            }
        }
    }

    fn show_results(&mut self, ui: &mut egui::Ui) {
        let total = self.questions.len();
        let score = self.score;
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 4.0);
            ui.heading("Résultats");
            ui.add_space(20.0);

            ui.label(egui::RichText::new(format!("{} / {}", score, total)).size(48.0).strong());
            ui.add_space(8.0);

            let pct = if total > 0 { score * 100 / total } else { 0 };
            let comment = match pct {
                100 => "Parfait ! 🎉",
                80..=99 => "Excellent !",
                60..=79 => "Bien joué !",
                40..=59 => "Pas mal…",
                _ => "À retravailler !",
            };
            ui.label(egui::RichText::new(comment).size(20.0).italics());

            ui.add_space(30.0);

            if ui.button("Rejouer").clicked() {
                self.state = AppState::Menu;
            }
        });
    }
}
