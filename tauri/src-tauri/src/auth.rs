use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::mpsc;
use tauri_plugin_keyring::KeyringExt;
use tiny_http::{Response, Server};
use url::Url;

#[derive(Serialize, Deserialize)]
pub struct StoredToken {
    pub token: TokenResponse,
    pub issued_at: i64, // unix seconds
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub token_type: String,
    pub id_token: Option<String>,
}

pub fn generate_pkce() -> (String, String) {
    let code_verifier: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(128)
        .map(char::from)
        .collect();

    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    // Replace encode_config with the new API
    let code_challenge = general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize());

    println!(
        "Code verifier {}. Code challenge {}",
        code_verifier, code_challenge
    );

    (code_verifier, code_challenge)
}

pub fn start_auth_server() -> (String, mpsc::Receiver<String>) {
    let (tx, rx) = mpsc::channel();

    let server = Server::http("127.0.0.1:0").unwrap();

    let port: u16 = {
        let addr_str = server.server_addr().to_string();
        match addr_str.parse::<std::net::SocketAddr>() {
            Ok(sock) => sock.port(),
            Err(_) => panic!("Unexpected listen address format: {}", addr_str),
        }
    };

    std::thread::spawn(move || {
        if let Ok(Some(request)) = server.recv_timeout(std::time::Duration::from_secs(300)) {
            let url_str = format!("http://localhost{}", request.url());
            let url = Url::parse(&url_str).unwrap();

            if let Some(code) = url
                .query_pairs()
                .find(|(key, _)| key == "code")
                .map(|(_, value)| value.into_owned())
            {
                let html = "<html><body><h1>Authentication successful!</h1><p>You can close this window.</p></body></html>";
                let response = Response::from_string(html).with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap(),
                );
                let _ = request.respond(response);

                tx.send(code).ok();
            }
        }
    });

    (format!("http://localhost:{}", port), rx)
}

pub async fn exchange_code_for_token(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
    client_id: &str,
    tenant_id: &str,
    app_handle: &tauri::AppHandle,
) -> Result<TokenResponse, Box<dyn std::error::Error + Send + Sync>> {
    let token_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        tenant_id
    );

    let params = [
        ("client_id", client_id),
        //("scope", "User.Read openid profile offline_access"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
        ("code_verifier", code_verifier),
    ];

    let client = reqwest::Client::new();
    let response = client.post(&token_url).form(&params).send().await?;
    let token_response: TokenResponse = response.json().await?;

    let token_json = serde_json::to_string(&token_response)?;
    let user = extract_user_from_id_token_or_os(&token_response)?;
    let service = app_handle.package_info().name.to_string();

    println!(
        "Setting service {} for user {} with json {}",
        service, user, token_json
    );

    // Persist the full token JSON to keyring
    if let Err(e) = save_token_to_keyring(&app_handle, &user, &token_response) {
        eprintln!("Warning: failed to save token to keyring: {}", e);
    }

    // Try to fetch user photo (best-effort). This does not change success of login.
    let _ = fetch_user_photo(&token_response.access_token).await;

    // if let Ok(Some(saved)) = app_handle
    //     .keyring()
    //     .get_password("tauri-plugin-keyring", &user)
    // {
    //     if let Ok(saved_token) = serde_json::from_str::<TokenResponse>(&saved) {
    //         // use saved_token.access_token / refresh_token / expires_in ...
    //     }
    // }

    Ok(token_response)
}

pub fn save_token_to_keyring(
    app_handle: &tauri::AppHandle,
    user: &str,
    token: &TokenResponse,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let service = app_handle.package_info().name.to_string();
    let stored = StoredToken {
        token: (*token).clone(),
        issued_at: Utc::now().timestamp(),
    };
    let json = serde_json::to_string(&stored)?;
    app_handle.keyring().set_password(&service, user, &json)?;
    // persist last_user entry for quick whoami lookup
    let last_user_key = format!("{}::last_user", &service);
    app_handle.keyring().set_password(&service, &last_user_key, user)?;
    println!("save_token_to_keyring: service={}, user={}, last_user_key={}", service, user, last_user_key);
    println!("save_token_to_keyring: stored json {}", json);
    Ok(())
}

