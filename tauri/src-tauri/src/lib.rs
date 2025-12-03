// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, Arc};
use tokio::sync::watch;
use tauri::{
    async_runtime,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tokio::time::{interval, Duration};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_keyring::KeyringExt;

mod auth;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Website {
    url: String,
    is_down: bool,
    last_checked: String,
    notification_cleared: bool,
}

struct AppState {
    websites: Mutex<Vec<Website>>,
    tray: TrayIcon,
    refresher_tx: Mutex<Option<watch::Sender<bool>>>,
}

#[tauri::command]
async fn login(
    client_id: String,
    tenant_id: String,
    app_handle: AppHandle
) -> Result<serde_json::Value, String> {

    println!("Logging in: {}", client_id);
    let (code_verifier, code_challenge) = auth::generate_pkce();
    let (redirect_uri, rx) = auth::start_auth_server();

    let auth_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope=User.Read%20openid%20profile%20offline_access&code_challenge={}&code_challenge_method=S256",
        tenant_id, client_id, 
        urlencoding::encode(&redirect_uri),
        code_challenge
    );

    println!("Auth Url {}", auth_url);

    webbrowser::open(&auth_url).map_err(|e| e.to_string())?;

    // Wait for callback
    let code = rx.recv_timeout(std::time::Duration::from_secs(300))
        .map_err(|_| "Authentication timeout")?;

    // Exchange code for token
    let app_handle_clone = app_handle.clone();
    let token = auth::exchange_code_for_token(
        &code,
        &code_verifier,
        &redirect_uri,
        &client_id,
        &tenant_id,
        &app_handle_clone
    )
    .await
    .map_err(|e| e.to_string())?;

    // After successful login persist token and start a background refresher
    let user = Arc::new(auth::extract_user_from_id_token_or_os(&token).unwrap_or_else(|_| "unknown".to_string()));
    let ah = app_handle.clone();
    let client_id_clone = client_id.clone();
    let tenant_id_clone = tenant_id.clone();
    // share user with background refresher using Arc (cheap clone of pointer)
    let user_for_refresher = Arc::clone(&user);

    // Create a watch channel to allow cancelling the refresher
    let (tx, mut rx) = watch::channel(false);
    // store sender in app state so logout can cancel
    let state = app_handle.state::<AppState>();
    *state.refresher_tx.lock().unwrap() = Some(tx.clone());

    async_runtime::spawn(async move {
        // initial attempt to compute sleep until expiry
        loop {
            println!("Refresher running");
            // Try to load stored token and compute sleep until expiry
            if let Ok(Some(stored)) = auth::load_token_from_keyring(&ah, &*user_for_refresher) {
                let expires_at = stored.issued_at + stored.token.expires_in;
                let now = Utc::now().timestamp();
                // sleep until 60 seconds before expiry, or at most 5 minutes
                let sleep_secs = if expires_at > now + 60 {
                    (expires_at - now - 60) as u64
                } else {
                    300u64
                };
                // wait either for cancel or timeout
                let sleep = tokio::time::sleep(Duration::from_secs(sleep_secs));
                tokio::select! {
                    _ = rx.changed() => {
                        // cancelled when value becomes true
                        if *rx.borrow() {
                            break;
                        }
                    }
                    _ = sleep => {
                        let _ = auth::ensure_valid_token(ah.clone(), &*user_for_refresher, &client_id_clone, &tenant_id_clone, 60).await;
                        // loop and recompute next sleep
                    }
                }
            } else {
                // no token stored yet; wait a short while before retrying
                let sleep = tokio::time::sleep(Duration::from_secs(60));
                tokio::select! {
                    _ = rx.changed() => {
                        if *rx.borrow() { break; }
                    }
                    _ = sleep => continue,
                }
            }
        }
    });

    println!("Returning refresh token {}", token.access_token);
    // Merge token fields and add a top-level `user` property so frontend can persist username
    match serde_json::to_value(&token) {
        Ok(mut v) => {
            if let serde_json::Value::Object(ref mut map) = v {
                map.insert("user".to_string(), serde_json::Value::String(user.as_ref().clone()));
                return Ok(serde_json::Value::Object(map.clone()));
            }
            Ok(v)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn get_access_token(
    user: String,
    client_id: String,
    tenant_id: String,
    app_handle: AppHandle,
) -> Result<String, String> {
    match auth::ensure_valid_token(app_handle.clone(), &user, &client_id, &tenant_id, 60).await {
        Ok(token_resp) => Ok(token_resp.access_token),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn whoami(
    client_id: String,
    tenant_id: String,
    app_handle: AppHandle,
) -> Result<serde_json::Value, String> {
    // Read the last_user entry (persisted when saving tokens)
    let service = app_handle.package_info().name.to_string();
    let last_user_key = format!("{}::last_user", &service);
    if let Ok(Some(last_user)) = app_handle.keyring().get_password(&service, &last_user_key) {
        println!("whoami: found last_user key = {}", last_user);
        // Try to ensure token is valid (attempt refresh immediately)
        match auth::ensure_valid_token(app_handle.clone(), &last_user, &client_id, &tenant_id, 0).await {
            Ok(_) => {
                println!("whoami: ensure_valid_token succeeded for user {}", last_user);
                return Ok(serde_json::json!({"user": last_user.clone(), "authenticated": true}));
            }
            Err(e) => {
                println!("whoami: ensure_valid_token failed for user {}: {}", last_user, e);
                // Refresh failed — fall back to checking stored token expiry directly
                if let Ok(Some(stored)) = auth::load_token_from_keyring(&app_handle, &last_user) {
                    let now = chrono::Utc::now().timestamp();
                    let expires_at = stored.issued_at + stored.token.expires_in;
                    println!("whoami: stored token for {} expires_at={}, now={}", last_user, expires_at, now);
                    if now < expires_at {
                        println!("whoami: stored token still valid for user {}", last_user);
                        return Ok(serde_json::json!({"user": last_user.clone(), "authenticated": true}));
                    }
                } else {
                    println!("whoami: no stored token found for user {}", last_user);
                }
                return Ok(serde_json::json!({"user": last_user.clone(), "authenticated": false}));
            }
        }
    }

    Ok(serde_json::json!({"user": "", "authenticated": false}))
}

#[tauri::command]
fn clear_last_user(app_handle: AppHandle) -> Result<(), String> {
    let service = app_handle.package_info().name.to_string();
    let last_user_key = format!("{}::last_user", &service);
    match app_handle.keyring().delete_password(&service, &last_user_key) {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn greet(name: &str, app_handle: AppHandle) -> String {
    let _ = app_handle
        .notification()
        .builder()
        .title("Website Monitor")
        .body(format!("{} has been greeted", name))
        .show();

    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn check_websites(
    state: tauri::State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let websites_clone = state.websites.lock().unwrap().clone();
    match do_check_websites(websites_clone, app_handle).await {
        Ok((updated_websites, message)) => {
            *state.websites.lock().unwrap() = updated_websites;
            Ok(message)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
async fn fetch_protected(
    api_url: String,
    user: String,
    client_id: String,
    tenant_id: String,
    app_handle: AppHandle,
) -> Result<String, String> {
    // try once, on 401 refresh and retry once
    let access = auth::ensure_valid_token(app_handle.clone(), &user, &client_id, &tenant_id, 60).await.map_err(|e| e.to_string())?;
    let client = reqwest::Client::new();
    let res = client.get(&api_url).bearer_auth(&access.access_token).send().await.map_err(|e| e.to_string())?;
    if res.status() == 401 {
        // force refresh and retry
        let refreshed = auth::ensure_valid_token(app_handle.clone(), &user, &client_id, &tenant_id, 0).await.map_err(|e| e.to_string())?;
        let res2 = client.get(&api_url).bearer_auth(&refreshed.access_token).send().await.map_err(|e| e.to_string())?;
        let text = res2.text().await.map_err(|e| e.to_string())?;
        return Ok(text);
    }
    let text = res.text().await.map_err(|e| e.to_string())?;
    Ok(text)
}

#[tauri::command]
fn logout(user: String, app_handle: AppHandle) -> Result<(), String> {
    // clear stored keyring entry and cancel refresher if present
    let service = app_handle.package_info().name.to_string();
    if let Err(e) = app_handle.keyring().delete_password(&service, &user) {
        eprintln!("failed to remove keyring entry: {}", e);
    }
    // remove persisted last_user entry if it matches this user
    let last_user_key = format!("{}::last_user", &service);
    if let Ok(Some(last)) = app_handle.keyring().get_password(&service, &last_user_key) {
        if last == user {
            if let Err(e) = app_handle.keyring().delete_password(&service, &last_user_key) {
                eprintln!("failed to remove last_user key: {}", e);
            }
        }
    }
    // no filesystem fallback to remove; keyring entry already deleted above
    let state = app_handle.state::<AppState>();
    if let Some(tx) = state.refresher_tx.lock().unwrap().take() {
        let _ = tx.send(true);
    }
    Ok(())
}

async fn do_check_websites(
    mut websites: Vec<Website>,
    app_handle: AppHandle,
) -> Result<(Vec<Website>, String), String> {
    
    for website in websites.iter_mut() {
        let was_down = website.is_down;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();

        println!("Checking site: {}", website.url);
        
        match client.get(&website.url).send().await {
            Ok(response) => {
                website.is_down = !response.status().is_success();
                println!("Site is up: {}", website.url);
            }
            Err(_) => {
                website.is_down = true;
                println!("Site is down: {}", website.url);
            }
        }
        
        website.last_checked = chrono::Utc::now().to_rfc3339();
        
        // Send notification if website just went down and hasn't been cleared
        if website.is_down && !was_down && !website.notification_cleared {
            let _ = app_handle
                .notification()
                .builder()
                .title("Website Down!")
                .body(format!("{} is not responding", website.url))
                .show();
        }
        
        // Reset notification flag if website is back up
        if !website.is_down && was_down {
            website.notification_cleared = false;
        }
    }

    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let message = format!("Websites checked successfully at {}!", now);
    println!("Check complete: {}", message);

    let window = app_handle.get_webview_window("main").unwrap();
    window
        .emit("website_check_complete", message.clone())
        .unwrap();

   Ok((websites, message))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_keyring::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            // Create initial tray menu (assume window is visible, so "Hide")
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Hide", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            // Create tray icon
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if event.id == "quit" {
                        app.exit(0);
                    } else if event.id == "show" {
                        let state = app.state::<AppState>();
                        let window = app.get_webview_window("main").unwrap();
                        let is_visible = window.is_visible().unwrap_or(false);
                        if is_visible {
                            // Hide window and update menu to "Show"
                            window.hide().unwrap();
                            let quit_i =
                                MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
                            let show_i =
                                MenuItem::with_id(app, "show", "Show", true, None::<&str>).unwrap();
                            let menu = Menu::with_items(app, &[&show_i, &quit_i]).unwrap();
                            state.tray.set_menu(Some(menu)).unwrap();
                        } else {
                            // Show window and update menu to "Hide"
                            window.show().unwrap();
                            window.set_focus().unwrap();
                            let quit_i =
                                MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
                            let show_i =
                                MenuItem::with_id(app, "show", "Hide", true, None::<&str>).unwrap();
                            let menu = Menu::with_items(app, &[&show_i, &quit_i]).unwrap();
                            state.tray.set_menu(Some(menu)).unwrap();
                        }
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        let state = app.state::<AppState>();
                        let window = app.get_webview_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        // Update menu to "Hide" after showing
                        let quit_i =
                            MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
                        let show_i =
                            MenuItem::with_id(app, "show", "Hide", true, None::<&str>).unwrap();
                        let menu = Menu::with_items(app, &[&show_i, &quit_i]).unwrap();
                        state.tray.set_menu(Some(menu)).unwrap();
                    }
                })
                .build(app)?;

            let initial_websites = vec![
                Website {
                    url: "https://example.com".to_string(),
                    is_down: false,
                    last_checked: "2025-11-28".to_string(), // Use current date or a placeholder
                    notification_cleared: false,
                },
                // Add more initial websites as needed
            ];

            // Manage the tray in app state
            app.manage(AppState {
                websites: Mutex::new(initial_websites),
                tray,
                refresher_tx: Mutex::new(None),
            });

            // Spawn a background task to check websites every 60 seconds
            let app_handle = app.handle().clone();
            async_runtime::spawn(async move {
                let mut interval = interval(Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    let state = app_handle.state::<AppState>();
                    let websites_clone = state.websites.lock().unwrap().clone();
                    if let Ok((updated_websites, _)) = do_check_websites(websites_clone, app_handle.clone()).await {
                        *state.websites.lock().unwrap() = updated_websites;
                    }
                }
            });


            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
                // Update menu to "Show" after hiding
                let app = window.app_handle();
                let state = app.state::<AppState>();
                let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
                let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>).unwrap();
                let menu = Menu::with_items(app, &[&show_i, &quit_i]).unwrap();
                state.tray.set_menu(Some(menu)).unwrap();
            }
        })
        .invoke_handler(tauri::generate_handler![login, greet, check_websites, get_access_token, fetch_protected, logout, whoami, clear_last_user])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
