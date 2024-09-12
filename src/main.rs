use fs::rename;
use lofty::config::WriteOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::read_from_path;
use lofty::tag::{ItemKey, Tag};
use std::env;
use std::ffi::OsStr;
use std::fs::{self};
use std::io;
use std::path::Path;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Provide the media directory as argument");
        std::process::exit(1);
    } else if args.len() > 2 {
        eprintln!("Too many arguments");
        std::process::exit(1);
    }

    let working_dir = &args[1];
    println!("Umlaut converter is starting.");
    visit_dirs(Path::new(working_dir))?;

    println!("Umlaut converter is finished.");
    Ok(())
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
                        "Couldn't rename the file {} to {}: {}",
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
            "Invalid file name",
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

fn visit_dirs(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        // Convert umplauts from directory names
        let renamed_dir = rename_file_or_dir(dir)?;
        // Read the directory contents
        for entry in fs::read_dir(renamed_dir)? {
            let path = entry?.path();
            if path.is_dir() {
                // If directory is found, visit recursively
                visit_dirs(&path)?;
            } else {
                let renamed_path = rename_file_or_dir(&path)?;
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
                            }
                        }
                    }
                    Err(error) => println!("Couldn't read the file: {}", error),
                };
            }
        }
    }
    Ok(())
}

fn convert_umlauts(src: String) -> String {
    src.replace('ä', "a")
        .replace("Ä", "A")
        .replace('ö', "o")
        .replace('Ö', "O")
}
