// src-tauri/src/renderer/mod.rs
// 画像生成モジュール
// SunPosition を受け取り、ロックスクリーン用PNG画像を生成する

pub mod composer;
pub mod palette;
pub mod preetham;
pub mod sky;

use crate::config::ImageConfig;
use crate::sun::SunPosition;
use anyhow::Result;
use image::ImageFormat;
use std::io::Cursor;
use std::path::Path;

/// メインエントリー: 太陽位置から画像を生成してパスに保存
pub fn render_and_save(pos: &SunPosition, cfg: &ImageConfig, output: &Path) -> Result<()> {
    let img = composer::compose(pos, cfg)?;
    img.save(output)?;
    Ok(())
}

/// 太陽位置から画像を生成して PNG バイト列として返す
pub fn render_to_bytes(pos: &SunPosition, cfg: &ImageConfig) -> Result<Vec<u8>> {
    let img = composer::compose(pos, cfg)?;
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Png)?;
    Ok(buf.into_inner())
}
