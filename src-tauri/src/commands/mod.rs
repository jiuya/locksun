// src-tauri/src/commands/mod.rs
// Tauri コマンド定義
// フロントエンド (TypeScript) から invoke() で呼び出せる関数群

use crate::{
    config,
    sun::{SunCalculator, SunPosition, SunTimes},
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// アプリ共有状態
pub struct AppState {
    pub force_update: Mutex<bool>,
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
    use base64::{engine::general_purpose, Engine as _};
    let cfg = config::load().map_err(|e| e.to_string())?;
    let now = Local::now();
    let pos = SunCalculator::position(&now, cfg.location.latitude, cfg.location.longitude);

    let img = crate::renderer::composer::compose(&pos, &cfg.image).map_err(|e| e.to_string())?;

    // PNG として base64 エンコード
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let encoded = general_purpose::STANDARD.encode(buf.into_inner());
    Ok(format!("data:image/png;base64,{encoded}"))
}

/// 即時更新をトリガーする（トレイメニューから呼ばれる）
pub fn trigger_update(state: State<AppState>) {
    let mut flag = state.force_update.lock().unwrap();
    *flag = true;
}

/// フロントエンドに返すレスポンス型
#[derive(Debug, Serialize, Deserialize)]
pub struct SunInfoResponse {
    pub position: SunPosition,
    pub times: SunTimes,
    pub phase: String,
    pub location_name: String,
}
