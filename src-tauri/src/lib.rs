// src-tauri/src/lib.rs
// Tauri アプリのコアライブラリ
// コマンド登録・プラグイン設定・アプリ初期化を担当

pub mod commands;
pub mod config;
pub mod lockscreen;
pub mod renderer;
pub mod scheduler;
pub mod sun;

use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .manage(commands::AppState {
            update_notify: Arc::new(tokio::sync::Notify::new()),
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            // アプリ起動時はウィンドウを非表示（トレイ常駐）
            // デバッグビルド時はすぐに表示する
            if let Some(window) = app.get_webview_window("settings") {
                #[cfg(debug_assertions)]
                window.show()?;
                #[cfg(not(debug_assertions))]
                window.hide()?;
            }

            // システムトレイメニューを構築
            let open_settings =
                MenuItem::with_id(app, "open_settings", "設定を開く", true, None::<&str>)?;
            let update_now =
                MenuItem::with_id(app, "update_now", "今すぐ更新", true, None::<&str>)?;
            let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&open_settings, &update_now, &separator, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open_settings" => {
                        if let Some(window) = app.get_webview_window("settings") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "update_now" => {
                        commands::trigger_update(app.state());
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("settings") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // スケジューラーをバックグラウンドで開始
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                scheduler::start(app_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::get_sun_info,
            commands::preview_image,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri アプリの起動に失敗しました");
}
