mod utils {
  pub mod tui;
}

mod api {
  pub mod media;
  pub mod login;
}

#[tokio::main]
async fn main() {
  // Authenticate before proceeding
  match api::login::get_valid_token().await {
    Ok(token) => {
      // Get user info
      match api::login::get_viewer_info(&token).await {
        Ok(viewer) => {
          // Run the TUI
          if let Err(e) = utils::tui::run_tui(viewer.name, token).await {
            eprintln!("TUI error: {}", e);
            std::process::exit(1);
          }
        }
        Err(e) => {
          eprintln!("Failed to get user info: {}", e);
          std::process::exit(1);
        }
      }
    }
    Err(e) => {
      eprintln!("Authentication failed: {}", e);
      eprintln!("\nPlease ensure:");
      eprintln!("1. You have created a .env file with your ANILIST_CLIENT_ID");
      eprintln!("2. Your client is configured at https://anilist.co/settings/developer");
      eprintln!("3. The redirect URL is set to: http://localhost:8080/callback");
      std::process::exit(1);
    }
  }
}