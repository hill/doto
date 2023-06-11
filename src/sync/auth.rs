use indicatif::ProgressBar;
use reqwest::StatusCode;
use serde_json::{json, to_writer_pretty};
use std::{fs::File, path::PathBuf};

pub fn login() {
    let challenge = new_challenge().expect("Failed to get challenge");

    // open browser with challenge
    open::that(format!(
        "{}/auth/login?challenge={}",
        super::constants::BASE_URL,
        challenge.challenge
    ))
    .expect("Failed to open browser");

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb.set_message("Please login in your browser...");
    pb.set_position(10);

    // poll to see if challenge has been accepted
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let challenge_result =
            complete_challenge(challenge.challenge.clone()).expect("Failed to complete challenge");

        match challenge_result {
            ChallengeResult::Success(res) => {
                pb.finish_with_message("Done");
                pb.disable_steady_tick();
                println!("Successfully logged in!");
                println!("Your access token is: {}", res.access_token);
                save_access_token(res.access_token);
                break;
            }
            ChallengeResult::Error(res) => match res {
                ChallengeError::AlreadyCompleted => {
                    println!("The challenge has already been completed, please login again.");
                    break;
                }
                ChallengeError::NotFound => {
                    println!("Challenge not found. Please login again.");
                    break;
                }
                ChallengeError::Expired => {
                    println!("Challenge expired. Please login again.");
                    break;
                }
                ChallengeError::NotClaimed => {
                    // do nothing ... keep polling
                }
            },
        }
    }
    pb.disable_steady_tick();
}

#[derive(serde::Deserialize, Debug)]
struct ChallengeResponse {
    challenge: String,
}

fn new_challenge() -> Result<ChallengeResponse, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let res: ChallengeResponse = client
        .post(format!("{}/challenge", super::constants::BASE_URL))
        .send()?
        .json()?;

    Ok(res)
}

#[derive(serde::Deserialize, Debug)]
struct CompleteChallengeResponse {
    access_token: String,
}

enum ChallengeError {
    AlreadyCompleted,
    NotFound,
    NotClaimed,
    Expired,
}

#[derive(serde::Deserialize, Debug)]
struct CompleteChallengeErrorResponse {
    error: String,
}

enum ChallengeResult {
    Success(CompleteChallengeResponse),
    Error(ChallengeError),
}

fn complete_challenge(challenge: String) -> Result<ChallengeResult, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(format!("{}/challenge/complete", super::constants::BASE_URL))
        .json(&json!({ "challenge": challenge }))
        .send()?;

    match res.status() {
        StatusCode::OK => {
            let res: CompleteChallengeResponse = res.json()?;
            Ok(ChallengeResult::Success(res))
        }
        StatusCode::BAD_REQUEST => {
            let res: CompleteChallengeErrorResponse = res.json()?;
            let error = match res.error.as_str() {
                "ALREADY_COMPLETED" => ChallengeError::AlreadyCompleted,
                "NOT_FOUND" => ChallengeError::NotFound,
                "NOT_CLAIMED" => ChallengeError::NotClaimed,
                "EXPIRED" => ChallengeError::Expired,
                _ => panic!("Unexpected status code"),
            };
            Ok(ChallengeResult::Error(error))
        }
        _ => panic!("Unexpected status code"),
    }
}

fn save_access_token(token: String) {
    // save the token to a config file in ~/.config/doto.json
    // let mut config_path = dirs::home_dir().expect("Could not determine home directory");
    if let Some(home_path) = dirs::home_dir() {
        let mut config_path = PathBuf::from(home_path);
        config_path.push(".config/doto.json");
        println!("Saving config to {:?}", config_path);
        let mut config_file = File::create(config_path).expect("Failed to create config file");

        let config = json!({
          "access_token": token,
        });
        to_writer_pretty(&mut config_file, &config).expect("Could not write to config file");
    }
}

pub fn get_access_token() -> Option<String> {
    if let Some(home_path) = dirs::home_dir() {
        let mut config_path = PathBuf::from(home_path);
        config_path.push(".config/doto.json");
        if config_path.exists() {
            let config_file = File::open(config_path).expect("Failed to open config file");
            let config: serde_json::Value =
                serde_json::from_reader(config_file).expect("Failed to parse config file");
            let access_token = config["access_token"].as_str().unwrap();
            Some(access_token.to_string())
        } else {
            None
        }
    } else {
        None
    }
}
