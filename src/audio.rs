//! Audio utilities

/// Audio data structure for cpal playback
#[derive(Debug, Clone)]
pub struct AudioData {
    pub samples: Vec<i16>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl AudioData {
    pub fn new(samples: Vec<i16>, sample_rate: u32, channels: u16) -> Self {
        Self {
            samples,
            sample_rate,
            channels,
        }
    }

    pub fn frames(&self) -> usize {
        self.samples.len() / self.channels as usize
    }
}
