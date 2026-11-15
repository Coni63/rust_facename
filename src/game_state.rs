//! État de l'application et logique de génération des parties.

use rand::RngExt;
use rand::seq::{IndexedRandom, SliceRandom};

use crate::data::{GameData, Sex};
use crate::modes::{Attr, GameSettings, Mode};

/// Nombre de propositions par question (réduit s'il n'y a pas assez de matière).
const NUM_CHOICES: usize = 4;

pub enum AppState {
    Menu,
    Learning { image_index: usize, start_time: f64 },
    Question { question_index: usize, selected: Option<usize> },
    Results,
}

#[derive(Clone)]
pub struct Sample {
    pub name: String,
    /// Poste déjà décliné selon le genre.
    pub position: String,
    /// Index dans `config.position` (identifie le poste quel que soit le genre).
    pub job_id: usize,
    pub img_path: String,
    pub sex: Sex,
}

impl Sample {
    pub fn attr(&self, attr: Attr) -> &str {
        match attr {
            Attr::Face => &self.img_path,
            Attr::Name => &self.name,
            Attr::Job => &self.position,
        }
    }
}

pub struct QuizQuestion {
    pub sample_index: usize,
    /// Jeu pur de cette question (en mode Mix, varie d'une question à l'autre).
    pub mode: Mode,
    /// Texte (nom / poste) ou chemin d'image selon `mode.answer()`.
    pub choices: Vec<String>,
    pub correct_index: usize,
}

pub struct MyGameApp {
    pub state: AppState,
    pub settings: GameSettings,
    pub data: GameData,
    pub dataset: Vec<Sample>,
    pub questions: Vec<QuizQuestion>,
    pub score: usize,
}

impl Default for MyGameApp {
    fn default() -> Self {
        Self {
            state: AppState::Menu,
            settings: GameSettings::default(),
            data: GameData::load(),
            dataset: Vec::new(),
            questions: Vec::new(),
            score: 0,
        }
    }
}

impl MyGameApp {
    /// Nombre de profils réellement jouables (limité par les visages et postes disponibles).
    pub fn num_profiles(&self) -> usize {
        self.settings
            .difficulty
            .profiles()
            .min(self.data.faces.len())
            .min(self.data.config.position.len())
    }

    /// Prépare une partie complète (profils + questions).
    pub fn start_game(&mut self) {
        self.create_dataset();
        self.generate_questions();
    }

    /// Tire N profils : visages, noms complets et postes tous distincts.
    fn create_dataset(&mut self) {
        let mut rng = rand::rng();
        let n = self.num_profiles();
        self.dataset.clear();
        self.questions.clear();
        self.score = 0;

        let mut faces: Vec<_> = self.data.faces.iter().collect();
        faces.shuffle(&mut rng);
        let mut job_ids: Vec<usize> = (0..self.data.config.position.len()).collect();
        job_ids.shuffle(&mut rng);

        for (face, job_id) in faces.into_iter().zip(job_ids).take(n) {
            let name = self
                .unique_name(face.sex, &mut rng)
                .unwrap_or_else(|| "Inconnu".to_string());
            self.dataset.push(Sample {
                name,
                position: self.data.config.position[job_id].for_sex(face.sex).to_string(),
                job_id,
                img_path: face.path.clone(),
                sex: face.sex,
            });
        }
    }

    /// Nom complet du bon genre, absent du dataset en cours de construction.
    fn unique_name(&self, sex: Sex, rng: &mut impl rand::Rng) -> Option<String> {
        let firstnames = self.data.config.firstname.for_sex(sex);
        for _ in 0..200 {
            let name = format!(
                "{} {}",
                firstnames.choose(rng)?,
                self.data.config.lastname.choose(rng)?
            );
            if !self.dataset.iter().any(|s| s.name == name) {
                return Some(name);
            }
        }
        None
    }

