use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use reqwest::Client;

extern crate serde;
extern crate serde_json;

const GET_CHARACTERS: &str = "
query ($page: Int, $perPage: Int, $search: String) {
  Page (page: $page, perPage: $perPage) {
    pageInfo {
      total
      currentPage
      lastPage
      hasNextPage
      perPage
    }
    characters (search: $search) {
      id
      name {
        full
        native
      }
      gender
      age
      dateOfBirth {
        month
        day
      }
    }
  }
}
";

fn parse_string_value<'de, D>(d: D) -> Result<String, D::Error> where D: Deserializer<'de> {
  Deserialize::deserialize(d)
    .map(|x: Option<_>| {
      x.unwrap_or("No value".to_string())
    })
}

fn parse_int_value<'de, D>(d: D) -> Result<i32, D::Error> where D: Deserializer<'de> {
  Deserialize::deserialize(d)
    .map(|x: Option<_>| {
      x.unwrap_or(0)
    })
}

pub struct Variables {
  pub page: i32,
  pub per_page: i32,
  pub search: String,
}

#[derive(Serialize, Deserialize)]
pub struct Name {
  #[serde(deserialize_with = "parse_string_value")]
  pub full: String,
  #[serde(deserialize_with = "parse_string_value")]
  pub native: String,
}

#[derive(Serialize, Deserialize)]
pub struct DateOfBirth {
  #[serde(deserialize_with = "parse_int_value")]
  pub month: i32,
  #[serde(deserialize_with = "parse_int_value")]
  pub day: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Character {
  pub id: i32,
  pub name: Name,
  #[serde(deserialize_with = "parse_string_value")]
  pub gender: String,
  #[serde(deserialize_with = "parse_string_value")]
  pub age: String,
  pub date_of_birth: DateOfBirth,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Data {
  pub page: Page,
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
  pub characters: Vec<Character>,
  pub page_info: PageInfo,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
  pub data: Data,
}

pub async fn get_characters(variables: Variables) -> Response {
  let client = Client::new();
  
  let json = json!({
    "query": GET_CHARACTERS,
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