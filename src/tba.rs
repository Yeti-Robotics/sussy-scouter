use lazy_static::lazy_static;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, StatusCode,
};
use serde::Deserialize;

lazy_static! {
    pub static ref TBA_SECRET: String = std::env::var("TBA_SECRET").unwrap();
    pub static ref CLIENT: Client = Client::builder()
        .default_headers(HeaderMap::from_iter([(
            HeaderName::from_static("x-tba-auth-key"),
            HeaderValue::from_static(&*TBA_SECRET)
        )]))
        .build()
        .unwrap();
}

pub const EVENT_KEY: &'static str = "2023joh";

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum CompLevel {
    #[serde(rename = "qm")]
    Qual,
    #[serde(rename = "ef")]
    /// Idk what this is
    Ef,
    #[serde(rename = "qf")]
    QuarterFinal,
    #[serde(rename = "sf")]
    SemiFinal,
    #[serde(rename = "f")]
    Final,
}

#[derive(Debug, Deserialize)]

pub enum WinningAlliance {
    #[serde(rename = "red")]
    Red,
    #[serde(rename = "blue")]
    Blue,
    #[serde(rename = "")]
    None,
}

#[derive(Debug, Deserialize)]
pub struct MatchSimple {
    pub key: String,
    pub comp_level: CompLevel,
    pub match_number: i32,
    /// Too lazy to make this type
    pub alliances: Option<serde_json::Value>,
    pub winning_alliance: Option<WinningAlliance>,
    pub event_key: String,
    pub time: Option<u64>,
    pub predicted_time: Option<u64>,
    pub actual_time: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum TBAError {
	#[error("Reqwest Error: {0:?}")]
	Reqwest(#[from] reqwest::Error),
	#[error("Bad status: {0}")]
	/// Lazy ah error handling
	Status(StatusCode)
}

pub async fn matches() -> Result<Vec<MatchSimple>, TBAError> {
	// Secret header is added by default from the config
    let res = CLIENT
        .get(format!("https://www.thebluealliance.com/api/v3/event/{EVENT_KEY}/matches/simple"))
        .send()
        .await?;

	if res.status().is_success() {
		Ok(res.json().await.unwrap())
	} else {
		Err(TBAError::Status(res.status()))
	}
}
