use droid_chatter::{setup_sounds, DroidChatter, Mood};
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sounds_path = manifest_dir.join("sounds");
    let output_dir = manifest_dir.join("output");

    std::fs::create_dir_all(&output_dir).ok();

    println!("BD-1 Demo - Droid Chatter Library");
    println!("================================\n");

    // Auto download sounds if needed
    println!("Checking sounds directory...");
    setup_sounds(&sounds_path).expect("Failed to setup sounds");
    println!();

    let chatter = DroidChatter::new(&sounds_path).expect("Failed to create DroidChatter");

    let output_path = output_dir.join("bd1_hello_happy.wav");
    println!("Generating BD-1 'hello i am bd1' in happy mood...");
    match chatter.bd1_to_file("hello i am bd1", Mood::Happy, &output_path) {
        Ok(_) => println!("Saved: {:?}", output_path),
        Err(e) => println!("Error: {}", e),
    }

    let output_path = output_dir.join("astro_r2d2.wav");
    println!("Generating Astromech 'r2d2'...");
    match chatter.astro_to_file("r2d2", &output_path) {
        Ok(_) => println!("Saved: {:?}", output_path),
        Err(e) => println!("Error: {}", e),
    }

    println!("\nDone!");
}
