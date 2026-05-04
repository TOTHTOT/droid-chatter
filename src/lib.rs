//! # droid-chatter
//!
//! A Rust library for generating droid (BD-1, D-O, Astromech, etc.) sounds.
//! Based on the node-ttastromech project.
//!
//! ## Supported Droids
//!
//! - **BD-1**: From Star Wars Jedi: Fallen Order, supports moods (happy, sad, angry)
//! - **D-O**: Pre-recorded phrases
//! - **Astromech**: Classic R2-D2 style letter-based sound
//! - **BB-8**: Pre-recorded sounds
//! - **Mouse Droid**: MSE-6 style
//! - **Chopper**: Protocol droid
//! - **Probe Droid**: Imperial probe droid
//! - **R2**: R2 unit sounds

use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

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

/// Mood variants for emotional droids like BD-1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Happy,
    Sad,
    Angry,
}

impl Mood {
    fn as_str(&self) -> &'static str {
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
    fn as_str(&self) -> &'static str {
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

/// Audio data structure for cpal playback
#[derive(Debug, Clone)]
pub struct AudioData {
    pub samples: Vec<i16>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl AudioData {
    /// Create a new AudioData instance
    pub fn new(samples: Vec<i16>, sample_rate: u32, channels: u16) -> Self {
        Self {
            samples,
            sample_rate,
            channels,
        }
    }

    /// Get the total number of frames (samples / channels)
    pub fn frames(&self) -> usize {
        self.samples.len() / self.channels as usize
    }
}

/// Main struct for droid sound generation
pub struct DroidChatter<'a> {
    sounds_dir: &'a Path,
}

impl<'a> DroidChatter<'a> {
    /// Create a new DroidChatter instance with the sounds directory
    pub fn new(sounds_dir: &'a Path) -> Result<Self, DroidError> {
        if !sounds_dir.exists() {
            return Err(DroidError::NoSoundsDir);
        }
        Ok(Self { sounds_dir })
    }

    /// Build directory path for a droid and optional mood
    fn build_dir(&self, droid: DroidType, mood: Option<Mood>) -> PathBuf {
        let mut path = self.sounds_dir.to_path_buf();
        path.push(droid.as_str());
        if let Some(m) = mood {
            path.push(m.as_str());
        }
        path
    }

