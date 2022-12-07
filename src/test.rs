use indicatif::{ProgressBar, ProgressStyle};
use rand::prelude::*;
use std::path::Path;
use walkdir::WalkDir;

use crate::midi_file::MidiFile;

#[test]
fn test_parse() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files");
    let sample_size = 1000;
    let walker = WalkDir::new(test_dir).into_iter();
    let midi_files = walker
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().unwrap() == "mid")
        .collect::<Vec<_>>();

    let mut rng = thread_rng();

    let midi_files = midi_files
        .choose_multiple(&mut rng, sample_size as usize)
        .cloned()
        .collect::<Vec<_>>();

    let pb = ProgressBar::new(midi_files.len() as u64);
    let pb_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap()
        .progress_chars("##-");
    pb.set_style(pb_style);

    for file in midi_files {
        let path = file.path();
        let file_name = path
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        pb.set_message(file_name);

        let data = std::fs::read(path).unwrap();
        let (_, midi_file) = MidiFile::parse(&data).unwrap();
        let parsed_data = midi_file.to_bytes();
        assert_eq!(data, parsed_data);

        pb.inc(1);
    }

    pb.finish();
}
