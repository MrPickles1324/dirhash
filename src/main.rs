use std::{
    env, eprintln,
    fs::{self},
    println,
    process::exit,
    time::Duration,
};

use anyhow::bail;
use chrono::Utc;
use indicatif::ProgressBar;
use sha256::{try_digest, Sha256Digest};
use tokio::{
    fs::File,
    io::AsyncWriteExt,
    sync::mpsc::{self, unbounded_channel, Receiver},
};
use walkdir::WalkDir;

const WRITE_BUF_TRASHOLD: usize = 256 * 1024 * 1024;

enum Message {
    Write(String),
    Drain,
}

async fn handle_write(mut rx: Receiver<Message>) {
    let mut buff: Vec<u8> = Vec::new();
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("hashes")
        .await
        .unwrap();
    while let Some(i) = rx.recv().await {
        match i {
            Message::Write(str) => buff.extend(str.as_bytes()),
            Message::Drain => file.write_all(buff.drain(..).as_slice()).await.unwrap(),
        }
        if buff.len() > WRITE_BUF_TRASHOLD {
            file.write_all(buff.drain(..).as_slice()).await.unwrap()
        }
    }
}

fn get_dir() -> anyhow::Result<String> {
    let dir_arg = env::args().nth(1);
    let the_dir;
    if let Some(d) = dir_arg {
        if fs::metadata(d.clone())?.is_dir() {
            the_dir = d.as_str().to_string();
        } else {
            bail!("specified path is not a directory");
        }
    } else {
        the_dir = "./".to_string();
    }
    Ok(the_dir)
}

#[tokio::main]
async fn main() {
    let dir = get_dir().unwrap_or_else(|e| {
        eprintln!("error: {e}");
        exit(1);
    });
    let (tx, rx) = mpsc::channel::<Message>(usize::MAX >> 3);
    tokio::spawn(async move {
        handle_write(rx).await;
    });
    for ent in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| f.path().is_file())
    {}
}

// async fn _yep() -> Result<(), Box<dyn std::error::Error>> {
//     let mut files: Vec<PathBuf> = Vec::new();
//     let mut size_to_check = 0;
//     let pb = ProgressBar::new_spinner();
//     pb.set_message("Gathering files...");
//
//     for ent in WalkDir::new(the_dir.clone())
//         .into_iter()
//         .filter_map(|e| e.ok())
//         .filter(|f| f.path().is_file())
//     {
//         pb.inc(1);
//         let file = fs::File::open(ent.path())
//             .map_err(|e| format!("Failed to open {}: {e}", ent.path().to_string_lossy()))?;
//         let meta = file.metadata().map_err(|e| {
//             format!(
//                 "Failed to get metadata for {}: {e}",
//                 ent.path().to_string_lossy()
//             )
//         })?;
//         size_to_check += meta.len();
//         files.push(ent.path().into());
//     }
//     files.sort();
//     pb.finish_with_message(format!(
//         "Calculating hashes for: {}",
//         humansize::format_size(size_to_check, humansize::DECIMAL)
//     ));
//
//     let pb = ProgressBar::new(files.len() as u64);
//     let mut hashes = Vec::new();
//
//     for p in files {
//         hashes.push((try_digest(p.as_path())?, p.to_string_lossy().into_owned()));
//         pb.inc(1);
//     }
//     pb.finish();
//
//     let mut hashes_str = hashes
//         .iter()
//         .map(|i| format!("{} {}", i.0, i.1))
//         .collect::<Vec<String>>()
//         .join("\n");
//
//     let hash_of_all_hashes = hashes
//         .into_iter()
//         .map(|i| i.0)
//         .collect::<Vec<String>>()
//         .join("\n")
//         .digest();
//
//     hashes_str.push_str("\n\n");
//     hashes_str.push_str(hash_of_all_hashes.as_str());
//
//     let mut hashes_file_p = Path::new("hashes.txt");
//
//     if hashes_file_p.exists() {
//         let date = Utc::now().to_string();
//         let f_name = format!("hashes{}.txt", date.as_str());
//         hashes_file_p = Path::new(f_name.as_str());
//         fs::write(hashes_file_p, hashes_str)?;
//         println!("Hashes writen in {}", hashes_file_p.to_str().unwrap());
//     } else {
//         fs::write(hashes_file_p, hashes_str)?;
//         println!("Hashes writen in {}", hashes_file_p.to_str().unwrap());
//     }
//     println!("hash of {the_dir} is: {hash_of_all_hashes}");
//     Ok(())
// }
