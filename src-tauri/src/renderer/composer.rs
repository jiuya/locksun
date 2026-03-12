// src-tauri/src/renderer/composer.rs
// 各レイヤーを合成して最終画像を生成する

use super::sky::{apply_blur, render_clouds, render_ground, render_sky, render_stars, render_sun};
use crate::config::ImageConfig;
use crate::sun::SunPosition;
use anyhow::Result;
use image::RgbImage;

/// 全レイヤーを合成した最終画像を返す
pub fn compose(pos: &SunPosition, cfg: &ImageConfig) -> Result<RgbImage> {
    // Layer 1: 空グラデーション（Preetham 大気散乱）
    let mut img = render_sky(pos, cfg);

    // Layer 2: 地面エリア + 地平線グロー
    render_ground(pos, cfg, &mut img);

    // Layer 3: 太陽方向の大気グロー（ディスクなし・ぼんやりした方向ヒントのみ）
    render_sun(pos, cfg, &mut img);

    // Layer 4: 星（夜間, cfg.show_stars が true の場合）
    if cfg.show_stars && pos.altitude < -6.0 {
        render_stars(pos, cfg, &mut img);
    }

    // Layer 5: 雲エフェクト（cfg.show_clouds が true の場合）
    if cfg.show_clouds {
        render_clouds(pos, cfg, &mut img);
    }

    // Layer 6: ガウシアンブラー（全体をソフトに仕上げる）
    // ボックスブラー 3 パスによるガウシアン近似。O(W×H) で動作。
    let img = apply_blur(img);

    Ok(img)
}
