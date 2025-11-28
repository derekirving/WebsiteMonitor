// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{
    async_runtime,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tokio::time::{interval, Duration};
use tauri_plugin_notification::NotificationExt;

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
}

#[tauri::command]
fn greet(name: &str, app_handle: AppHandle) -> String {
    let _ = app_handle
        .notification()
        .builder()
        .title("Hello!")
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
        .setup(|app| {
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
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![greet, check_websites])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
