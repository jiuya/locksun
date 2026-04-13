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
            // Windows のみ: 権限エラーを検知してトレイ通知する（1回のみ）
            #[cfg(target_os = "windows")]
            {
                if !lockscreen::check_permission() {
                    log::warn!("権限エラーを検出: {e:#}");
                    let state = app.state::<AppState>();
                    let mut notified = state.permission_notified.lock().unwrap();
                    if !*notified {
                        *notified = true;
                        if let Some(tray) = app.tray_by_id("main") {
                            let _ = tray.set_tooltip(Some("⚠️ Locksun: 管理者権限が必要です"));
                        }
                        log::warn!("権限エラー: トレイ通知を更新しました");
                    }
                } else {
                    log::error!("更新サイクルエラー: {e:#}");
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                log::error!("更新サイクルエラー: {e:#}");
            }
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

    // Gemini AI 強化が有効な場合は画像を加工する
    if cfg.gemini.enabled && !cfg.gemini.api_key.is_empty() {
        let png_bytes = std::fs::read(&output_path)?;
        match gemini::enhance_image(&cfg.gemini, &pos, png_bytes).await {
            Ok(enhanced_bytes) => {
                std::fs::write(&output_path, &enhanced_bytes)?;
                log::info!("Gemini AI 強化済み画像を保存しました");
            }
            Err(e) => {
                // AI 強化に失敗しても通知してループは継続する
                log::error!("Gemini AI 強化に失敗しました（元の画像を使用）: {e:#}");
                if let Some(tray) = app.tray_by_id("main") {
                    let _ = tray.set_tooltip(Some("⚠️ Locksun: Gemini AI 強化に失敗しました"));
                }
            }
        }
    }

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
