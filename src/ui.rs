use crate::game_state::{AppState, MyGameApp, Sample};
use crate::modes::{Attr, Difficulty, Mode};
use eframe::egui::{self, Color32};

const GREEN: Color32 = Color32::from_rgb(46, 160, 67);
const RED: Color32 = Color32::from_rgb(200, 60, 60);

impl eframe::App for MyGameApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| match self.state {
            AppState::Menu => self.show_menu(ui),
            AppState::Learning {
                image_index,
                start_time,
            } => {
                let now = ctx.input(|i| i.time);
                if now - start_time >= self.settings.learning_time() {
                    self.next_profile(image_index, now);
                }
                self.show_learning(ui, image_index, start_time);
                ctx.request_repaint(); // fait avancer le minuteur
            }
            AppState::Question {
                question_index,
                selected,
            } => self.show_questions(ui, question_index, selected),
            AppState::Results => self.show_results(ui),
        });
    }
}

fn show_image(ui: &mut egui::Ui, path: &str, size: f32) {
    ui.add(egui::Image::new(path).fit_to_exact_size(egui::vec2(size, size)));
}

impl MyGameApp {
    fn begin(&mut self, mode: Mode, now: f64) {
        self.settings.mode = mode;
        self.start_game();
        self.state = AppState::Learning {
            image_index: 0,
            start_time: now,
        };
    }

    fn next_profile(&mut self, idx: usize, now: f64) {
        self.state = if idx + 1 < self.dataset.len() {
            AppState::Learning {
                image_index: idx + 1,
                start_time: now,
            }
        } else {
            AppState::Question {
                question_index: 0,
                selected: None,
            }
        };
    }

