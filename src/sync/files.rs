use chrono::DateTime;
use std::{io::Write, path::PathBuf, time::SystemTime};

use indicatif::ProgressBar;
use serde::Deserialize;

pub fn sync_files() {
    let Some(access_token) = crate::sync::auth::get_access_token() else {
        println!("No access token found, please login");
        return;
    };
    let todo_files = crate::todo::get_all_files();

    println!("Using access token: {}", access_token);
    let uploaded = list_uploaded_files(&access_token).expect("Failed to list uploaded files");

    // transform into a map for easy lookup
    let uploaded_files: std::collections::HashMap<String, SystemTime> = uploaded
        .files
        .into_iter()
        .map(|f| {
            let parsed_date = parse_js_date(&f.last_modified);
            // println!("{}: {:?}", f.name, parsed_date);
            (f.name, parsed_date)
        })
        .collect();

    let modified_todo_files = todo_files
        .iter()
        .filter(|file| {
            let last_modified = get_last_modified_time(&file).expect("Failed to get last modified");
            if let Some(last_modified_uploaded) =
                uploaded_files.get(file.file_name().unwrap().to_str().unwrap())
            {
                if last_modified <= *last_modified_uploaded {
                    return false;
                }
            }
            true
        })
        .collect::<Vec<&PathBuf>>();

    let pb = ProgressBar::new(modified_todo_files.len() as u64);
    for file in modified_todo_files {
        upload_file(&file, &access_token).expect("Failed to upload file");
        pb.inc(1);
    }
    pb.finish_with_message("Upload complete.");

    let server_modified_files = todo_files
        .iter()
        .filter(|file| {
            let last_modified = get_last_modified_time(&file).expect("Failed to get last modified");
            if let Some(last_modified_uploaded) =
                uploaded_files.get(file.file_name().unwrap().to_str().unwrap())
            {
                if last_modified >= *last_modified_uploaded {
                    return false;
                }
                // println!(
                //     "Will download {:?} because {:?} < remote {:?}",
                //     file, last_modified, last_modified_uploaded
                // );
            }
            true
        })
        .collect::<Vec<&PathBuf>>();

    // TODO:
    // - more robust comparison of modified time ... currently always uploads / downloads because of upload time
    //  - consider content hash?
    // - handle deleted files
    // - handle new files on server
    // - parrallelize upload / download

    let pb = ProgressBar::new(server_modified_files.len() as u64);
    for file in server_modified_files {
        pb.inc(1);
        let downloaded = download_file(&file, &access_token).expect("Failed to download file");
        let mut editable_file = std::fs::File::create(file).expect("Failed to create file");
        editable_file
            .write(downloaded.as_bytes())
            .expect("Failed to write file");
    }
    pb.finish_with_message("Download complete.");
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
    name: String,
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
