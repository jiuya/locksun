// src-tauri/src/renderer/sky.rs
// 空のグラデーション背景を生成する

use super::palette::SkyColors;
use crate::config::ImageConfig;
use crate::sun::SunPosition;
use image::{ImageBuffer, Rgb, RgbImage};

/// 空のグラデーション画像を生成する
pub fn render_sky(pos: &SunPosition, cfg: &ImageConfig) -> RgbImage {
    let colors = SkyColors::from_altitude(pos.altitude);
    let (w, h) = (cfg.width, cfg.height);
    let mut img: RgbImage = ImageBuffer::new(w, h);

    for (_, y, pixel) in img.enumerate_pixels_mut() {
        // y=0 が天頂, y=h が地平線
        let t = y as f64 / h as f64;
        // べき乗で地平線付近に色変化を集中させる（指数 > 1.0 → 天頂は天頂色、下端で急変）
        let t_curved = t.powf(2.0);
        *pixel = colors.zenith.lerp(colors.horizon, t_curved).to_rgb();
    }

    img
}

/// 太陽ディスクとハローを描画する
/// 高度角が -5° 以下の場合は描画しない
pub fn render_sun(pos: &SunPosition, cfg: &ImageConfig, base: &mut RgbImage) {
    if pos.altitude < -5.0 {
        return;
    }

    let colors = SkyColors::from_altitude(pos.altitude);
    let (w, h) = (cfg.width, cfg.height);

    // 画面上の太陽位置を算出
    // 高度角を垂直位置に変換（0°=地平線=h px, 90°≈天頂=0 px）
    let sun_y = h as f64 * (1.0 - (pos.altitude / 95.0).clamp(0.0, 1.0));
    // 方位角を水平位置に変換
    let sun_x = w as f64 * (pos.azimuth / 360.0);

    let halo_radius: i32 = (w.min(h) as f64 * 0.25) as i32;
    let disk_radius: i32 = (w.min(h) as f64 * 0.025).max(8.0) as i32;

    let cx = sun_x as i32;
    let cy = sun_y as i32;

    // ハロー（ソフトグロー）を描画
    for dy in -halo_radius..=halo_radius {
        for dx in -halo_radius..=halo_radius {
            let px = cx + dx;
            let py = cy + dy;
            if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                continue;
            }
            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            if dist > halo_radius as f64 {
                continue;
            }
            let t = 1.0 - (dist / halo_radius as f64);
            // ガウス的減衰
            let alpha = (t * t * t).min(1.0) * 0.6;
            let base_pixel = base.get_pixel(px as u32, py as u32).0;
            let halo = colors.sun_halo;
            let blended = Rgb([
                blend(base_pixel[0], halo.0, alpha),
                blend(base_pixel[1], halo.1, alpha),
                blend(base_pixel[2], halo.2, alpha),
            ]);
            base.put_pixel(px as u32, py as u32, blended);
        }
    }

    // 太陽ディスクを描画
    for dy in -disk_radius..=disk_radius {
        for dx in -disk_radius..=disk_radius {
            let px = cx + dx;
            let py = cy + dy;
            if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                continue;
            }
            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            if dist <= disk_radius as f64 {
                let disk = colors.sun_disk;
                base.put_pixel(px as u32, py as u32, disk.to_rgb());
            }
        }
    }
}

/// alpha ブレンド (0.0=入力, 1.0=オーバーレイ)
fn blend(base: u8, overlay: u8, alpha: f64) -> u8 {
    (base as f64 * (1.0 - alpha) + overlay as f64 * alpha).clamp(0.0, 255.0) as u8
}