    fn show_menu(&mut self, ui: &mut egui::Ui) {
        let now = ui.ctx().input(|i| i.time);
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("Facename");
            ui.label("Mémorisez les visages, les noms et les postes");
            ui.add_space(15.0);

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 330.0).max(0.0) / 2.0);
                ui.label("Difficulté :");
                for d in Difficulty::ALL {
                    ui.selectable_value(&mut self.settings.difficulty, d, d.label());
                }
            });
            let d = self.settings.difficulty;
            ui.label(
                egui::RichText::new(format!(
                    "{} profils · {} questions · {} s par profil (×1,5 en Mix)",
                    self.num_profiles(),
                    d.questions().min(self.num_profiles()),
                    d.learning_time()
                ))
                .weak(),
            );
            ui.add_space(20.0);

            let mut chosen = None;
            for mode in Mode::ALL {
                let btn = egui::Button::new(egui::RichText::new(mode.title()).size(18.0).strong())
                    .min_size(egui::vec2(260.0, 40.0));
                if ui.add(btn).on_hover_text(mode.description()).clicked() {
                    chosen = Some(mode);
                }
                ui.label(egui::RichText::new(mode.description()).small().weak());
                ui.add_space(8.0);
            }
            if let Some(mode) = chosen {
                self.begin(mode, now);
            }
        });
    }

    fn show_learning(&mut self, ui: &mut egui::Ui, idx: usize, start_time: f64) {
        let total = self.dataset.len();
        let Some(sample) = self.dataset.get(idx).cloned() else {
            return;
        };
        let now = ui.ctx().input(|i| i.time);
        let progress = ((now - start_time) / self.settings.learning_time()) as f32;

        ui.vertical_centered(|ui| {
            ui.add_space(15.0);
            ui.heading(format!("Mémorisez ce profil ({} / {})", idx + 1, total));
            ui.add_space(10.0);
            ui.add(egui::ProgressBar::new(progress.clamp(0.0, 1.0)).desired_width(300.0));
            ui.add_space(15.0);

            show_image(ui, &sample.img_path, 260.0);

            ui.add_space(15.0);
            ui.label(egui::RichText::new(&sample.name).size(30.0).strong());
            ui.label(egui::RichText::new(&sample.position).size(20.0).italics());
            ui.add_space(20.0);

            if ui.button("Suivant ➡").clicked() {
                self.next_profile(idx, now);
            }
        });
    }

    fn show_questions(&mut self, ui: &mut egui::Ui, idx: usize, selected: Option<usize>) {
        let Some(question) = self.questions.get(idx) else {
            self.state = AppState::Results;
            return;
        };
        let mode = question.mode;
        let choices = question.choices.clone();
        let correct_index = question.correct_index;
        let total = self.questions.len();
        let Some(sample) = self.dataset.get(question.sample_index).cloned() else {
            return;
        };

        let mut clicked: Option<usize> = None;
        let mut go_next = false;

        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.label(format!("Question {} / {}", idx + 1, total));
            ui.add_space(10.0);

            Self::show_cue(ui, mode.cue(), &sample);

            ui.add_space(15.0);
            ui.heading(mode.prompt());
            ui.add_space(15.0);

            let fill_for = |i: usize| -> Option<Color32> {
                let sel = selected?;
                if i == correct_index {
                    Some(GREEN)
                } else if i == sel {
                    Some(RED)
                } else {
                    None
                }
            };

            // Les boutons ont une taille fixe : on réserve exactement la largeur
            // de la rangée/grille pour que `vertical_centered` la centre.
            let spacing = 10.0;
            let (btn_size, cols) = if mode.answer() == Attr::Face {
                (egui::vec2(150.0, 150.0), choices.len().max(1))
            } else {
                (egui::vec2(260.0, 40.0), 2.min(choices.len()).max(1))
            };
            let rows = choices.len().div_ceil(cols);
            let block = egui::vec2(
                cols as f32 * btn_size.x + (cols - 1) as f32 * spacing,
                rows as f32 * btn_size.y + (rows - 1) as f32 * spacing,
            );
            ui.allocate_ui_with_layout(
                block,
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::Grid::new("choices_grid")
                        .num_columns(cols)
                        .spacing([spacing, spacing])
                        .show(ui, |ui| {
                            for (i, choice) in choices.iter().enumerate() {
                                let mut btn = if mode.answer() == Attr::Face {
                                    egui::Button::image(
                                        egui::Image::new(choice.as_str())
                                            .fit_to_exact_size(egui::vec2(140.0, 140.0)),
                                    )
                                } else {
                                    egui::Button::new(egui::RichText::new(choice).size(15.0))
                                };
                                if let Some(c) = fill_for(i) {
                                    btn = btn.fill(c);
                                }
                                if ui.add_sized(btn_size, btn).clicked() && selected.is_none() {
                                    clicked = Some(i);
                                }
                                if (i + 1) % cols == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                },
            );

            if let Some(sel) = selected {
                ui.add_space(15.0);
                let (msg, color) = if sel == correct_index {
                    ("Correct !".to_string(), GREEN)
                } else {
                    (format!("Raté — réponse : {}", Self::answer_label(mode, &sample)), RED)
                };
                ui.label(egui::RichText::new(msg).size(18.0).color(color).strong());
                ui.add_space(8.0);
                let last = idx + 1 >= total;
                if ui.button(if last { "Résultats" } else { "Question suivante ➡" }).clicked() {
                    go_next = true;
                }
            }
        });

        if let Some(i) = clicked {
            if i == correct_index {
                self.score += 1;
            }
            self.state = AppState::Question {
                question_index: idx,
                selected: Some(i),
            };
        } else if go_next {
            self.state = if idx + 1 < total {
                AppState::Question {
                    question_index: idx + 1,
                    selected: None,
                }
            } else {
                AppState::Results
            };
        }
    }

    fn show_cue(ui: &mut egui::Ui, cue: Attr, sample: &Sample) {
        match cue {
            Attr::Face => show_image(ui, &sample.img_path, 220.0),
            Attr::Name => {
                ui.add_space(40.0);
                ui.label(egui::RichText::new(&sample.name).size(34.0).strong());
            }
            Attr::Job => {
                ui.add_space(40.0);
                ui.label(egui::RichText::new(&sample.position).size(28.0).italics());
            }
        }
    }

    /// Texte de la bonne réponse (pour les visages, le nom de la personne).
    fn answer_label(mode: Mode, sample: &Sample) -> String {
        match mode.answer() {
            Attr::Face => sample.name.clone(),
            attr => sample.attr(attr).to_string(),
        }
    }

    fn show_results(&mut self, ui: &mut egui::Ui) {
        let total = self.questions.len();
        let score = self.score;
        let now = ui.ctx().input(|i| i.time);
        let mode = self.settings.mode;
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 5.0);
            ui.heading(format!("Résultats — {}", mode.title()));
            ui.add_space(20.0);

            ui.label(egui::RichText::new(format!("{score} / {total}")).size(48.0).strong());
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
                self.begin(mode, now);
            }
            ui.add_space(8.0);
            if ui.button("Menu").clicked() {
                self.state = AppState::Menu;
            }
        });
    }
}