pub fn load_token_from_keyring(
    app_handle: &tauri::AppHandle,
    user: &str,
) -> Result<Option<StoredToken>, Box<dyn std::error::Error + Send + Sync>> {
    let service = app_handle.package_info().name.to_string();
    if let Ok(Some(json)) = app_handle.keyring().get_password(&service, user) {
        let stored: StoredToken = serde_json::from_str(&json)?;
        return Ok(Some(stored));
    }
    Ok(None)
}

pub async fn ensure_valid_token(
    app_handle: tauri::AppHandle,
    user: &str,
    client_id: &str,
    tenant_id: &str,
    margin_seconds: i64,
) -> Result<TokenResponse, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(stored) = load_token_from_keyring(&app_handle, user)? {
        let expires_at = stored.issued_at + stored.token.expires_in;
        let now = Utc::now().timestamp();
        if now + margin_seconds >= expires_at {
            // need refresh
            if let Some(ref refresh_token) = stored.token.refresh_token {
                let refreshed = refresh_access_token(refresh_token, client_id, tenant_id, app_handle.clone()).await?;
                // refresh_access_token persists the new token (it calls set_password)
                return Ok(refreshed);
            } else {
                return Err("no refresh_token available".into());
            }
        } else {
            return Ok(stored.token);
        }
    }
    Err("no stored token found".into())
}

pub async fn refresh_access_token(
    refresh_token: &str,
    client_id: &str,
    tenant_id: &str,
    app_handle: tauri::AppHandle, // changed: borrow instead of owned
) -> Result<TokenResponse, Box<dyn std::error::Error + Send + Sync>> {
    let token_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        tenant_id
    );

    let params = [
        ("client_id", client_id),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];

    let client = reqwest::Client::new();
    let response = client.post(&token_url).form(&params).send().await?;
    let token_response: TokenResponse = response.json().await?;

    // update persisted token JSON
    let token_json = serde_json::to_string(&token_response)?;
    let user = extract_user_from_id_token_or_os(&token_response)?;
    let service = app_handle.package_info().name.to_string();
    app_handle
        .keyring()
        .set_password(&service, &user, &token_json)?;
    // update last_user
    let last_user_key = format!("{}::last_user", &service);
    app_handle.keyring().set_password(&service, &last_user_key, &user)?;

    Ok(token_response)
}

pub fn extract_user_from_id_token_or_os(
    tr: &TokenResponse,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(id_token) = &tr.id_token {
        if let Some(payload) = id_token.split('.').nth(1) {
            if let Ok(decoded) = general_purpose::URL_SAFE_NO_PAD.decode(payload) {
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&decoded) {
                    println!("json {}", json);
                    if let Some(v) = json
                        .get("preferred_username")
                        .or_else(|| json.get("upn"))
                        .or_else(|| json.get("email"))
                    {
                        if let Some(s) = v.as_str() {
                            return Ok(s.to_string());
                        }
                    }
                }
            }
        }
    }
    Ok(std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown".into()))
}

pub async fn fetch_user_photo(
    access_token: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://graph.microsoft.com/v1.0/me/photo/$value")
        .bearer_auth(access_token)
        .send()
        .await

        ?;

    if res.status() == reqwest::StatusCode::OK {
        let content_type = res
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "image/jpeg".to_string());

        let bytes = res.bytes().await?;
        let b64 = general_purpose::STANDARD.encode(&bytes);
        let data_url = format!("data:{};base64,{}", content_type, b64);
        return Ok(Some(data_url));
    }

    // If photo not found (404) or other non-success, treat as None (best-effort)
    Ok(None)
}
