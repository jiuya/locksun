// src-tauri/src/commands/mod.rs
// Tauri コマンド定義
// フロントエンド (TypeScript) から invoke() で呼び出せる関数群

use crate::{
    config,
    sun::{SunCalculator, SunPosition, SunTimes},
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;
use tokio::sync::Notify;

/// アプリ共有状態
pub struct AppState {
    pub update_notify: Arc<Notify>,
    /// 権限エラー通知済みフラグ（重複通知を防ぐ）
    pub permission_notified: Mutex<bool>,
}

/// 設定を取得する
#[tauri::command]
pub fn get_config() -> Result<config::AppConfig, String> {
    config::load().map_err(|e| e.to_string())
}

/// 設定を保存する
#[tauri::command]
pub fn save_config(cfg: config::AppConfig) -> Result<(), String> {
    config::save(&cfg).map_err(|e| e.to_string())
}

/// 現在の太陽位置・時刻情報を返す（プレビュー用）
#[tauri::command]
pub fn get_sun_info() -> Result<SunInfoResponse, String> {
    let cfg = config::load().map_err(|e| e.to_string())?;
    let now = Local::now();
    let pos = SunCalculator::position(&now, cfg.location.latitude, cfg.location.longitude);
    let times = SunCalculator::times(&now, cfg.location.latitude, cfg.location.longitude);
    let phase = SunCalculator::phase(&now, cfg.location.latitude, cfg.location.longitude);
    Ok(SunInfoResponse {
        position: pos,
        times,
        phase: format!("{phase:?}"),
        location_name: cfg.location.name,
    })
}

/// プレビュー用: 現在の設定で画像を生成してbase64を返す
#[tauri::command]
pub fn preview_image() -> Result<String, String> {
    let cfg = config::load().map_err(|e| e.to_string())?;
    let now = Local::now();
    let pos = SunCalculator::position(&now, cfg.location.latitude, cfg.location.longitude);
    let img = crate::renderer::composer::compose(&pos, &cfg.image).map_err(|e| e.to_string())?;
    encode_to_png_base64(img)
}

/// プレビュー用: 指定した設定で画像を生成してbase64を返す（保存は行わない）
#[tauri::command]
pub fn preview_image_with_config(cfg: config::AppConfig) -> Result<String, String> {
    let now = Local::now();
    let pos = SunCalculator::position(&now, cfg.location.latitude, cfg.location.longitude);
    let img = crate::renderer::composer::compose(&pos, &cfg.image).map_err(|e| e.to_string())?;
    encode_to_png_base64(img)
}

/// PNG 画像を base64 DataURL に変換する
fn encode_to_png_base64(img: image::RgbImage) -> Result<String, String> {
    use base64::{engine::general_purpose, Engine as _};
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let encoded = general_purpose::STANDARD.encode(buf.into_inner());
    Ok(format!("data:image/png;base64,{encoded}"))
}

/// プレビュー用: AI 強化済み画像を base64 で返す
#[tauri::command]
pub async fn preview_image_enhanced() -> Result<String, String> {
    let cfg = config::load().map_err(|e| e.to_string())?;
    let now = Local::now();
    let pos = SunCalculator::position(&now, cfg.location.latitude, cfg.location.longitude);
    let img = crate::renderer::composer::compose(&pos, &cfg.image).map_err(|e| e.to_string())?;

    // PNG バイト列に変換
    let png_bytes = {
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        buf.into_inner()
    };

    // Gemini AI 強化
    let enhanced = crate::gemini::enhance_image(&cfg.gemini, &pos, &png_bytes)
        .await
        .map_err(|e| e.to_string())?;

    // base64 DataURL に変換
    use base64::{engine::general_purpose, Engine as _};
    let encoded = general_purpose::STANDARD.encode(&enhanced);
    Ok(format!("data:image/png;base64,{encoded}"))
}

/// 現在の設定で画像を生成してロックスクリーンに即座に適用する
#[tauri::command]
pub async fn apply_to_lockscreen(app: tauri::AppHandle) -> Result<(), String> {
    crate::scheduler::run_once(&app)
        .await
        .map_err(|e| e.to_string())
}

/// 指定した設定でロックスクリーンに適用する（ファイル保存なし・プレビューと一致した画像を適用）
#[tauri::command]
pub async fn apply_to_lockscreen_with_config(
    cfg: config::AppConfig,
    app: tauri::AppHandle,
) -> Result<(), String> {
    crate::scheduler::run_once_with_config(&app, &cfg)
        .await
        .map_err(|e| e.to_string())
}

/// 指定した設定で太陽情報を返す（保存済み設定ではなくフォームの値を使う）
#[tauri::command]
pub fn get_sun_info_for_config(cfg: config::AppConfig) -> Result<SunInfoResponse, String> {
    let now = Local::now();
    let pos = SunCalculator::position(&now, cfg.location.latitude, cfg.location.longitude);
    let times = SunCalculator::times(&now, cfg.location.latitude, cfg.location.longitude);
    let phase = SunCalculator::phase(&now, cfg.location.latitude, cfg.location.longitude);
    Ok(SunInfoResponse {
        position: pos,
        times,
        phase: format!("{phase:?}"),
        location_name: cfg.location.name,
    })
}

/// プレビュー用: 指定した設定で AI 強化済み画像を base64 で返す
#[tauri::command]
pub async fn preview_image_enhanced_with_config(cfg: config::AppConfig) -> Result<String, String> {
    let now = Local::now();
    let pos = SunCalculator::position(&now, cfg.location.latitude, cfg.location.longitude);
    let img = crate::renderer::composer::compose(&pos, &cfg.image).map_err(|e| e.to_string())?;

    let png_bytes = {
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        buf.into_inner()
    };

    let enhanced = crate::gemini::enhance_image(&cfg.gemini, &pos, &png_bytes)
        .await
        .map_err(|e| e.to_string())?;

    use base64::{engine::general_purpose, Engine as _};
    let encoded = general_purpose::STANDARD.encode(&enhanced);
    Ok(format!("data:image/png;base64,{encoded}"))
}

/// 即時更新をトリガーする（トレイメニューから呼ばれる）
pub fn trigger_update(state: State<AppState>) {
    state.update_notify.notify_one();
}

/// フロントエンドに返すレスポンス型
#[derive(Debug, Serialize, Deserialize)]
pub struct SunInfoResponse {
    pub position: SunPosition,
    pub times: SunTimes,
    pub phase: String,
    pub location_name: String,
}
