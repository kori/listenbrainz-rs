extern crate chrono;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::time::SystemTime;

// source: https://listenbrainz.readthedocs.io/en/latest/dev/api.html#constants
//
// Maximum overall listen size in bytes, to prevent egregious spamming.
pub const MAX_LISTEN_SIZE: u16 = 10240;
// The maximum number of listens returned in a single GET request.
pub const MAX_ITEMS_PER_GET: u8 = 100;
// The default number of listens returned in a single GET request.
pub const DEFAULT_ITEMS_PER_GET: u8 = 25;
// The maximum number of tags per listen.
pub const MAX_TAGS_PER_LISTEN: u8 = 50;
// The maximum length of a tag
pub const MAX_TAG_SIZE: u8 = 64;
// API main entrypoint
pub const API_ROOT_URL: &str = "https://api.listenbrainz.org";

#[derive(Serialize, Deserialize)]
pub struct Payload {
    #[serde(skip_serializing_if = "Value::is_null")]
    listened_at: Value,
    track_metadata: Track,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "listen_type")]
pub enum Listen {
    #[serde(rename = "single")]
    Single { payload: Vec<Payload> },
    #[serde(rename = "playing_now")]
    PlayingNow { payload: Payload },
}

pub enum SubmissionType {
    Single,
    PlayingNow,
}

#[derive(Serialize, Deserialize)]
pub struct Track {
    #[serde(rename = "artist_name")]
    artist: String,
    #[serde(rename = "track_name")]
    title: String,
    #[serde(rename = "release_name")]
    album: String,
}

// get current unix timestamp
fn unix_timestamp() -> u64 {
    return SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
}

// source: https://listenbrainz.readthedocs.io/en/latest/dev/api.html
// Listens should be submitted for tracks when the user has listened to
// half the track or 4 minutes of the track, whichever is lower. If the
// user hasn’t listened to 4 minutes or half the track, it doesn’t fully
// count as a listen and should not be submitted.
pub fn get_submission_time(length: u64) -> u64 {
    return (length / 2).min(240);
}

pub trait Submittable {
    fn format(&self, _: Track) -> String;
}

impl Submittable for SubmissionType {
    fn format(&self, t: Track) -> String {
        let submission = match &self {
            SubmissionType::Single => Listen::Single {
                payload: vec![Payload {
                    listened_at: serde_json::json!(unix_timestamp()),
                    track_metadata: t,
                }],
            },
            SubmissionType::PlayingNow => Listen::PlayingNow {
                payload: Payload {
                    listened_at: Value::Null,
                    track_metadata: t,
                },
            },
        };

        return match serde_json::to_string_pretty(&submission) {
            Ok(string) => string,
            Err(_) => String::from(""),
        };
    }
}

pub async fn submit(body: String) -> String {
    let client = reqwest::Client::new();
    let response = client.post(API_ROOT_URL).json(&body).send().await;

    return match response {
        Ok(r) => match r.text().await {
            Ok(t) => t,
            Err(_) => String::from(""),
        },
        Err(_) => String::from(""),
    };
}

fn main() {
    let t = Track {
        artist: String::from("rick astley"),
        title: String::from("never gonna give you up"),
        album: String::from("whenever you need somebody"),
    };

    println!("{}", SubmissionType::Single.format(t))
}
