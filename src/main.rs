use std::{
    env, eprintln, fs,
    path::{Path, PathBuf},
    println,
    process::exit,
};

use chrono::Utc;
use indicatif::ProgressBar;
use sha256::{try_digest, Sha256Digest};
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir_arg = env::args().nth(1);
    let the_dir;
    if let Some(d) = dir_arg {
        if fs::metadata(d.clone())?.is_dir() {
            the_dir = d.as_str().to_string();
        } else {
            eprintln!("specified path is not a directory");
            exit(1);
        }
    } else {
        the_dir = "./".to_string();
    }

    let mut files: Vec<PathBuf> = Vec::new();
    let mut size_to_check = 0;
    let pb = ProgressBar::new_spinner();
    pb.set_message("Gathering files...");

    for ent in WalkDir::new(the_dir.clone())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| f.path().is_file())
    {
        pb.inc(1);
        let file = fs::File::open(ent.path())
            .map_err(|e| format!("Failed to open {}: {e}", ent.path().to_string_lossy()))?;
        let meta = file.metadata().map_err(|e| {
            format!(
                "Failed to get metadata for {}: {e}",
                ent.path().to_string_lossy()
            )
        })?;
        size_to_check += meta.len();
        files.push(ent.path().into());
    }
    files.sort();
    pb.finish_with_message(format!(
        "Calculating hashes for: {}",
        humansize::format_size(size_to_check, humansize::DECIMAL)
    ));

    let pb = ProgressBar::new(files.len() as u64);
    let mut hashes = Vec::new();

    for p in files {
        hashes.push((try_digest(p.as_path())?, p.to_string_lossy().into_owned()));
        pb.inc(1);
    }
    pb.finish();

    let mut hashes_str = hashes
        .iter()
        .map(|i| format!("{} {}", i.0, i.1))
        .collect::<Vec<String>>()
        .join("\n");

    let hash_of_all_hashes = hashes
        .into_iter()
        .map(|i| i.0)
        .collect::<Vec<String>>()
        .join("\n")
        .digest();

    hashes_str.push_str("\n\n");
    hashes_str.push_str(hash_of_all_hashes.as_str());

    let mut hashes_file_p = Path::new("hashes.txt");

    if hashes_file_p.exists() {
        let date = Utc::now().to_string();
        let f_name = format!("hashes{}.txt", date.as_str());
        hashes_file_p = Path::new(f_name.as_str());
        fs::write(hashes_file_p, hashes_str)?;
        println!("Hashes writen in {}", hashes_file_p.to_str().unwrap());
    } else {
        fs::write(hashes_file_p, hashes_str)?;
        println!("Hashes writen in {}", hashes_file_p.to_str().unwrap());
    }
    println!("hash of {the_dir} is: {hash_of_all_hashes}");
    Ok(())
}
