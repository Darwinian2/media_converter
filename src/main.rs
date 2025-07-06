// This file is the entry point of the application. It initializes the application, sets up command-line argument parsing, and orchestrates the conversion process.

use std::env;
use std::path::PathBuf;
mod converter;
mod chapters;
mod utils;

#[tokio::main]
async fn main() {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  {} --rip-cd <output_folder> [device]", args[0]);
        eprintln!("  {} <input_folder>", args[0]);
        std::process::exit(1);
    }

    if args[1] == "--rip-cd" {
        let output_folder = args.get(2).expect("Output folder required");
        let device = args.get(3).map(|s| s.as_str()).unwrap_or("/dev/cdrom");

        println!("Ripping CD from device {}...", device);
        let wav_files = match utils::rip_cd_to_wav(output_folder, device) {
            Ok(files) => files,
            Err(e) => {
                eprintln!("CD rip failed: {}", e);
                std::process::exit(1);
            }
        };

        println!("Getting disc ID...");
        let disc_id = match utils::get_disc_id(device) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to get disc ID: {}", e);
                std::process::exit(1);
            }
        };

        println!("Fetching track names from MusicBrainz...");
        let track_names = utils::fetch_cd_track_names_musicbrainz(&disc_id).await;
        if let Some(names) = track_names {
            for (i, name) in names.iter().enumerate() {
                println!("Track {}: {}", i + 1, name);
            }
        } else {
            println!("Could not fetch track names from MusicBrainz.");
        }

        println!("Converting WAV files to m4b...");
        let output_file = format!("{output_folder}.m4b");
        let converter = converter::Converter::new();
        let wav_paths: Vec<std::path::PathBuf> = wav_files.iter().map(|f| std::path::PathBuf::from(f)).collect();
        // Provide a third argument as required by the method signature, e.g., None or an appropriate value
        if let Err(e) = converter.convert_and_merge_to_m4b(&wav_paths, &output_file, None) {
            eprintln!("Conversion failed: {}", e);
            std::process::exit(1);
        }
        println!("Successfully created {}", output_file);
        return;
    }

    // Default: convert from already ripped media (ogg, mp3, wav, etc)
    let input_folder = &args[1];
    let input_path = PathBuf::from(input_folder);
    let folder_name = input_path.file_name().unwrap_or_default().to_string_lossy();
    let output_file = format!("{}.m4b", folder_name);

    let media_files = utils::collect_media_files_recursive(input_folder, &["ogg", "mp3", "wav"]);
    if media_files.is_empty() {
        eprintln!("No media files found in the specified folder: {}", input_folder);
        std::process::exit(1);
    }

    let converter = converter::Converter::new();
    // Provide a third argument as required by the method signature, e.g., None or an appropriate value
    match converter.convert_and_merge_to_m4b(&media_files, &output_file, None) {
        Ok(_) => println!("Successfully created {}", output_file),
        Err(e) => eprintln!("Conversion failed: {}", e),
    }
}