use utils::search::search;

mod utils {
  pub mod io;
  pub mod media;
  pub mod characters;
  pub mod search;
}

mod api {
  pub mod media;
  pub mod characters;
}

#[tokio::main]
async fn main() {
  search().await;
}