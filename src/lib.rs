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
//!
//! ## Example
//!
//! ```ignore
//! use droid_chatter::{setup_sounds, DroidChatter, Mood};
//!
//! setup_sounds("./sounds").unwrap();
//! let chatter = DroidChatter::new("./sounds").unwrap();
//! chatter.bd1("hello", Mood::Happy).unwrap();
//! ```

pub mod audio;
pub mod chatter;
pub mod download;
pub mod droid;
pub mod error;
pub mod utils;

pub use audio::AudioData;
pub use chatter::DroidChatter;
pub use download::setup_sounds;
pub use droid::{DroidType, Mood};
pub use error::DroidError;
pub use utils::{generate_random_string, get_available_phrases};