    /// Génère K questions. Les mauvaises réponses mélangent des profils déjà vus
    /// et des valeurs inventées (ou d'autres visages du fonds d'images).
    /// En mode Mix, les types de questions sont répartis sur les 6 jeux purs.
    fn generate_questions(&mut self) {
        let mut rng = rand::rng();
        let k = self.settings.difficulty.questions().min(self.dataset.len());
        let want = NUM_CHOICES - 1;

        let mut indices: Vec<usize> = (0..self.dataset.len()).collect();
        indices.shuffle(&mut rng);
        indices.truncate(k);

        let mut pure_modes = Mode::PURE;
        pure_modes.shuffle(&mut rng);

        let mut questions = Vec::with_capacity(k);
        for (q, sample_index) in indices.into_iter().enumerate() {
            let mode = match self.settings.mode {
                Mode::Mix => pure_modes[q % pure_modes.len()],
                pure => pure,
            };
            let answer = mode.answer();
            let sample = &self.dataset[sample_index];
            let correct = sample.attr(answer).to_string();

            // Noms et visages doivent rester du même genre que la bonne réponse,
            // sinon elle s'élimine toute seule. Les postes sont déclinés au genre
            // de la bonne réponse, donc aucun filtre n'est nécessaire.
            let sex_matters = answer != Attr::Job;

            // Distracteurs issus des autres profils vus.
            let mut seen: Vec<String> = self
                .dataset
                .iter()
                .enumerate()
                .filter(|&(i, other)| {
                    i != sample_index && (!sex_matters || other.sex == sample.sex)
                })
                .map(|(_, other)| self.value_for(other, answer, sample.sex))
                .collect();
            seen.shuffle(&mut rng);

            // Distracteurs inventés (ou visages hors dataset).
            let mut invented = self.invented(answer, sample.sex, want, &mut rng);

            // Moitié vus, moitié inventés ; on complète avec ce qui reste.
            let from_seen = want.div_ceil(2).min(seen.len());
            let mut choices: Vec<String> = seen.drain(..from_seen).collect();
            while choices.len() < want {
                match invented.pop().or_else(|| seen.pop()) {
                    Some(d) => choices.push(d),
                    None => break,
                }
            }

            let correct_index = rng.random_range(0..=choices.len());
            choices.insert(correct_index, correct);
            questions.push(QuizQuestion {
                sample_index,
                mode,
                choices,
                correct_index,
            });
        }
        self.questions = questions;
    }

    /// Valeur d'un attribut d'un autre profil, présentée comme distracteur pour
    /// une personne de genre `target_sex` (les postes sont alors déclinés à son genre).
    fn value_for(&self, other: &Sample, attr: Attr, target_sex: Sex) -> String {
        match attr {
            Attr::Job => self.data.config.position[other.job_id]
                .for_sex(target_sex)
                .to_string(),
            _ => other.attr(attr).to_string(),
        }
    }

    /// Valeurs plausibles mais n'appartenant à aucun profil présenté.
    fn invented(
        &self,
        attr: Attr,
        sex: Sex,
        count: usize,
        rng: &mut impl rand::Rng,
    ) -> Vec<String> {
        let used = |v: &str| self.dataset.iter().any(|s| s.attr(attr) == v);
        let cfg = &self.data.config;

        let mut out: Vec<String> = match attr {
            Attr::Face => self
                .data
                .faces
                .iter()
                .filter(|f| f.sex == sex)
                .map(|f| f.path.clone())
                .filter(|p| !used(p))
                .collect(),
            Attr::Job => cfg
                .position
                .iter()
                .enumerate()
                .filter(|&(id, _)| !self.dataset.iter().any(|s| s.job_id == id))
                .map(|(_, p)| p.for_sex(sex).to_string())
                .collect(),
            Attr::Name => {
                let firstnames = cfg.firstname.for_sex(sex);
                let mut names: Vec<String> = Vec::new();
                for _ in 0..200 {
                    if names.len() >= count {
                        break;
                    }
                    let (Some(f), Some(l)) = (firstnames.choose(rng), cfg.lastname.choose(rng))
                    else {
                        break;
                    };
                    let name = format!("{f} {l}");
                    if !used(&name) && !names.contains(&name) {
                        names.push(name);
                    }
                }
                names
            }
        };
        out.shuffle(rng);
        out.truncate(count);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modes::Difficulty;

    #[test]
    fn questions_are_well_formed_for_every_mode_and_difficulty() {
        let mut app = MyGameApp::default();
        for mode in Mode::ALL {
            for difficulty in Difficulty::ALL {
                app.settings.mode = mode;
                app.settings.difficulty = difficulty;
                for _ in 0..50 {
                    app.start_game();
                    assert!(!app.questions.is_empty());
                    for q in &app.questions {
                        assert_ne!(q.mode, Mode::Mix);
                        let sample = &app.dataset[q.sample_index];
                        assert_eq!(q.choices[q.correct_index], sample.attr(q.mode.answer()));
                        let mut uniq = q.choices.clone();
                        uniq.sort();
                        uniq.dedup();
                        assert_eq!(uniq.len(), q.choices.len(), "doublon: {:?}", q.choices);
                        assert!(q.choices.len() >= 2);
                    }
                }
            }
        }
    }

    #[test]
    fn mix_covers_several_question_types() {
        let mut app = MyGameApp::default();
        app.settings.mode = Mode::Mix;
        app.settings.difficulty = Difficulty::Hard;
        app.start_game();
        let mut modes: Vec<_> = app.questions.iter().map(|q| q.mode).collect();
        modes.dedup();
        assert_eq!(app.questions.len(), 6);
        assert!(modes.len() > 1);
    }
}
