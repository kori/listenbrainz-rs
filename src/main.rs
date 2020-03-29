extern crate chrono;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use serde_derive;
use serde_json as json;

// use std::collections::HashMap;
use std::time::SystemTime;

// source: https://listenbrainz.readthedocs.io/en/latest/dev/api.html#constants

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
pub enum Payload {
    Single {
        listened_at: u64,
        track_metadata: Track,
    },
    PlayingNow {
        track_metadata: Track,
    },
}

#[derive(Serialize, Deserialize)]
pub enum Submission {
    #[serde(flatten)]
    Single {
        listen_type: String,
        payload: Vec<Payload>,
    },
    PlayingNow {
        listen_type: String,
        payload: Payload,
    },
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

pub enum SubmissionType {
    Single,
    PlayingNow,
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
fn get_submission_time(length: u64) -> u64 {
    return (length / 2).min(240);
}

fn fmt(st: SubmissionType, t: Track) -> String {
    let submission = match st {
        SubmissionType::Single => Submission::Single {
            listen_type: String::from("single"),
            payload: vec![Payload::Single {
                listened_at: unix_timestamp(),
                track_metadata: t,
            }],
        },
        SubmissionType::PlayingNow => Submission::PlayingNow {
            listen_type: String::from("playing_now"),
            payload: Payload::PlayingNow { track_metadata: t },
        },
    };

    return match json::to_string(&submission) {
        Ok(string) => string,
        Err(_) => String::from(""),
    };
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

    println!("{}", fmt(SubmissionType::PlayingNow, t))
}
