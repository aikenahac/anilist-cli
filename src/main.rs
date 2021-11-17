use utils::media::print_media;

mod utils {
  pub mod io;
  pub mod media;
}

mod api {
  pub mod media;
}

#[tokio::main]
async fn main() {
  println!("Search for:\n    [0] Anime/Manga\n    [1] Characters");
  let option: String = utils::io::read_line();

  if option == "0" {
  } else if option == "1" {
    println!("{}", "Not yet implemented");
    return;
  } else {
    println!("{}", "Invalid option");
    return;
  }

  println!("\nEnter search query: ");
  let query: String = utils::io::read_line();

  let variables: api::media::Variables = api::media::Variables {
    page: 1,
    per_page: 10,
    search: query,
  };

  let response: api::media::Response = api::media::get_media(variables).await;
  let page: api::media::Page = response.data.page;

  if option == "0" {
    print_media(page);
  }
}