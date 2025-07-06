# Media Converter

This is a Rust application that converts media files from OGG and MP3 formats to a single M4B file with chapter support, even when your input is organized into subfolders (such as `cd1`, `cd2`, etc).

## Features

- Convert OGG and MP3 files (including those in subfolders) to a single M4B file
- Automatically orders and merges tracks from subfolders
- Support for chapter metadata in the resulting M4B file

## Requirements

- Rust (1.50 or later)
- FFmpeg (for media conversion)

## External Dependencies

This application requires several external tools to be installed on your system:

- **ffmpeg**: For audio conversion and merging.
- **ffprobe**: For extracting audio durations (usually included with ffmpeg).
- **cdparanoia**: For ripping audio tracks from CDs.
- **cd-discid**: For obtaining the disc ID for MusicBrainz metadata lookup.

### Install on Ubuntu/Debian

```bash
sudo apt update
sudo apt install ffmpeg cdparanoia cd-discid
```

### Install on Fedora

```bash
sudo dnf install ffmpeg cdparanoia cd-discid
```

### Install on Arch Linux

```bash
sudo pacman -S ffmpeg cdparanoia cd-discid
```

### Install on openSUSE

```bash
sudo zypper install ffmpeg cdparanoia cd-discid
```

> **Note:**  

> - `ffprobe` is included with the `ffmpeg` package on all major distributions.
> - You must have a working CD drive and permissions to access it for CD ripping features.

---

## Installation

1. Clone the repository:

   ```bash
   git clone <repository-url>
   cd media_converter
   ```

2. Build the project:

   ```bash
   cargo build --release
   ```

## Usage

To convert a folder (with possible subfolders) of media files, run:

```bash
cargo run -- <input_folder> <output_file.m4b>
```

- `<input_folder>`: Path to the folder containing your OGG/MP3 files. All subfolders (e.g., `cd1`, `cd2`, etc) will be processed in order.
- `<output_file.m4b>`: Desired output file name.

## Chapter Support

Chapters are automatically generated for each track, using file and folder names for chapter titles. If your input files are tagged with chapter information, this will be included in the output.

## Contributing

Feel free to submit issues or pull requests for improvements or bug fixes.

## License

This project is licensed under the MIT License. See the LICENSE file for details

## TODO

1. Rip directly from Audio CD, and sending the scanned track data to the online catalogue to get chapter details, etc.

