use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use indicatif::{ProgressBar, ProgressStyle};

pub struct Converter;

impl Converter {
    pub fn new() -> Self {
        Converter
    }

    fn ffmpeg_with_log(args: &[&str], log_file: &PathBuf) -> Result<(), String> {
        let mut log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        let mut cmd = Command::new("ffmpeg");
        cmd.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn ffmpeg: {}", e))?;

        if let Some(ref mut stdout) = child.stdout {
            std::io::copy(stdout, &mut log).ok();
        }
        if let Some(ref mut stderr) = child.stderr {
            std::io::copy(stderr, &mut log).ok();
        }

        let status = child.wait().map_err(|e| format!("Failed to wait for ffmpeg: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err("ffmpeg command failed, see log for details".to_string())
        }
    }

    pub fn convert_ogg_to_m4b(input: &PathBuf, output: &PathBuf, log_file: &PathBuf) -> Result<(), String> {
        let args = [
            "-i", input.to_str().unwrap(),
            "-c:a", "aac", "-b:a", "64k", "-vn",
            output.to_str().unwrap(),
        ];
        Self::ffmpeg_with_log(&args, log_file)
    }

    pub fn convert_mp3_to_m4b(input: &PathBuf, output: &PathBuf, log_file: &PathBuf) -> Result<(), String> {
        let args = [
            "-i", input.to_str().unwrap(),
            "-c:a", "aac", "-b:a", "64k", "-vn",
            output.to_str().unwrap(),
        ];
        Self::ffmpeg_with_log(&args, log_file)
    }

    /// Converts and merges media files into a single m4b file with chapters.
    /// If `chapter_titles` is provided, it will be used for chapter names; otherwise, filenames are used.
    pub fn convert_and_merge_to_m4b(
        &self,
        media_files: &[PathBuf],
        output_file: &str,
        chapter_titles: Option<&[String]>,
    ) -> Result<(), String> {
        let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let mut aac_files = Vec::new();
        let log_file = temp_dir.path().join("ffmpeg.log");

        // Progress bar setup
        let pb = ProgressBar::new(media_files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );

        // Convert all input files to AAC (.m4a) format
        for (i, input) in media_files.iter().enumerate() {
            let temp_aac = temp_dir.path().join(format!("{:04}.m4a", i));
            let ext = input.extension().and_then(|s| s.to_str()).unwrap_or("");
            pb.set_message(format!(
                "Processing: {}",
                input.display()
            ));
            let result = match ext {
                "ogg" => Self::convert_ogg_to_m4b(input, &temp_aac, &log_file),
                "mp3" => Self::convert_mp3_to_m4b(input, &temp_aac, &log_file),
                "wav" => Self::convert_mp3_to_m4b(input, &temp_aac, &log_file), // treat wav as mp3 for ffmpeg
                _ => Err(format!("Unsupported file format: {:?}", input)),
            };
            result?;
            aac_files.push(temp_aac);
            pb.inc(1);
        }
        pb.finish_with_message("All files converted.");

        // Create a file list for ffmpeg concat
        let concat_list = temp_dir.path().join("concat.txt");
        {
            let mut file = File::create(&concat_list).map_err(|e| e.to_string())?;
            for aac in &aac_files {
                writeln!(file, "file '{}'", aac.display()).map_err(|e| e.to_string())?;
            }
        }

        // Get durations for chapters
        let mut durations = Vec::new();
        for aac in &aac_files {
            let duration = get_duration_seconds(aac)?;
            durations.push(duration);
        }

        // Build ffmetadata file for chapters
        let ffmeta_path = temp_dir.path().join("chapters.ffmeta");
        {
            let mut ffmeta = BufWriter::new(File::create(&ffmeta_path).map_err(|e| e.to_string())?);
            writeln!(ffmeta, ";FFMETADATA1").map_err(|e| e.to_string())?;

            let mut start = 0.0;
            for (i, duration) in durations.iter().enumerate() {
                let end = start + duration;
                let title = chapter_titles
                    .and_then(|titles| titles.get(i))
                    .cloned()
                    .unwrap_or_else(|| {
                        aac_files
                            .get(i)
                            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
                            .unwrap_or_else(|| format!("Chapter {}", i + 1))
                    });
                writeln!(ffmeta, "[CHAPTER]").map_err(|e| e.to_string())?;
                writeln!(ffmeta, "TIMEBASE=1/1000").map_err(|e| e.to_string())?;
                writeln!(ffmeta, "START={}", (start * 1000.0) as u64).map_err(|e| e.to_string())?;
                writeln!(ffmeta, "END={}", (end * 1000.0) as u64).map_err(|e| e.to_string())?;
                writeln!(ffmeta, "title={}", title).map_err(|e| e.to_string())?;
                start = end;
            }
        }

        // Merge all AAC files into a single M4B with chapters
        println!("Merging files into final m4b with chapters...");
        let args = [
            "-f", "concat",
            "-safe", "0",
            "-i", concat_list.to_str().unwrap(),
            "-i", ffmeta_path.to_str().unwrap(),
            "-map_metadata", "1",
            "-c", "copy",
            output_file,
        ];
        Self::ffmpeg_with_log(&args, &log_file)?;

        println!("FFmpeg log written to: {}", log_file.display());
        Ok(())
    }
}

/// Helper to get duration in seconds of an audio file using ffprobe
fn get_duration_seconds(path: &PathBuf) -> Result<f64, String> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run ffprobe: {}", e))?;
    if !output.status.success() {
        return Err("ffprobe failed".to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().parse::<f64>().map_err(|e| format!("Failed to parse duration: {}", e))
}