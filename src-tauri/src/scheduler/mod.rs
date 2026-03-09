// src-tauri/src/scheduler/mod.rs
// 定期更新スケジューラー
// - 設定された interval_secs ごとに画像を再生成・適用する
// - アプリ設定の変更を検知して即時更新する

use crate::{config, lockscreen, renderer, sun::SunCalculator};
use chrono::Local;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tokio::time::{sleep, Duration};

/// バックグラウンドで永続的にスケジューラーを実行する
pub async fn start(app: AppHandle) {
    log::info!("スケジューラー開始");

    loop {
        if let Err(e) = run_once(&app) {
            log::error!("更新サイクルエラー: {e:#}");
        }

        let interval = config::load()
            .map(|c| c.update.interval_secs)
            .unwrap_or(300);

        log::debug!("次回更新まで {interval} 秒待機");
        sleep(Duration::from_secs(interval)).await;
    }
}

/// 1回の更新サイクル: 太陽位置計算 → 画像生成 → ロックスクリーン適用
pub fn run_once(app: &AppHandle) -> anyhow::Result<()> {
    let cfg = config::load()?;
    let now = Local::now();

    let pos = SunCalculator::position(&now, cfg.location.latitude, cfg.location.longitude);
    log::info!(
        "太陽位置 altitude={:.2}° azimuth={:.2}° @ {}",
        pos.altitude,
        pos.azimuth,
        now.format("%H:%M:%S")
    );

    let output_path = output_image_path(app);
    renderer::render_and_save(&pos, &cfg.image, &output_path)?;

    lockscreen::set_lockscreen_image(&output_path)?;

    Ok(())
}

/// 画像出力先パスを返す
fn output_image_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_cache_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("lockscreen.png")
}
