//! Core droid chatter implementation

use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::audio::AudioData;
use crate::droid::{DroidType, Mood};
use crate::error::DroidError;
use crate::utils::{generate_random_string, rand_idx};

pub struct DroidChatter<'a> {
    sounds_dir: &'a Path,
}

impl<'a> DroidChatter<'a> {
    pub fn new(sounds_dir: &'a Path) -> Result<Self, DroidError> {
        if !sounds_dir.exists() {
            return Err(DroidError::NoSoundsDir);
        }
        Ok(Self { sounds_dir })
    }

    fn build_dir(&self, droid: DroidType, mood: Option<Mood>) -> PathBuf {
        let mut path = self.sounds_dir.to_path_buf();
        path.push(droid.as_str());
        if let Some(m) = mood {
            path.push(m.as_str());
        }
        path
    }

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

    fn read_audio_from_paths(&self, paths: &[PathBuf]) -> Result<AudioData, DroidError> {
        let mut all_samples: Vec<i16> = Vec::new();
        let mut sample_rate = 44100u32;
        let mut channels = 1u16;

        for path in paths {
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

    pub fn bd1(&self, phrase: &str, mood: Mood) -> Result<(), DroidError> {
        let paths = self.build_letter_paths(phrase, DroidType::Bd1, Some(mood));
        if paths.is_empty() {
            return Ok(());
        }
        self.play_wav_sequence(&paths)
    }

    pub fn bd1_random(&self, length: usize, mood: Mood) -> Result<(), DroidError> {
        self.bd1(&generate_random_string(length), mood)
    }

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

    pub fn astro(&self, phrase: &str) -> Result<(), DroidError> {
        let paths = self.build_letter_paths(phrase, DroidType::Astro, None);
        if paths.is_empty() {
            return Ok(());
        }
        self.play_wav_sequence(&paths)
    }

    pub fn astro_random(&self, length: usize) -> Result<(), DroidError> {
        self.astro(&generate_random_string(length))
    }

    fn play_droid_sound(&self, droid: DroidType, sound_name: &str) -> Result<(), DroidError> {
        let mut path = self.build_dir(droid, None);
        path.push(sound_name);
        if path.exists() {
            self.play_wav(&path)?;
        }
        Ok(())
    }

    pub fn bb8(&self, sound_name: &str) -> Result<(), DroidError> {
        self.play_droid_sound(DroidType::Bb8, sound_name)
    }

    pub fn chopper(&self, sound_name: &str) -> Result<(), DroidError> {
        self.play_droid_sound(DroidType::Chopper, sound_name)
    }

    pub fn mouse(&self, sound_name: &str) -> Result<(), DroidError> {
        self.play_droid_sound(DroidType::Mouse, sound_name)
    }

    pub fn probe(&self, sound_name: &str) -> Result<(), DroidError> {
        self.play_droid_sound(DroidType::Probe, sound_name)
    }

    pub fn r2(&self, sound_name: &str) -> Result<(), DroidError> {
        self.play_droid_sound(DroidType::R2, sound_name)
    }

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
        self.read_audio_from_paths(&paths)
    }

    pub fn bd1_audio(&self, phrase: &str, mood: Mood) -> Result<AudioData, DroidError> {
        self.get_audio_data(phrase, DroidType::Bd1, Some(mood))
    }

    pub fn astro_audio(&self, phrase: &str) -> Result<AudioData, DroidError> {
        self.get_audio_data(phrase, DroidType::Astro, None)
    }

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

    pub fn bd1_to_file(
        &self,
        phrase: &str,
        mood: Mood,
        output_path: &Path,
    ) -> Result<(), DroidError> {
        self.phrase_to_file(phrase, DroidType::Bd1, Some(mood), output_path)
    }

    pub fn astro_to_file(&self, phrase: &str, output_path: &Path) -> Result<(), DroidError> {
        self.phrase_to_file(phrase, DroidType::Astro, None, output_path)
    }

    fn combine_wav_files(
        &self,
        input_paths: &[PathBuf],
        output_path: &Path,
    ) -> Result<(), DroidError> {
        use hound::{SampleFormat, WavSpec, WavWriter};

        let audio = self.read_audio_from_paths(input_paths)?;

        let spec = WavSpec {
            channels: audio.channels,
            sample_rate: audio.sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(output_path, spec)
            .map_err(|e| DroidError::PlaybackError(e.to_string()))?;

        for sample in audio.samples {
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
