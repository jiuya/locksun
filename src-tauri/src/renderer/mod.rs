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
use std::path::Path;

/// メインエントリー: 太陽位置から画像を生成してパスに保存
pub fn render_and_save(pos: &SunPosition, cfg: &ImageConfig, output: &Path) -> Result<()> {
    let img = composer::compose(pos, cfg)?;
    img.save(output)?;
    Ok(())
}

/// 太陽位置から画像を生成し、PNG バイト列をインメモリで返す（ディスク I/O なし）
///
/// example/generate_gemini_test.rs と同じ PNG エンコード方式（`PngEncoder::write_image`）を使用する。
pub fn render_to_bytes(pos: &SunPosition, cfg: &ImageConfig) -> Result<Vec<u8>> {
    use image::ImageEncoder;
    let img = composer::compose(pos, cfg)?;
    let mut png_bytes: Vec<u8> = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png_bytes).write_image(
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(png_bytes)
}
