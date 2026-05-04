//! Error types for droid-chatter

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DroidError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid droid type: {0}")]
    InvalidDroid(String),

    #[error("Sound file not found: {0}")]
    SoundNotFound(String),

    #[error("No sounds directory provided")]
    NoSoundsDir,

    #[error("Audio playback error: {0}")]
    PlaybackError(String),
}
