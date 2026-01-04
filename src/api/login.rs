use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use axum::{
    Router,
    routing::get,
    response::Html,
    extract::State,
};
use reqwest::Client;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenData {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saved_at: Option<i64>,
}

impl TokenData {
    pub fn is_expired(&self) -> bool {
        if let Some(saved_at) = self.saved_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            // Consider token expired if it's within 1 day of expiration
            let expires_at = saved_at + self.expires_in;
            now >= (expires_at - 86400)
        } else {
            // If we don't know when it was saved, assume it's expired
            true
        }
    }
}

fn get_token_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let config_dir = PathBuf::from(home).join(".config").join("anilist-cli");

    fs::create_dir_all(&config_dir).ok();

    config_dir.join("token.json")
}

pub fn save_token(token: &TokenData) -> Result<(), Box<dyn std::error::Error>> {
    let mut token_with_timestamp = token.clone();
    token_with_timestamp.saved_at = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64
    );

    let token_path = get_token_path();
    let json = serde_json::to_string_pretty(&token_with_timestamp)?;
    fs::write(token_path, json)?;
    Ok(())
}

pub fn load_token() -> Option<TokenData> {
    let token_path = get_token_path();
    if !token_path.exists() {
        return None;
    }

    let content = fs::read_to_string(token_path).ok()?;
    serde_json::from_str(&content).ok()
}

pub async fn validate_token(token: &str) -> bool {
    let client = Client::new();

    // Use AniList's Viewer query to check if token is valid
    let query = r#"
        query {
            Viewer {
                id
                name
            }
        }
    "#;

    let json = serde_json::json!({
        "query": query
    });

    let response = client
        .post("https://graphql.anilist.co/")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&json)
        .send()
        .await;

    match response {
        Ok(res) => res.status().is_success(),
        Err(_) => false,
    }
}

#[derive(Clone)]
struct AppState {
    token: Arc<Mutex<Option<TokenData>>>,
}

async fn callback_handler(State(_state): State<AppState>) -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>AniList Authentication</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background-color: #0b1622;
            color: #ffffff;
        }
        .container {
            text-align: center;
            padding: 2rem;
            background-color: #152232;
            border-radius: 8px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
        }
        .success {
            color: #4caf50;
            font-size: 1.5rem;
            margin-bottom: 1rem;
        }
        .error {
            color: #f44336;
            font-size: 1.5rem;
            margin-bottom: 1rem;
        }
    </style>
</head>
<body>
    <div class="container">
        <div id="message">Processing authentication...</div>
    </div>
    <script>
        const messageDiv = document.getElementById('message');

        // Extract token from URL fragment
        const hash = window.location.hash.substring(1);
        const params = new URLSearchParams(hash);
        const accessToken = params.get('access_token');
        const tokenType = params.get('token_type');
        const expiresIn = params.get('expires_in');

        if (accessToken) {
            // Send token to server
            const tokenData = {
                access_token: accessToken,
                token_type: tokenType || 'Bearer',
                expires_in: parseInt(expiresIn) || 31536000
            };

            fetch('/save-token', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(tokenData)
            })
            .then(response => {
                if (response.ok) {
                    messageDiv.className = 'success';
                    messageDiv.textContent = '✓ Authentication successful! You can close this window.';
                } else {
                    throw new Error('Failed to save token');
                }
            })
            .catch(error => {
                messageDiv.className = 'error';
                messageDiv.textContent = '✗ Authentication failed. Please try again.';
                console.error('Error:', error);
            });
        } else {
            messageDiv.className = 'error';
            messageDiv.textContent = '✗ No access token found in URL. Please try again.';
        }
    </script>
</body>
</html>
    "#)
}

async fn save_token_handler(
    State(state): State<AppState>,
    axum::extract::Json(token_data): axum::extract::Json<TokenData>,
) -> axum::http::StatusCode {
    let mut token = state.token.lock().await;
    *token = Some(token_data.clone());

    axum::http::StatusCode::OK
}

pub async fn authenticate() -> Result<TokenData, Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let client_id = std::env::var("ANILIST_CLIENT_ID")
        .map_err(|_| "ANILIST_CLIENT_ID not found in environment. Please create a .env file with your client ID.")?;

    let auth_url = format!(
        "https://anilist.co/api/v2/oauth/authorize?client_id={}&response_type=token",
        client_id
    );

    println!("\nOpening browser for authentication...");
    println!("If the browser doesn't open automatically, visit this URL:");
    println!("{}\n", auth_url);

    if let Err(e) = open::that(&auth_url) {
        eprintln!("Failed to open browser: {}", e);
        println!("Please open the URL manually.");
    }

    let state = AppState {
        token: Arc::new(Mutex::new(None)),
    };

    let app = Router::new()
        .route("/callback", get(callback_handler))
        .route("/save-token", axum::routing::post(save_token_handler))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    println!("Waiting for authentication callback on http://localhost:8080/callback...\n");

    // Run the server in a background task
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await
    });

    // Wait for token to be received (with timeout)
    let token_data = loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let token = state.token.lock().await;
        if let Some(token_data) = token.as_ref() {
            break token_data.clone();
        }
    };

    server_handle.abort();

    save_token(&token_data)?;

    println!("✓ Authentication successful! Token saved.");

    Ok(token_data)
}

pub async fn get_valid_token() -> Result<String, Box<dyn std::error::Error>> {
    // Try to load existing token
    if let Some(token_data) = load_token() {
        if !token_data.is_expired() {
            if validate_token(&token_data.access_token).await {
                println!("Using existing authentication token.");
                return Ok(token_data.access_token);
            } else {
                println!("Existing token is invalid.");
            }
        } else {
            println!("Existing token has expired.");
        }
    }

    // No valid token exists, authenticate
    println!("Authentication required.");
    let token_data = authenticate().await?;
    Ok(token_data.access_token)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ViewerData {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ViewerResponse {
    viewer: ViewerData,
}

#[derive(Serialize, Deserialize, Debug)]
struct ViewerApiResponse {
    data: ViewerResponse,
}

pub async fn get_viewer_info(token: &str) -> Result<ViewerData, Box<dyn std::error::Error>> {
    let client = Client::new();

    let query = r#"
        query {
            Viewer {
                id
                name
            }
        }
    "#;

    let json = serde_json::json!({
        "query": query
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

    let result: ViewerApiResponse = serde_json::from_str(&response)?;
    Ok(result.data.viewer)
}
