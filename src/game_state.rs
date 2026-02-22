use image::ImageReader;
use rand::Rng;
use rand::seq::IndexedRandom;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;

use crate::ui;

pub enum AppState {
    Menu,
    Learning { image_index: usize, start_time: f64 },
    Question { question_index: usize },
    Results,
}

pub enum Mode {
    FaceToName,
    NameToFace,
    FaceToPosition,
    PositionToFace,
}

pub struct GameSettings {
    pub mode: Mode,
    pub num_pictures: usize,
    pub num_questions: usize,
    pub learning_time: f64, // seconds
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            mode: Mode::FaceToName,
            num_pictures: 3,
            num_questions: 3,
            learning_time: 5.0, // seconds
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameData {
    pub firstname: FirstNames,
    pub lastname: Vec<String>,
    pub position: Vec<String>,
    pub images: Vec<ImageEntry>,
}

pub struct Sample {
    pub name: String,
    pub position: String,
    pub img_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FirstNames {
    pub male: Vec<String>,
    pub female: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")] // Permet de lire "male" ou "female" en minuscules
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageEntry {
    pub name: String,
    pub sex: Sex,
}

pub struct MyGameApp {
    pub state: AppState,
    pub settings: GameSettings,
    pub data: GameData,
    pub dataset: Vec<Sample>,
}

impl Default for MyGameApp {
    fn default() -> Self {
        let file_path = "assets/config.json";

        // 2. Lire le contenu du fichier dans une String
        let data = fs::read_to_string(file_path)
            .expect("Impossible de lire le fichier. Vérifie qu'il existe !");

        // 3. Désérialiser le JSON vers la structure TeamData
        let team: GameData =
            serde_json::from_str(&data).expect("Erreur de désérialisation du JSON");

        println!("Données chargées : {:?}", team);

        Self {
            state: AppState::Menu,
            settings: GameSettings::default(),
            data: team,
            dataset: Vec::new(),
        }
    }
}

impl MyGameApp {
    pub fn create_dataset(&mut self) {
        let mut rng = rand::rng();

        // On mélange et on prend le nombre nécessaire directement
        self.data.images.shuffle(&mut rng);
        let picked_entries = self.data.images.iter().take(self.settings.num_pictures);

        for entry in picked_entries {
            // 1. Charger l'image
            let path = format!("file://assets/img/{}", entry.name);

            // 2. Sélectionner la liste de prénoms selon le genre
            let name_pool = match entry.sex {
                Sex::Male => &self.data.firstname.male,
                Sex::Female => &self.data.firstname.female,
            };

            // 3. Tirage aléatoire (on utilise une closure pour éviter la répétition)
            let mut pick = |list: &Vec<String>, default: &str| {
                list.choose(&mut rng)
                    .cloned()
                    .unwrap_or_else(|| default.to_string())
            };

            let firstname = pick(name_pool, "Inconnu");
            let lastname = pick(&self.data.lastname, "Nom");
            let position = pick(&self.data.position, "Poste");

            // 4. Construction de l'échantillon
            self.dataset.push(Sample {
                name: format!("{} {}", firstname, lastname),
                position,
                img_path: path,
            });
        }
    }
}
