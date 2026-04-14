// src-tauri/src/scheduler/mod.rs
// 定期更新スケジューラー
// - 設定された interval_secs ごとに画像を再生成・適用する
// - アプリ設定の変更を検知して即時更新する

use crate::{commands::AppState, config, gemini, lockscreen, renderer, sun::SunCalculator};
use chrono::Local;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tokio::time::{sleep, Duration};

/// バックグラウンドで永続的にスケジューラーを実行する
pub async fn start(app: AppHandle) {
    log::info!("スケジューラー開始");

    let notify = app.state::<AppState>().update_notify.clone();

    loop {
        if let Err(e) = run_once(&app).await {
            log::error!("更新サイクルエラー: {e:#}");
        }

        let interval = config::load()
            .map(|c| c.update.interval_secs)
            .unwrap_or(300);
        let interval = interval.max(30);

        log::debug!("次回更新まで {interval} 秒待機");

        tokio::select! {
            _ = sleep(Duration::from_secs(interval)) => {},
            _ = notify.notified() => {
                log::info!("即時更新トリガー");
            },
        }
    }
}

/// 1回の更新サイクル: 太陽位置計算 → 画像生成 → (Gemini AI 強化) → ロックスクリーン適用
pub async fn run_once(app: &AppHandle) -> anyhow::Result<()> {
    let cfg = config::load()?;
    run_once_with_config(app, &cfg).await
}

/// 指定した設定を使って1回の更新サイクルを実行する（ファイル保存なし）
pub async fn run_once_with_config(app: &AppHandle, cfg: &config::AppConfig) -> anyhow::Result<()> {
    let now = Local::now();

    let pos = SunCalculator::position(&now, cfg.location.latitude, cfg.location.longitude);
    log::info!(
        "太陽位置 altitude={:.2}° azimuth={:.2}° @ {}",
        pos.altitude,
        pos.azimuth,
        now.format("%H:%M:%S")
    );

    let output_path = output_image_path(app);

    // ベース画像を PNG バイト列としてインメモリ生成（example と同じ方式）
    let base_png = renderer::render_to_bytes(&pos, &cfg.image)?;

    // Gemini AI 強化が有効な場合は画像を加工する（失敗時はベース画像にフォールバック）
    let final_bytes = if cfg.gemini.enabled && !cfg.gemini.api_key.is_empty() {
        match gemini::enhance_image(&cfg.gemini, &pos, &base_png).await {
            Ok(enhanced_bytes) => {
                log::info!("Gemini AI 強化済み画像を保存しました");
                enhanced_bytes
            }
            Err(e) => {
                // AI 強化に失敗しても通知してループは継続する
                log::error!("Gemini AI 強化に失敗しました（元の画像を使用）: {e:#}");
                if let Some(tray) = app.tray_by_id("main") {
                    let _ = tray.set_tooltip(Some("⚠️ Locksun: Gemini AI 強化に失敗しました"));
                }
                base_png
            }
        }
    } else {
        base_png
    };

    std::fs::write(&output_path, &final_bytes)?;
    lockscreen::set_lockscreen_image(&output_path)?;

    Ok(())
}

/// 画像出力先パスを返す
fn output_image_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_cache_dir()
        .unwrap_or_else(|_| {
            log::warn!("アプリキャッシュディレクトリの取得に失敗しました。カレントディレクトリを使用します");
            PathBuf::from(".")
        })
        .join("lockscreen.png")
}
