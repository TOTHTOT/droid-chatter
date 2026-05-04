//! Droid type definitions

/// Mood variants for emotional droids like BD-1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Happy,
    Sad,
    Angry,
}

impl Mood {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mood::Happy => "happy",
            Mood::Sad => "sad",
            Mood::Angry => "angry",
        }
    }
}

/// Droid type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DroidType {
    Astro,
    Bd1,
    Do,
    Bb8,
    Chopper,
    Mouse,
    Probe,
    R2,
}

impl DroidType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DroidType::Astro => "astro",
            DroidType::Bd1 => "bd1",
            DroidType::Do => "do",
            DroidType::Bb8 => "bb8",
            DroidType::Chopper => "chopper",
            DroidType::Mouse => "mouse",
            DroidType::Probe => "probe",
            DroidType::R2 => "r2",
        }
    }
}
