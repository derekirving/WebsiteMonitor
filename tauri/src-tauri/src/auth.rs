use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::mpsc;
use tiny_http::{Response, Server};
use url::Url;
use base64::{Engine as _, engine::general_purpose};

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub token_type: String,
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

    println!("Code verifier {}. Code challenge {}", code_verifier, code_challenge);

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
            
            if let Some(code) = url.query_pairs()
                .find(|(key, _)| key == "code")
                .map(|(_, value)| value.into_owned()) {
                
                let html = "<html><body><h1>Authentication successful!</h1><p>You can close this window.</p></body></html>";
                let response = Response::from_string(html)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
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
) -> Result<TokenResponse, Box<dyn std::error::Error>> {
    let token_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        tenant_id
    );

    let params = [
        ("client_id", client_id),
        ("scope", "User.Read openid profile offline_access"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
        ("code_verifier", code_verifier),
    ];

    let client = reqwest::Client::new();
    let response = client
        .post(&token_url)
        .form(&params)
        .send()
        .await?;

    let token_response: TokenResponse = response.json().await?;
    Ok(token_response)
}