mod captcha;
mod log;
mod pages;
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

use base64::Engine as _;
use log::LogLevel;
use sierra_keygen::{ChallengeType, DeviceGeneration};
use worker::*;

fn parse_auth_header(auth: &str) -> Option<(String, String)> {
    if !auth.starts_with("Basic ") {
        return None;
    }

    Some(auth)
        .map(|s| s.trim_start_matches("Basic "))
        .and_then(|s| {
            base64::engine::general_purpose::STANDARD
                .decode(s.as_bytes())
                .ok()
        })
        .and_then(|s| String::from_utf8(s).ok())
        .and_then(|s| {
            s.split_once(':')
                .and_then(|(u, p)| Some((u.to_string(), p.to_string())))
        })
}

#[event(fetch)]
async fn main(mut req: Request, env: Env, ctx: Context) -> Result<Response> {
    if req.path() != "/" {
        return Response::error("Not Found", 404);
    }

    let logs: Arc<Mutex<Vec<log::LogMessage>>> = Arc::new(Mutex::new(vec![]));
    let logflare_api_key = match env.secret("LOGFLARE_API_KEY") {
        Ok(val) => Some(val.to_string()),
        Err(_) => None,
    };
    let logflare_source_id = match env.var("LOGFLARE_SOURCE_ID") {
        Ok(val) => Some(val.to_string()),
        Err(_) => None,
    };
    let logging_enabled = logflare_api_key.is_some() && logflare_source_id.is_some();

    let log = |level: LogLevel, msg: String| {
        match level {
            LogLevel::Error => console_error!("{}", msg),
            LogLevel::Warn => console_warn!("{}", msg),
            LogLevel::Info => console_log!("{}", msg),
            LogLevel::Debug => console_debug!("{}", msg),
        }
        // give up if we can't get a lock
        let _ = logs
            .lock()
            .and_then(|mut l| Ok(l.push(log::make_log(level, msg))));
    };

    let post_logs = || {
        let logs = logs.clone();
        ctx.wait_until(async move {
            if !logging_enabled {
                return;
            }
            // give up if we can't get a lock
            let logs = logs.lock();
            if logs.is_ok() && logs.as_ref().unwrap().len() > 0 {
                log::log(
                    logflare_api_key.unwrap().as_str(),
                    logflare_source_id.unwrap().as_str(),
                    logs.unwrap().clone(),
                )
                .await;
            }
        });
    };

    let authed_user = req
        .headers()
        .get("Authorization")
        .unwrap()
        .and_then(|s| parse_auth_header(s.as_ref()))
        .and_then(|(username, password)| {
            if !username.chars().all(|c| c.is_ascii_alphanumeric()) {
                return None; // invalid username -- should be alphanumeric
            }

            let expected_password =
                match env.secret(format!("BASIC_AUTH_USER_{}", username.to_lowercase()).as_str()) {
                    Ok(val) => Ok(val.to_string()),
                    Err(_) => Err(()),
                };

            if expected_password.is_ok_and(|pw| pw.len() > 0 && pw == password) {
                Some(username)
            } else {
                None
            }
        });
    if authed_user.is_none() {
        let mut headers = Headers::new();
        let _ = headers.set("WWW-Authenticate", "Basic");
        post_logs();
        return Ok(Response::error("Unauthorized", 401)?.with_headers(headers));
    }

    let hcaptcha_secret = match env.secret("HCAPTCHA_SECRET") {
        Ok(val) => Some(val.to_string()),
        Err(_) => None,
    };
    let hcaptcha_sitekey = if hcaptcha_secret.is_some() {
        match env.var("HCAPTCHA_SITEKEY") {
            Ok(val) => Some(val.to_string()),
            Err(_) => None,
        }
    } else {
        None
    };

    if req.method() == Method::Get {
        let page = pages::Main {
            device_generation: None,
            challenge_type: None,
            challenge: None,
            challenge_response: None,
            hcaptcha_sitekey,
            error_msg: None,
        };

        post_logs();
        return Response::from_html(page.to_string());
    } else if req.method() == Method::Post {
        let params = req.form_data().await?;

        let mut error_msg: Option<String> = None;

        // captcha
        let h_captcha_response = match params.get("h-captcha-response") {
            Some(FormEntry::Field(val)) => Some(val),
            _ => None,
        };
        if h_captcha_response.is_some() {
            let captcha_valid = match captcha::verify_captcha(
                h_captcha_response.as_ref().unwrap(),
                hcaptcha_secret.as_ref().unwrap(),
                hcaptcha_sitekey.as_ref().unwrap(),
            )
            .await
            {
                Some(val) => val,
                None => false,
            };
            if !captcha_valid {
                error_msg = Some("Invalid CAPTCHA".to_string());
            }
        } else {
            error_msg = Some("Invalid CAPTCHA".to_string());
        }

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
            Some(FormEntry::Field(val)) => Some(val.trim().to_string()),
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
                hcaptcha_sitekey,
                error_msg,
            };
            post_logs();
            return Response::from_html(page.to_string());
        }

        log(
            LogLevel::Info,
            format!(
                "{} generating code for {} {} {}",
                authed_user.unwrap(),
                device_generation.clone().unwrap(),
                challenge_type.clone().unwrap(),
                challenge_str.clone().unwrap()
            ),
        );

        // otherwise, generate key
        let challenge_response = sierra_keygen::generate_code(
            device_generation.clone().unwrap(),
            challenge_type.clone().unwrap(),
            &challenge.unwrap(),
        );

        if challenge_response.is_err() {
            let err_str = format!("{:?}", challenge_response.err().unwrap());
            let page = pages::Main {
                device_generation,
                challenge_type,
                challenge: challenge_str,
                challenge_response: None,
                hcaptcha_sitekey,
                error_msg: Some(format!("Error: {}", err_str)),
            };
            log(
                LogLevel::Error,
                format!("got error while generating challenge: {}", err_str),
            );
            post_logs();
            return Response::from_html(page.to_string());
        }

        let page = pages::Main {
            device_generation,
            challenge_type,
            challenge: challenge_str,
            challenge_response: challenge_response.ok(),
            hcaptcha_sitekey,
            error_msg: None,
        };

        post_logs();
        return Response::from_html(page.to_string());
    } else {
        post_logs();
        return Response::error("Method not allowed", 405);
    }
}
