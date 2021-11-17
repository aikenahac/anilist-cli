use crate::api::characters::Page;

pub fn print_characters(page: Page) {
  println!("{}", "RESULTS:");
  for character in page.characters {
    println!("{}", "----------------");
    println!("{}: {}", "ID", character.id);
    println!("{}: {}", "Name", character.name.full);
    println!("{}: {}", "Native name", character.name.native);
    println!("{}: {}", "Gender", character.gender);
    println!("{}: {}.{}", "Birthday", character.date_of_birth.day, character.date_of_birth.month);
  }
  println!("{}", "----------------\n");

  println!("{}", "Pagination:");
  println!("{}: {}", "Current page", page.page_info.current_page);
  println!("{}: {}", "Has next page", page.page_info.has_next_page);
  println!("{}: {}", "Last page", page.page_info.last_page);
  println!("{}: {}", "Results per page", page.page_info.per_page);
  println!("{}: {}", "Total results", page.page_info.total);
} 