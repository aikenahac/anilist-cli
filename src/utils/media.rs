use crate::api::media::Page;

pub fn print_media(page: Page) {
  println!("{}", "RESULTS:");
  for media in page.media {
    println!("{}", "----------------");
    println!("{}: {}", "ID", media.id);
    println!("{}: {}", "English title:", media.title.english);
    println!("{}: {}", "Native title:", media.title.native);
    println!("{}: {}", "Romaji title:", media.title.romaji);
  }
  println!("{}", "----------------\n");

  println!("{}", "Pagination:");
  println!("{}: {}", "Current page", page.page_info.current_page);
  println!("{}: {}", "Has next page", page.page_info.has_next_page);
  println!("{}: {}", "Last page", page.page_info.last_page);
  println!("{}: {}", "Results per page", page.page_info.per_page);
  println!("{}: {}", "Total results", page.page_info.total);
} 