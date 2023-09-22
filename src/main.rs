use base64::Engine;
use std::io::Write;
use std::str::FromStr;

use clap::Parser;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    text: String,

    #[arg(long, default_value = "false")]
    dump: bool,
}

use reqwest::header::HeaderValue;


use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub input: Input,
    pub voice: Voice,
    pub audio_config: AudioConfig,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Voice {
    pub language_code: String,
    pub name: String,
    pub ssml_gender: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioConfig {
    pub audio_encoding: String,
}

fn main() {
    let args = Args::parse();
    let text = args.text;
    if text.is_empty() {
        eprintln!("text is empty");
        std::process::exit(1);
    }
    let GOOGLE_APPLICATION_CREDENTIALS = std::env::var("GOOGLE_APPLICATION_CREDENTIALS");
    if GOOGLE_APPLICATION_CREDENTIALS.is_err() {
        eprintln!("GOOGLE_APPLICATION_CREDENTIALS env is empty");
        std::process::exit(2);
    }

    let tts_request = Request {
        input: Input { text },
        voice: Voice {
            language_code: "en-US".into(),
            name: "en-US-Standard-J".into(),
            ssml_gender: "MALE".into(),
        },
        audio_config: AudioConfig {
            audio_encoding: "MP3".into(),
        },
    };

    let mut headers = reqwest::header::HeaderMap::new();

    let ACCESS_TOKEN = get_access_token();
    let header0 = HeaderValue::from_str(&format!("Bearer {}", ACCESS_TOKEN)).unwrap();
    headers.insert("Authorization", header0);
    let header1 = HeaderValue::from_str("application/json; charset=utf-8").unwrap();
    headers.insert("Content-Type", header1);
    dbg!(&headers);
    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    let tts_request = serde_json::to_string(&tts_request).unwrap();
    let result = client
        .post("https://texttospeech.googleapis.com/v1/text:synthesize")
        .body(tts_request)
        .send();
    match result {
        Err(err) => {
            eprintln!("request error: {:?}", err);
            std::process::exit(3);
        }
        Ok(response) => {
            let response: ResponseType = response.json().unwrap();
            let audio_bytes = base64::engine::GeneralPurpose::new(
                &base64::alphabet::STANDARD,
                base64::engine::general_purpose::PAD,
            )
            .decode(response.audioContent)
            .unwrap();
            let mut file = tempfile::NamedTempFile::new().unwrap();
            let file_path = file.path().display().to_string();
            file.write(audio_bytes.as_slice())
                .expect("write file failed");
            file.flush().expect("flush file failed");

            match std::process::Command::new("mpv")
                .args(&[file_path.clone()])
                .output()
            {
                Err(err) => {
                    eprintln!("play error: {:?}", err);
                }
                Ok(output) => {
                    eprintln!("play output: {:?}", output);
                }
            }
        }
    }
}

fn get_access_token() -> String {
    std::process::Command::new("gcloud")
        .args(["auth", "application-default", "print-access-token"])
        .output()
        .unwrap()
        .stdout
        .into_iter()
        .take_while(|&c| c != b'\n')
        .collect::<Vec<_>>()
        .into_iter()
        .map(|c| c as char)
        .collect::<String>()
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseType {
    audioContent: String,
}
