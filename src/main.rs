use utils::media::print_media;
use utils::characters::print_characters;

mod utils {
  pub mod io;
  pub mod media;
  pub mod characters;
}

mod api {
  pub mod media;
  pub mod characters;
}

#[tokio::main]
async fn main() {
  println!("Search for:\n    [0] Anime/Manga\n    [1] Characters");
  let option: String = utils::io::read_line();

  if option == "0" {
    println!("\nEnter search query: ");
    let query: String = utils::io::read_line();

    let variables: api::media::Variables = api::media::Variables {
      page: 1,
      per_page: 10,
      search: query,
    };

    let response: api::media::Response = api::media::get_media(variables).await;
    let page: api::media::Page = response.data.page;

    print_media(page);
  } else if option == "1" {
    println!("\nEnter search query: ");
    let query: String = utils::io::read_line();

    let variables: api::characters::Variables = api::characters::Variables {
      page: 1,
      per_page: 10,
      search: query,
    };

    let response: api::characters::Response = api::characters::get_characters(variables).await;
    let page: api::characters::Page = response.data.page;

    print_characters(page);
  } else {
    println!("\nInvalid option.");
  }
}