    /// Build paths for letter-based sound files
    fn build_letter_paths(
        &self,
        phrase: &str,
        droid: DroidType,
        mood: Option<Mood>,
    ) -> Vec<PathBuf> {
        let filtered = phrase.chars().filter(|c| c.is_alphabetic());
        let dir = self.build_dir(droid, mood);

        filtered
            .flat_map(|ch| {
                let mut path = dir.clone();
                path.push(format!("{}.wav", ch.to_ascii_lowercase()));
                if path.exists() {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get raw audio data from a phrase (for cpal playback)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use droid_chatter::{DroidChatter, Mood};
    /// use std::path::Path;
    ///
    /// let chatter = DroidChatter::new(Path::new("sounds")).unwrap();
    /// let audio = chatter.get_audio_data("hello", DroidType::Bd1, Some(Mood::Happy)).unwrap();
    ///
    /// // Use with cpal:
    /// let config = cpal::StreamConfig {
    ///     channels: audio.channels,
    ///     sample_rate: cpal::SampleRate(audio.sample_rate),
    ///     ..Default::default()
    /// };
    /// ```
    pub fn get_audio_data(
        &self,
        phrase: &str,
        droid: DroidType,
        mood: Option<Mood>,
    ) -> Result<AudioData, DroidError> {
        let paths = self.build_letter_paths(phrase, droid, mood);
        if paths.is_empty() {
            return Err(DroidError::SoundNotFound("No sound files found".into()));
        }

        let mut all_samples: Vec<i16> = Vec::new();
        let mut sample_rate = 44100u32;
        let mut channels = 1u16;

        for path in &paths {
            if let Ok(file) = File::open(path) {
                if let Ok(reader) = hound::WavReader::new(BufReader::new(file)) {
                    sample_rate = reader.spec().sample_rate;
                    channels = reader.spec().channels;
                    for s in reader.into_samples::<i16>().flatten() {
                        all_samples.push(s);
                    }
                }
            }
        }

        if all_samples.is_empty() {
            return Err(DroidError::SoundNotFound("No audio data".into()));
        }

        Ok(AudioData::new(all_samples, sample_rate, channels))
    }

    /// Get BD-1 raw audio data
    pub fn bd1_audio(&self, phrase: &str, mood: Mood) -> Result<AudioData, DroidError> {
        self.get_audio_data(phrase, DroidType::Bd1, Some(mood))
    }

    /// Get Astromech raw audio data
    pub fn astro_audio(&self, phrase: &str) -> Result<AudioData, DroidError> {
        self.get_audio_data(phrase, DroidType::Astro, None)
    }

    /// Play a single WAV file
    fn play_wav(&self, path: &Path) -> Result<(), DroidError> {
        let (_stream, stream_handle) =
            OutputStream::try_default().map_err(|e| DroidError::PlaybackError(e.to_string()))?;
        let sink =
            Sink::try_new(&stream_handle).map_err(|e| DroidError::PlaybackError(e.to_string()))?;

        let file = File::open(path)?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| DroidError::PlaybackError(e.to_string()))?;
        sink.append(source);
        sink.sleep_until_end();

        Ok(())
    }

    /// Play multiple WAV files sequentially
    fn play_wav_sequence(&self, paths: &[PathBuf]) -> Result<(), DroidError> {
        let (_stream, stream_handle) =
            OutputStream::try_default().map_err(|e| DroidError::PlaybackError(e.to_string()))?;
        let sink =
            Sink::try_new(&stream_handle).map_err(|e| DroidError::PlaybackError(e.to_string()))?;

        for path in paths {
            if let Ok(file) = File::open(path) {
                if let Ok(source) = Decoder::new(BufReader::new(file)) {
                    sink.append(source);
                }
            }
        }

        sink.sleep_until_end();
        Ok(())
    }

    /// Make BD-1 speak a phrase with a specific mood
    pub fn bd1(&self, phrase: &str, mood: Mood) -> Result<(), DroidError> {
        let paths = self.build_letter_paths(phrase, DroidType::Bd1, Some(mood));
        if paths.is_empty() {
            return Ok(());
        }
        self.play_wav_sequence(&paths)
    }

    /// Generate random BD-1 sounds
    pub fn bd1_random(&self, length: usize, mood: Mood) -> Result<(), DroidError> {
        self.bd1(&generate_random_string(length), mood)
    }

    /// Make D-O speak a specific phrase
    pub fn do_(&self, phrase: &str) -> Result<(), DroidError> {
        let mut path = self.build_dir(DroidType::Do, None);
        let phrase_lower = phrase.to_lowercase();

        if phrase_lower == "random" {
            let entries: Vec<_> = WalkDir::new(&path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "wav")
                        .unwrap_or(false)
                })
                .collect();

            if let Some(entry) = entries.get(rand_idx(entries.len())) {
                self.play_wav(entry.path())?;
            }
        } else {
            path.push(format!("{}.wav", phrase_lower));
            if path.exists() {
                self.play_wav(&path)?;
            }
        }

        Ok(())
    }

    /// Make Astromech droid speak
    pub fn astro(&self, phrase: &str) -> Result<(), DroidError> {
        let paths = self.build_letter_paths(phrase, DroidType::Astro, None);
        if paths.is_empty() {
            return Ok(());
        }
        self.play_wav_sequence(&paths)
    }

    /// Generate random Astromech sounds
    pub fn astro_random(&self, length: usize) -> Result<(), DroidError> {
        self.astro(&generate_random_string(length))
    }

    /// Play a pre-recorded sound for a specific droid
    fn play_droid_sound(&self, droid: DroidType, sound_name: &str) -> Result<(), DroidError> {
        let mut path = self.build_dir(droid, None);
        path.push(sound_name);
        if path.exists() {
            self.play_wav(&path)?;
        }
        Ok(())
    }

    /// Play BB-8 pre-recorded sound
    pub fn bb8(&self, sound_name: &str) -> Result<(), DroidError> {
        self.play_droid_sound(DroidType::Bb8, sound_name)
    }

    /// Play Chopper pre-recorded sound
    pub fn chopper(&self, sound_name: &str) -> Result<(), DroidError> {
        self.play_droid_sound(DroidType::Chopper, sound_name)
    }

    /// Play Mouse droid sound
    pub fn mouse(&self, sound_name: &str) -> Result<(), DroidError> {
        self.play_droid_sound(DroidType::Mouse, sound_name)
    }

    /// Play Probe droid sound
    pub fn probe(&self, sound_name: &str) -> Result<(), DroidError> {
        self.play_droid_sound(DroidType::Probe, sound_name)
    }

    /// Play R2 unit sound
    pub fn r2(&self, sound_name: &str) -> Result<(), DroidError> {
        self.play_droid_sound(DroidType::R2, sound_name)
    }

    /// Generate a combined WAV file from a phrase and save to output path
    pub fn phrase_to_file(
        &self,
        phrase: &str,
        droid: DroidType,
        mood: Option<Mood>,
        output_path: &Path,
    ) -> Result<(), DroidError> {
        let paths = self.build_letter_paths(phrase, droid, mood);
        if paths.is_empty() {
            return Err(DroidError::SoundNotFound("No sound files found".into()));
        }
        self.combine_wav_files(&paths, output_path)
    }

    /// Generate a combined WAV file from BD-1 phrase and save to output path
    pub fn bd1_to_file(
        &self,
        phrase: &str,
        mood: Mood,
        output_path: &Path,
    ) -> Result<(), DroidError> {
        self.phrase_to_file(phrase, DroidType::Bd1, Some(mood), output_path)
    }

    /// Generate a combined WAV file from Astromech phrase and save to output path
    pub fn astro_to_file(&self, phrase: &str, output_path: &Path) -> Result<(), DroidError> {
        self.phrase_to_file(phrase, DroidType::Astro, None, output_path)
    }

    /// Combine multiple WAV files into a single WAV file
    fn combine_wav_files(
        &self,
        input_paths: &[PathBuf],
        output_path: &Path,
    ) -> Result<(), DroidError> {
        use hound::{SampleFormat, WavSpec, WavWriter};

        let mut all_samples: Vec<i16> = Vec::new();
        let mut sample_rate = 44100;
        let mut channels = 1;

        for path in input_paths {
            if let Ok(file) = File::open(path) {
                if let Ok(reader) = hound::WavReader::new(BufReader::new(file)) {
                    sample_rate = reader.spec().sample_rate;
                    channels = reader.spec().channels;
                    for s in reader.into_samples::<i16>().flatten() {
                        all_samples.push(s);
                    }
                }
            }
        }

        if all_samples.is_empty() {
            return Err(DroidError::SoundNotFound("No audio data".into()));
        }

        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(output_path, spec)
            .map_err(|e| DroidError::PlaybackError(e.to_string()))?;

        for sample in all_samples {
            writer
                .write_sample(sample)
                .map_err(|e| DroidError::PlaybackError(e.to_string()))?;
        }

        writer
            .finalize()
            .map_err(|e| DroidError::PlaybackError(e.to_string()))?;
        Ok(())
    }
}

/// Generate random string of specified length
fn generate_random_string(length: usize) -> String {
    let alphabet: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
    (0..length)
        .map(|_| alphabet[rand_idx(alphabet.len())])
        .collect()
}

/// Simple random index helper
fn rand_idx(max: usize) -> usize {
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

/// Setup sounds directory, downloading from npm if not present
///
/// This function checks if sounds directory exists and has valid content.
/// If not, it downloads the sounds from npm registry.
///
/// # Example
///
/// ```ignore
/// use droid_chatter::{DroidChatter, setup_sounds};
/// use std::path::Path;
///
/// // Auto download if needed
/// setup_sounds(Path::new("./sounds")).unwrap();
///
/// // Now use the sounds
/// let chatter = DroidChatter::new(Path::new("./sounds")).unwrap();
/// ```
pub fn setup_sounds(dest_dir: &Path) -> Result<(), DroidError> {
    // Check if sounds directory already exists with valid content
    let bd1_dir = dest_dir.join("bd1").join("happy");
    let astro_dir = dest_dir.join("astro");

    if bd1_dir.exists() && astro_dir.exists() {
        // Verify we have letter files
        let sample_file = bd1_dir.join("a.wav");
        if sample_file.exists() {
            return Ok(()); // Already setup
        }
    }

    const NPM_REGISTRY: &str =
        "https://registry.npmjs.org/node-droid-language/-/node-droid-language-1.0.2.tgz";

    std::fs::create_dir_all(dest_dir).map_err(DroidError::Io)?;

    println!("Downloading sounds from npm...");

    let response =
        reqwest::blocking::get(NPM_REGISTRY)
            .map_err(|e| DroidError::PlaybackError(format!("Download failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(DroidError::PlaybackError(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .map_err(|e| DroidError::PlaybackError(format!("Read bytes failed: {}", e)))?;

    println!("Extracting sounds...");

    let mut tar_reader = tar::Archive::new(flate2::read::GzDecoder::new(bytes.as_ref()));
    let dest_path = dest_dir.to_path_buf();

    for entry in tar_reader
        .entries()
        .map_err(|e| DroidError::PlaybackError(format!("Tar error: {}", e)))?
    {
        let mut entry =
            entry.map_err(|e| DroidError::PlaybackError(format!("Entry error: {}", e)))?;
        let path = entry
            .path()
            .map_err(|e| DroidError::PlaybackError(format!("Path error: {}", e)))?;

        let path_str = path.to_string_lossy();

        // Only extract sounds directory
        if !path_str.starts_with("package/sounds/") {
            continue;
        }

        let relative = path_str.strip_prefix("package/sounds/").unwrap_or(&path_str);
        let out_path = dest_path.join(relative);

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(DroidError::Io)?;
        }

        // Only extract regular files (not directories)
        if entry.header().entry_type().is_file() {
            entry.unpack(&out_path).map_err(|e| DroidError::PlaybackError(format!("Unpack error: {}", e)))?;
        }
    }

    println!("Sounds extracted to {:?}", dest_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_mock_sounds(tmpdir: &TempDir) -> PathBuf {
        let sounds = tmpdir.path().join("sounds");
        std::fs::create_dir_all(&sounds).unwrap();

        let bd1_happy = sounds.join("bd1").join("happy");
        std::fs::create_dir_all(&bd1_happy).unwrap();
        for letter in b'a'..=b'z' {
            std::fs::write(bd1_happy.join(format!("{}.wav", letter as char)), b"RIFF").unwrap();
        }

        let d_o = sounds.join("do");
        std::fs::create_dir_all(&d_o).unwrap();
        std::fs::write(d_o.join("hello1.wav"), b"RIFF").unwrap();
        std::fs::write(d_o.join("batterycharged.wav"), b"RIFF").unwrap();

        let astro = sounds.join("astro");
        std::fs::create_dir_all(&astro).unwrap();
        for letter in b'a'..=b'z' {
            std::fs::write(astro.join(format!("{}.wav", letter as char)), b"RIFF").unwrap();
        }

        sounds
    }

    #[test]
    fn test_generate_random_string() {
        let result = generate_random_string(10);
        assert_eq!(result.len(), 10);
        assert!(result.chars().all(|c| c.is_alphabetic()));
    }

    #[test]
    fn test_mood_as_str() {
        assert_eq!(Mood::Happy.as_str(), "happy");
        assert_eq!(Mood::Sad.as_str(), "sad");
        assert_eq!(Mood::Angry.as_str(), "angry");
    }

    #[test]
    fn test_droid_type_as_str() {
        assert_eq!(DroidType::Astro.as_str(), "astro");
        assert_eq!(DroidType::Bd1.as_str(), "bd1");
        assert_eq!(DroidType::Do.as_str(), "do");
        assert_eq!(DroidType::Bb8.as_str(), "bb8");
        assert_eq!(DroidType::Chopper.as_str(), "chopper");
        assert_eq!(DroidType::Mouse.as_str(), "mouse");
        assert_eq!(DroidType::Probe.as_str(), "probe");
        assert_eq!(DroidType::R2.as_str(), "r2");
    }

    #[test]
    fn test_get_available_phrases() {
        let tmpdir = TempDir::new().unwrap();
        let sounds = create_mock_sounds(&tmpdir);

        let phrases = get_available_phrases(&sounds, DroidType::Do);
        assert!(phrases.contains(&"hello1".to_string()));
        assert!(phrases.contains(&"batterycharged".to_string()));
    }

    #[test]
    fn test_new_with_invalid_dir() {
        let result = DroidChatter::new(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
