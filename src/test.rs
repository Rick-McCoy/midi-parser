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
        .filter(|e| {
            e.file_type().is_file()
                && match e.path().extension() {
                    Some(ext) => ext == "mid",
                    None => false,
                }
        })
        .collect::<Vec<_>>();

    let seed = [0xca; 32];
    let mut rng = StdRng::from_seed(seed);

    let midi_files = midi_files
        .choose_multiple(&mut rng, sample_size as usize)
        .cloned()
        .collect::<Vec<_>>();

    let pb = ProgressBar::new(midi_files.len() as u64);
    let pb_style = ProgressStyle::default_bar()
        .template(
            "{spinner:.green} [{elapsed_precise}] [{msg}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta_precise}) [{per_sec}]",
        )
        .unwrap()
        .progress_chars("##-");
    pb.set_style(pb_style);

    for file in midi_files {
        let path = file.path();
        let file_name_osstr = match path.file_name() {
            Some(s) => s,
            None => continue,
        };
        let file_name = match file_name_osstr.to_os_string().into_string() {
            Ok(s) => s,
            Err(_) => continue,
        };
        pb.set_message(file_name);

        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let midi_file = match MidiFile::parse(&data) {
            Ok((_, m)) => m,
            Err(_) => continue,
        };
        let parsed_data = midi_file.to_bytes();
        let reopened_midi_file = match MidiFile::parse(&parsed_data) {
            Ok((_, m)) => m,
            Err(_) => continue,
        };
        assert_eq!(midi_file.header, reopened_midi_file.header);
        assert_eq!(midi_file.tracks.len(), reopened_midi_file.tracks.len());
        for (track, reopened_track) in midi_file
            .tracks
            .iter()
            .zip(reopened_midi_file.tracks.iter())
        {
            assert_eq!(track.chunk_type, reopened_track.chunk_type);
            assert_eq!(track.data.len(), reopened_track.data.len());
            for (event, reopened_event) in track.data.iter().zip(reopened_track.data.iter()) {
                assert_eq!(event.delta_time, reopened_event.delta_time);
                assert_eq!(event.event, reopened_event.event);
            }
        }

        pb.inc(1);
    }

    pb.finish();
}
