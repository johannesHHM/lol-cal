#![allow(dead_code)] // TODO: remove this later
use serde::Deserialize;
use serde::Serialize;

use reqwest::Client;

use crate::net::*;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Root {
    data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    leagues: Vec<League>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct League {
    pub name: String,
    pub slug: String,
    pub id: String,
    pub image: String,
    pub priority: i64,
    pub region: String,
}

const LEAGUES_URL: &str = "https://esports-api.lolesports.com/persisted/gw/getLeagues?hl=en-US";

pub async fn fetch_leagues(client: &Client) -> Result<Vec<League>, Error> {
    let response = client
        .get(LEAGUES_URL)
        .header(X_API_KEY_NAME, X_API_KEY_VALUE)
        .send()
        .await?;

    if response.status().is_success() {
        let api_response: Root = response.json().await?;
        return Ok(api_response.data.leagues);
    } else {
        return Err(Error::Request(response.status()));
    }
}
