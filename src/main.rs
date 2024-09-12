use fs::rename;
use lofty::config::WriteOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::read_from_path;
use lofty::tag::{ItemKey, Tag};
use std::env;
use std::ffi::OsStr;
use std::fs::{self};
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;

#[allow(clippy::comparison_chain)]
fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Provide the media directory as argument");
        std::process::exit(1);
    } else if args.len() > 2 {
        eprintln!("Too many arguments");
        std::process::exit(1);
    }

    let working_dir = &args[1];

    print!(
        "Going to permanently edit records in directory {}, are you sure you want to continue? [y]/[n]\n==> ",
        working_dir
    );

    let _ = io::stdout().flush();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    match input.trim().chars().next() {
        Some('y') => {
            println!("Umlaut converter is starting.");
            match visit_dirs(Path::new(working_dir), 0) {
                Ok(edited_count) => {
                    println!(
                        "Umlaut converter is finished. Edited total of {} records.",
                        edited_count
                    );
                    Ok(())
                }
                Err(e) => {
                    eprintln!("The converter failed: {}", e);
                    Err(e)
                }
            }
        }
        _ => {
            println!("Aborting");
            std::process::exit(1);
        }
    }
}

fn rename_file_or_dir(file: &Path) -> io::Result<PathBuf> {
    if let Some(file_name) = file.file_name().and_then(OsStr::to_str) {
        let new_filename = convert_umlauts(file_name.to_string());

        if file_name != new_filename {
            let new_file_path = file.with_file_name(&new_filename);
            match rename(file, &new_file_path) {
                Ok(()) => Ok(new_file_path),
                Err(error) => {
                    println!(
                        "Couldn't rename from {} to {}: {}",
                        file.display(),
                        new_filename,
                        error
                    );
                    Err(error)
                }
            }
        } else {
            Ok(file.to_path_buf())
        }
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid record name",
        ))
    }
}

fn convert_tags(tag: &mut Tag, item_key: ItemKey) -> bool {
    if let Some(result) = tag.get_string(&item_key) {
        let converted_result = convert_umlauts(result.to_string());
        if result != converted_result {
            tag.insert_text(item_key, converted_result);
            return true;
        }
    }

    false
}

fn visit_dirs(dir: &Path, mut edited: i32) -> Result<i32, String> {
    if dir.is_dir() {
        // Convert umplauts from directory names
        let renamed_dir = rename_file_or_dir(dir).map_err(|e| e.to_string())?;
        // Read the directory contents
        for entry in fs::read_dir(renamed_dir).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();

            // If directory is found, traverse recursively
            if path.is_dir() {
                edited = visit_dirs(&path, edited)?;

            // If the file is in desired format, process it
            } else if path.display().to_string().contains(".mp3")
                || path.display().to_string().contains(".flac")
            {
                let renamed_path = rename_file_or_dir(&path).map_err(|e| e.to_string())?;
                match read_from_path(&renamed_path) {
                    Ok(mut tagged_file) => {
                        if let Some(tag) = tagged_file.primary_tag_mut() {
                            let mut changed = false;
                            changed |= convert_tags(tag, ItemKey::TrackArtist);
                            changed |= convert_tags(tag, ItemKey::TrackTitle);
                            changed |= convert_tags(tag, ItemKey::AlbumArtist);
                            changed |= convert_tags(tag, ItemKey::AlbumTitle);
                            changed |= convert_tags(tag, ItemKey::Genre);
                            if changed {
                                match tagged_file
                                    .save_to_path(&renamed_path, WriteOptions::default())
                                {
                                    Ok(_) => {}
                                    Err(error) => println!("Failed to save tags: {}", error),
                                }
                                edited += 1;
                            }
                        }
                    }
                    Err(error) => println!(
                        "Couldn't read the file {} because: {}",
                        renamed_path.display(),
                        error
                    ),
                };
            }
        }
    }
    Ok(edited)
}

fn convert_umlauts(src: String) -> String {
    src.replace('ä', "a")
        .replace("Ä", "A")
        .replace('ö', "o")
        .replace('Ö', "O")
}
