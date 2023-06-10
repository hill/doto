use serde_json::json;

pub fn login() {
    let challenge = new_challenge().expect("Failed to get challenge");

    // open browser with challenge
    open::that(format!(
        "{}/auth/login?challenge={}",
        super::constants::BASE_URL,
        challenge.challenge
    ))
    .expect("Failed to open browser");

    // poll to see if challenge has been accepted
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

fn complete_challenge(challenge: String) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(format!(
            "{}/auth/challenge/complete",
            super::constants::BASE_URL
        ))
        .json(&json!({ "challenge": challenge }))
        .send()?;

    Ok(())
}
