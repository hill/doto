use chrono::DateTime;
use std::{
    io::Write,
    path::{self, PathBuf},
    time::SystemTime,
};

use indicatif::ProgressBar;
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub fn sync_files() {
    let Some(access_token) = crate::sync::auth::get_access_token() else {
        println!("No access token found, please login");
        return;
    };
    let todo_files = crate::todo::get_all_files();

    let uploaded = list_uploaded_files(&access_token).expect("Failed to list uploaded files");

    // transform into a map for easy lookup
    let uploaded_files: std::collections::HashMap<String, (String, SystemTime)> = uploaded
        .files
        .into_iter()
        .map(|f| (f.name, (f.hash, parse_js_date(&f.last_modified))))
        .collect();

    let modified_todo_files = todo_files
        .iter()
        .filter(|file| {
            let last_modified = get_last_modified_time(&file).expect("Failed to get last modified");
            if let Some(uploaded_file) =
                uploaded_files.get(file.file_name().unwrap().to_str().unwrap())
            {
                if hash_file(&file) == *uploaded_file.0 {
                    return false;
                }

                // Server version is more recent
                if last_modified < uploaded_file.1 {
                    return false;
                }

                println!("Local version more recent: {:?}", file);
            }
            true
        })
        .map(|f| f.to_path_buf())
        .collect::<Vec<PathBuf>>();

    let mut files_to_download = Vec::new();
    let todo_dir = path::PathBuf::from(crate::util::get_doto_path());
    for (filename, (hash, _)) in uploaded_files.iter() {
        let file_path = todo_dir.join(filename);
        if !modified_todo_files.contains(&file_path) {
            if !file_path.exists() || hash_file(&file_path) != *hash {
                files_to_download.push(file_path.clone());
                println!("Server file more recent: {:?}", file_path)
            }
        }
    }

    // TODO:
    // - more robust comparison of modified time ... currently always uploads / downloads because of upload time
    //  - consider content hash?
    // - handle deleted files
    // - handle new files on server
    // - parrallelize upload / download

    let total_files: u64 = (modified_todo_files.len() + files_to_download.len()) as u64;

    if total_files == 0 {
        println!("Up to date.");
        return;
    }

    let pb = ProgressBar::new(total_files);
    for file in &modified_todo_files {
        upload_file(file, &access_token).expect("Failed to upload file");
        pb.inc(1);
    }

    for file in files_to_download {
        pb.inc(1);
        let downloaded = download_file(&file, &access_token).expect("Failed to download file");
        let mut editable_file = std::fs::File::create(file).expect("Failed to create file");
        editable_file
            .write(downloaded.as_bytes())
            .expect("Failed to write file");
    }
    pb.finish_with_message("Sync complete.");
}

fn upload_file(file_path: &PathBuf, access_token: &String) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();

    let form = reqwest::blocking::multipart::Form::new()
        .file("file", file_path)
        .expect("Failed to read file");

    client
        .post(&format!("{}/sync/file", super::constants::BASE_URL))
        .bearer_auth(access_token)
        .multipart(form)
        .send()?;

    Ok(())
}

fn get_last_modified_time(file: &PathBuf) -> Result<SystemTime, std::io::Error> {
    let metadata = std::fs::metadata(file)?;
    let modified_time = metadata.modified()?;
    Ok(modified_time)
}

#[derive(Deserialize, Debug)]
struct ListUploadedFilesResponse {
    files: Vec<UploadedFile>,
}

#[derive(Deserialize, Debug)]
struct UploadedFile {
    name: String,
    last_modified: String,
    hash: String,
}

fn list_uploaded_files(access_token: &String) -> Result<ListUploadedFilesResponse, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let res: ListUploadedFilesResponse = client
        .get(&format!("{}/sync/files", super::constants::BASE_URL))
        .bearer_auth(access_token)
        .send()?
        .json()?;

    Ok(res) // TODO: handle errors
}

#[derive(Deserialize, Debug)]
struct DownloadedFile {
    content: String,
}

fn download_file(file: &PathBuf, access_token: &String) -> Result<String, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let res: DownloadedFile = client
        .get(&format!(
            "{}/sync/download/{}",
            super::constants::BASE_URL,
            file.file_name().unwrap().to_str().unwrap()
        ))
        .bearer_auth(access_token)
        .send()?
        .json()?;
    Ok(res.content)
}

fn parse_js_date(js_date: &String) -> SystemTime {
    let dt = DateTime::parse_from_rfc3339(js_date).unwrap();
    let ts = dt.timestamp();
    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(ts as u64)
}

fn hash_file(file_path: &PathBuf) -> String {
    let mut hasher = Sha256::new();

    let file_contents = std::fs::read_to_string(file_path)
        .expect(format!("Could not read file contents: {:?}", file_path).as_str());
    hasher.update(file_contents);

    let result = hasher.finalize();

    return result
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
}
