// src-tauri/src/renderer/composer.rs
// 各レイヤーを合成して最終画像を生成する

use super::sky::{render_clouds, render_sky, render_stars, render_sun};
use crate::config::ImageConfig;
use crate::sun::SunPosition;
use anyhow::Result;
use image::RgbImage;

/// 全レイヤーを合成した最終画像を返す
pub fn compose(pos: &SunPosition, cfg: &ImageConfig) -> Result<RgbImage> {
    // Layer 1: 空グラデーション
    let mut img = render_sky(pos, cfg);

    // Layer 2: 太陽ディスク + ハロー
    render_sun(pos, cfg, &mut img);

    // Layer 3: 星（夜間, cfg.show_stars が true の場合）
    if cfg.show_stars && pos.altitude < -6.0 {
        render_stars(pos, cfg, &mut img);
    }

    // Layer 4: 雲エフェクト（cfg.show_clouds が true の場合）
    if cfg.show_clouds {
        render_clouds(pos, cfg, &mut img);
    }

    Ok(img)
}
