//! Utility functions

use std::path::Path;
use walkdir::WalkDir;

pub use crate::droid::DroidType;

/// Generate random string of specified length
pub fn generate_random_string(length: usize) -> String {
    let alphabet: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
    (0..length)
        .map(|_| alphabet[rand_idx(alphabet.len())])
        .collect()
}

/// Simple random index helper
pub fn rand_idx(max: usize) -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as usize
        % max
}

/// Get all available phrases for a specific droid
pub fn get_available_phrases(sounds_dir: &Path, droid: DroidType) -> Vec<String> {
    let mut path = sounds_dir.to_path_buf();
    path.push(droid.as_str());

    WalkDir::new(&path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str().map(|s| s.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_string() {
        let result = generate_random_string(10);
        assert_eq!(result.len(), 10);
        assert!(result.chars().all(|c| c.is_alphabetic()));
    }

    #[test]
    fn test_get_available_phrases() {
        let tmpdir = tempfile::TempDir::new().unwrap();
        let sounds = tmpdir.path();

        let do_dir = sounds.join("do");
        std::fs::create_dir_all(&do_dir).unwrap();
        std::fs::write(do_dir.join("hello1.wav"), b"RIFF").unwrap();
        std::fs::write(do_dir.join("batterycharged.wav"), b"RIFF").unwrap();

        let phrases = get_available_phrases(sounds, DroidType::Do);
        assert!(phrases.contains(&"hello1".to_string()));
        assert!(phrases.contains(&"batterycharged".to_string()));
    }
}
