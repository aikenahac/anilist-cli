use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use reqwest::Client;

extern crate serde;
extern crate serde_json;


const GET_MEDIA: &str = "
query ($page: Int, $perPage: Int, $search: String) {
  Page (page: $page, perPage: $perPage) {
    pageInfo {
      total
      currentPage
      lastPage
      hasNextPage
      perPage
    }
    media (search: $search) {
      id
      title {
        english
        romaji
        native
      }
    }
  }
}
";

fn parse_title<'de, D>(d: D) -> Result<String, D::Error> where D: Deserializer<'de> {
  Deserialize::deserialize(d)
    .map(|x: Option<_>| {
      x.unwrap_or("No title".to_string())
    })
}

pub struct Variables {
  pub page: i32,
  pub per_page: i32,
  pub search: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Titles {
  #[serde(deserialize_with = "parse_title")]
  pub english: String,
  #[serde(deserialize_with = "parse_title")]
  pub romaji: String,
  #[serde(deserialize_with = "parse_title")]
  pub native: String,
}

#[derive(Serialize, Deserialize)]
pub struct Media {
  pub id: i32,
  pub title: Titles,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
  pub current_page: i32,
  pub has_next_page: bool,
  pub last_page: i32,
  pub per_page: i32,
  pub total: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
  pub media: Vec<Media>,
  pub page_info: PageInfo,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Data {
  pub page: Page,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
  pub data: Data,
}

pub async fn get_media(variables: Variables) -> Response {
  let client: Client = Client::new();

  let json = json!({
    "query": GET_MEDIA,
    "variables": {
      "page": variables.page,
      "perPage": variables.per_page,
      "search": variables.search,
    }
  });

  let res = client.post("https://graphql.anilist.co/")
    .header("Content-Type", "application/json")
    .header("Accept", "application/json")
    .body(json.to_string())
    .send()
    .await
    .unwrap()
    .text()
    .await;

  let result: Response = serde_json::from_str::<Response>(&res.unwrap_or_default()).unwrap();
  return result;
}

const GET_MEDIA_DETAILS: &str = "
query ($id: Int) {
  Media (id: $id) {
    id
    title {
      english
      romaji
      native
    }
    type
    format
    status
    description
    episodes
    chapters
    volumes
    averageScore
    popularity
    genres
  }
}
";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MediaDetails {
    pub id: i32,
    pub title: Titles,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub format: Option<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub episodes: Option<i32>,
    pub chapters: Option<i32>,
    pub volumes: Option<i32>,
    #[serde(rename = "averageScore")]
    pub average_score: Option<i32>,
    pub popularity: Option<i32>,
    pub genres: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MediaDetailsData {
    pub media: MediaDetails,
}

#[derive(Serialize, Deserialize)]
pub struct MediaDetailsResponse {
    pub data: MediaDetailsData,
}

pub async fn get_media_details(id: i32, token: &str) -> Result<MediaDetails, Box<dyn std::error::Error>> {
    let client = Client::new();

    let json = json!({
        "query": GET_MEDIA_DETAILS,
        "variables": {
            "id": id
        }
    });

    let response = client
        .post("https://graphql.anilist.co/")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&json)
        .send()
        .await?
        .text()
        .await?;

    let result: MediaDetailsResponse = serde_json::from_str(&response)?;
    Ok(result.data.media)
}