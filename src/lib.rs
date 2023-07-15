mod pages;
use std::str::FromStr;

use sierra_keygen::{ChallengeType, DeviceGeneration};
use worker::*;

#[event(fetch)]
async fn main(mut req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    if req.method() == Method::Get {
        let page = pages::Main {
            device_generation: None,
            challenge_type: None,
            challenge: None,
            challenge_response: None,
            error_msg: None,
        };

        return Response::from_html(page.to_string());
    } else if req.method() == Method::Post {
        let params = req.form_data().await?;

        let mut error_msg: Option<String> = None;

        // captcha
        // TODO

        // device_generation
        let device_generation = match params.get("device_generation") {
            Some(FormEntry::Field(val)) => match DeviceGeneration::from_str(&val) {
                Ok(gen) => Some(gen),
                Err(_) => None,
            },
            _ => None,
        };
        if device_generation.is_none() && error_msg.is_none() {
            error_msg = Some("Invalid device generation".to_string());
        }

        // challenge_type
        let challenge_type = match params.get("challenge_type") {
            Some(FormEntry::Field(val)) => match ChallengeType::from_str(&val) {
                Ok(chal) => Some(chal),
                Err(_) => None,
            },
            _ => None,
        };
        if challenge_type.is_none() && error_msg.is_none() {
            error_msg = Some("Invalid challenge type".to_string());
        }

        // challenge
        let challenge_str = match params.get("challenge") {
            Some(FormEntry::Field(val)) => Some(val),
            _ => None,
        };
        let challenge = match &challenge_str {
            Some(chal) => match hex::decode(chal) {
                Ok(chal) => Some(chal),
                Err(_) => None,
            },
            None => None,
        };
        if challenge.is_none() && error_msg.is_none() {
            error_msg = Some("Invalid challenge".to_string());
        }

        // should we return early?
        if error_msg.is_some() {
            let page = pages::Main {
                device_generation,
                challenge_type,
                challenge: challenge_str,
                challenge_response: None,
                error_msg,
            };
            return Response::from_html(page.to_string());
        }

        // otherwise, generate key
        let challenge_response = sierra_keygen::generate_code(
            device_generation.clone().unwrap(),
            challenge_type.clone().unwrap(),
            &challenge.unwrap(),
        );

        if challenge_response.is_err() {
            let page = pages::Main {
                device_generation,
                challenge_type,
                challenge: challenge_str,
                challenge_response: None,
                error_msg: Some(format!("{:?}", challenge_response.err().unwrap())),
            };
            return Response::from_html(page.to_string());
        }

        let page = pages::Main {
            device_generation,
            challenge_type,
            challenge: challenge_str,
            challenge_response: challenge_response.ok(),
            error_msg: None,
        };

        return Response::from_html(page.to_string());
    } else {
        return Response::error("Method not allowed", 405);
    }
}
