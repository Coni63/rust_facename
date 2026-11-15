//! Les jeux (indice → réponse) et les niveaux de difficulté.

/// Un des trois attributs d'un profil.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Attr {
    Face,
    Name,
    Job,
}

/// Un jeu = un indice (`cue`) montré et un attribut à retrouver (`answer`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    FaceToName,
    NameToFace,
    FaceToJob,
    JobToFace,
    NameToJob,
    JobToName,
    /// Mélange des six jeux : le type de chaque question est tiré parmi `Mode::PURE`.
    Mix,
}

impl Mode {
    /// Les six jeux "purs" (un seul sens indice → réponse).
    pub const PURE: [Mode; 6] = [
        Mode::FaceToName,
        Mode::NameToFace,
        Mode::FaceToJob,
        Mode::JobToFace,
        Mode::NameToJob,
        Mode::JobToName,
    ];

    /// Tous les jeux proposés dans le menu.
    pub const ALL: [Mode; 7] = [
        Mode::FaceToName,
        Mode::NameToFace,
        Mode::FaceToJob,
        Mode::JobToFace,
        Mode::NameToJob,
        Mode::JobToName,
        Mode::Mix,
    ];

    /// Indice montré. Réservé aux jeux purs : `Mix` est résolu question par question.
    pub fn cue(self) -> Attr {
        match self {
            Mode::FaceToName | Mode::FaceToJob => Attr::Face,
            Mode::NameToFace | Mode::NameToJob => Attr::Name,
            Mode::JobToFace | Mode::JobToName => Attr::Job,
            Mode::Mix => unreachable!("Mix est résolu en un jeu pur par question"),
        }
    }

    /// Attribut à retrouver. Réservé aux jeux purs.
    pub fn answer(self) -> Attr {
        match self {
            Mode::NameToFace | Mode::JobToFace => Attr::Face,
            Mode::FaceToName | Mode::JobToName => Attr::Name,
            Mode::FaceToJob | Mode::NameToJob => Attr::Job,
            Mode::Mix => unreachable!("Mix est résolu en un jeu pur par question"),
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Mode::FaceToName => "Visage → Nom",
            Mode::NameToFace => "Nom → Visage",
            Mode::FaceToJob => "Visage → Poste",
            Mode::JobToFace => "Poste → Visage",
            Mode::NameToJob => "Nom → Poste",
            Mode::JobToName => "Poste → Nom",
            Mode::Mix => "Mix",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Mode::FaceToName => "Retrouver le nom d'après le visage",
            Mode::NameToFace => "Retrouver le visage d'après le nom",
            Mode::FaceToJob => "Qui fait quoi ? Le poste d'après le visage",
            Mode::JobToFace => "Qui fait quoi ? Le visage d'après le poste",
            Mode::NameToJob => "Retrouver le poste d'après le nom",
            Mode::JobToName => "Retrouver le nom d'après le poste",
            Mode::Mix => "Un mélange des 6 jeux, avec un peu plus de temps",
        }
    }

    pub fn prompt(self) -> &'static str {
        match self {
            Mode::FaceToName => "Comment s'appelle cette personne ?",
            Mode::NameToFace => "Quel est le visage de cette personne ?",
            Mode::FaceToJob | Mode::NameToJob => "Quel poste occupe cette personne ?",
            Mode::JobToFace => "Qui occupe ce poste ?",
            Mode::JobToName => "Comment s'appelle la personne à ce poste ?",
            Mode::Mix => unreachable!("Mix est résolu en un jeu pur par question"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub const ALL: [Difficulty; 3] = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];

    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "Facile",
            Difficulty::Medium => "Moyen",
            Difficulty::Hard => "Difficile",
        }
    }

    /// N : nombre de profils présentés.
    pub fn profiles(self) -> usize {
        match self {
            Difficulty::Easy => 3,
            Difficulty::Medium => 5,
            Difficulty::Hard => 8,
        }
    }

    /// K : nombre de questions (K <= N).
    pub fn questions(self) -> usize {
        match self {
            Difficulty::Easy => 3,
            Difficulty::Medium => 4,
            Difficulty::Hard => 6,
        }
    }

    /// Temps d'affichage de chaque profil, en secondes.
    pub fn learning_time(self) -> f64 {
        match self {
            Difficulty::Easy => 6.0,
            Difficulty::Medium => 5.0,
            Difficulty::Hard => 3.5,
        }
    }
}

pub struct GameSettings {
    pub mode: Mode,
    pub difficulty: Difficulty,
}

impl GameSettings {
    /// Temps par profil : plus long en mode Mix (il faut retenir les trois infos).
    pub fn learning_time(&self) -> f64 {
        let base = self.difficulty.learning_time();
        if self.mode == Mode::Mix { base * 1.5 } else { base }
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            mode: Mode::FaceToName,
            difficulty: Difficulty::Easy,
        }
    }
}
