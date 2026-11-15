//! Chargement des données : noms/postes (config.json) et visages (dossiers d'images).

use serde::Deserialize;
use std::fs;
use std::path::Path;

const CONFIG_PATH: &str = "assets/config.json";
const IMG_DIR: &str = "assets/img";
const IMG_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sex {
    Male,
    Female,
}

impl Sex {
    /// Sous-dossier de `assets/img` contenant les visages de ce genre.
    fn folder(self) -> &'static str {
        match self {
            Sex::Male => "men",
            Sex::Female => "women",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FirstNames {
    pub male: Vec<String>,
    pub female: Vec<String>,
}

impl FirstNames {
    pub fn for_sex(&self, sex: Sex) -> &[String] {
        match sex {
            Sex::Male => &self.male,
            Sex::Female => &self.female,
        }
    }
}

/// Poste : identique pour les deux genres, ou décliné au masculin/féminin.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Position {
    Neutral(String),
    Gendered { male: String, female: String },
}

impl Position {
    pub fn for_sex(&self, sex: Sex) -> &str {
        match (self, sex) {
            (Position::Neutral(s), _) => s,
            (Position::Gendered { male, .. }, Sex::Male) => male,
            (Position::Gendered { female, .. }, Sex::Female) => female,
        }
    }
}

/// Contenu de `config.json`.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub firstname: FirstNames,
    pub lastname: Vec<String>,
    pub position: Vec<Position>,
}

/// Un visage : URI chargeable par egui + genre (déduit du dossier).
#[derive(Debug, Clone)]
pub struct Face {
    pub path: String,
    pub sex: Sex,
}

pub struct GameData {
    pub config: Config,
    pub faces: Vec<Face>,
}

impl GameData {
    pub fn load() -> Self {
        let raw = fs::read_to_string(CONFIG_PATH)
            .unwrap_or_else(|e| panic!("Impossible de lire {CONFIG_PATH} : {e}"));
        let config: Config = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("{CONFIG_PATH} invalide : {e}"));

        let mut faces = Vec::new();
        for sex in [Sex::Male, Sex::Female] {
            faces.extend(scan_faces(sex));
        }
        Self { config, faces }
    }
}

/// Liste (triée) les images de `assets/img/<men|women>/`.
fn scan_faces(sex: Sex) -> Vec<Face> {
    let dir = Path::new(IMG_DIR).join(sex.folder());
    let Ok(entries) = fs::read_dir(&dir) else {
        eprintln!("Dossier d'images introuvable : {}", dir.display());
        return Vec::new();
    };

    let mut files: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let ext = path.extension()?.to_str()?.to_lowercase();
            IMG_EXTENSIONS
                .contains(&ext.as_str())
                .then(|| e.file_name().to_string_lossy().into_owned())
        })
        .collect();
    files.sort();

    files
        .into_iter()
        .map(|name| Face {
            path: format!("file://{IMG_DIR}/{}/{name}", sex.folder()),
            sex,
        })
        .collect()
}
