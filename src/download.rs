//! Download and setup sounds from npm

use crate::error::DroidError;
use std::path::Path;

const NPM_REGISTRY: &str =
    "https://registry.npmjs.org/node-droid-language/-/node-droid-language-1.0.2.tgz";

pub fn setup_sounds(dest_dir: &Path) -> Result<(), DroidError> {
    let bd1_dir = dest_dir.join("bd1").join("happy");
    let astro_dir = dest_dir.join("astro");

    if bd1_dir.exists() && astro_dir.exists() {
        let sample_file = bd1_dir.join("a.wav");
        if sample_file.exists() {
            return Ok(());
        }
    }

    std::fs::create_dir_all(dest_dir).map_err(DroidError::Io)?;

    println!("Downloading sounds from npm...");

    let response = reqwest::blocking::get(NPM_REGISTRY)
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

        if !path_str.starts_with("package/sounds/") {
            continue;
        }

        let relative = path_str
            .strip_prefix("package/sounds/")
            .unwrap_or(&path_str);
        let out_path = dest_path.join(relative);

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(DroidError::Io)?;
        }

        if entry.header().entry_type().is_file() {
            entry
                .unpack(&out_path)
                .map_err(|e| DroidError::PlaybackError(format!("Unpack error: {}", e)))?;
        }
    }

    println!("Sounds extracted to {:?}", dest_path);
    Ok(())
}
