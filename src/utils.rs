use std::fs;
use std::path::PathBuf;
use std::io;

pub fn collect_media_files_recursive(folder: &str, exts: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(folder) {
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_media_files_recursive(path.to_str().unwrap(), exts));
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if exts.contains(&ext) {
                    files.push(path);
                }
            }
        }
    }
    files
}

/// Fetch the disc ID using `cd-discid` (must be installed)
pub fn get_disc_id(device: &str) -> Result<String, String> {
    use std::process::Command;

    let output = Command::new("cd-discid")
        .arg(device)
        .output()
        .map_err(|e| format!("Failed to run cd-discid: {}", e))?;
    if !output.status.success() {
        return Err("cd-discid failed".to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let disc_id = stdout.split_whitespace().next().unwrap_or("").to_string();
    if disc_id.is_empty() {
        Err("Could not parse disc ID".to_string())
    } else {
        Ok(disc_id)
    }
}

pub async fn fetch_cd_track_names_musicbrainz(disc_id: &str) -> Option<Vec<String>> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Medium {
        tracks: Vec<Track>,
    }
    #[derive(Debug, Deserialize)]
    struct Track {
        title: String,
    }
    #[derive(Debug, Deserialize)]
    struct Release {
        media: Vec<Medium>,
    }
    #[derive(Debug, Deserialize)]
    struct MBResponse {
        releases: Vec<Release>,
    }

    let url = format!(
        "https://musicbrainz.org/ws/2/discid/{}?inc=recordings+artists&fmt=json",
        disc_id
    );
    let client = Client::new();
    let resp = client.get(&url)
        .header("User-Agent", "media-converter/0.1 ( https://github.com/yourname/media-converter )")
        .send().await.ok()?;

    let mb: MBResponse = resp.json().await.ok()?;
    let release = mb.releases.get(0)?;
    let medium = release.media.get(0)?;
    let titles = medium.tracks.iter().map(|t| t.title.clone()).collect();
    Some(titles)
}

pub fn rip_cd_to_wav(output_folder: &str, _device: &str) -> Result<Vec<String>, io::Error> {
    // Placeholder implementation: simulate ripping by creating dummy WAV files
    fs::create_dir_all(output_folder)?;
    let mut files = Vec::new();
    for i in 1..=10 {
        let file_path = format!("{}/track{:02}.wav", output_folder, i);
        fs::write(&file_path, b"dummy wav data")?;
        files.push(file_path);
    }
    Ok(files)
}