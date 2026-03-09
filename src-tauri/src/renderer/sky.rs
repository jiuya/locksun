// src-tauri/src/renderer/sky.rs
// 空のグラデーション背景を生成する

use super::palette::SkyColors;
use crate::config::ImageConfig;
use crate::sun::SunPosition;
use image::{ImageBuffer, Rgb, RgbImage};

/// 星の配置を決める固定シード（同一シードで毎回同じ配置になる）
const STAR_PLACEMENT_SEED: u64 = 0xDEAD_BEEF_1234_5678;

/// 星の色範囲: R/G の最小値・変動幅（白〜淡青を実現）
const STAR_COLOR_MIN_RG: u8 = 200;
const STAR_COLOR_RANGE_RG: u32 = 56; // 200..=255

/// 星の色範囲: B の最小値・変動幅（B を R/G より高くして淡青に）
const STAR_COLOR_MIN_B: u8 = 220;
const STAR_COLOR_RANGE_B: u32 = 36; // 220..=255

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

/// 星を描画する
/// `altitude < -6.0`（市民薄明以前）のときに呼び出すことを想定しているが、
/// `altitude >= -6.0` で呼んだ場合は `night_depth` が 0 になり何も描画しない（安全）。
pub fn render_stars(pos: &SunPosition, cfg: &ImageConfig, base: &mut RgbImage) {
    let (w, h) = (cfg.width, cfg.height);

    // 夜の深さ: -6°=0.0, -18°以下=1.0
    let night_depth = ((-pos.altitude - 6.0) / 12.0).clamp(0.0, 1.0);

    // 描画数: 200〜800 点（高度に応じて変化）
    let star_count = (200.0 + night_depth * 600.0) as u32;

    // 固定シードの簡易 LCG（再現性のある配置）
    let mut seed: u64 = STAR_PLACEMENT_SEED;

    for _ in 0..star_count {
        let x = lcg_rand(&mut seed) % w;
        let y = lcg_rand(&mut seed) % h;

        // 地平線付近（y が大きい）ほど alpha を下げる（大気屈折の簡易模倣）
        let horizon_factor = 1.0 - (y as f64 / h as f64).powf(1.5);

        // 星の色（白〜淡青: R 200-255, G 200-255, B 220-255）
        let r = STAR_COLOR_MIN_RG + (lcg_rand(&mut seed) % STAR_COLOR_RANGE_RG) as u8;
        let g = STAR_COLOR_MIN_RG + (lcg_rand(&mut seed) % STAR_COLOR_RANGE_RG) as u8;
        let b = STAR_COLOR_MIN_B + (lcg_rand(&mut seed) % STAR_COLOR_RANGE_B) as u8;

        // 輝度ゆらぎ（0.6〜1.0）と夜の深さ・地平線減衰を合算
        let flicker = 0.6 + (lcg_rand(&mut seed) % 40) as f64 / 100.0;
        let alpha = (night_depth * horizon_factor * flicker).clamp(0.0, 1.0);

        let base_pixel = base.get_pixel(x, y).0;
        let blended = Rgb([
            blend(base_pixel[0], r, alpha),
            blend(base_pixel[1], g, alpha),
            blend(base_pixel[2], b, alpha),
        ]);
        base.put_pixel(x, y, blended);
    }
}

/// 64-bit LCG 乱数（Knuth の乗算定数 + 奇数加算定数）
fn lcg_rand(seed: &mut u64) -> u32 {
    *seed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    (*seed >> 32) as u32
}

/// alpha ブレンド (0.0=入力, 1.0=オーバーレイ)
fn blend(base: u8, overlay: u8, alpha: f64) -> u8 {
    (base as f64 * (1.0 - alpha) + overlay as f64 * alpha).clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ImageConfig;
    use crate::sun::SunPosition;

    fn test_cfg() -> ImageConfig {
        ImageConfig {
            width: 192,
            height: 108,
            show_stars: true,
            show_clouds: false,
        }
    }

    fn night_pos() -> SunPosition {
        SunPosition { altitude: -20.0, azimuth: 0.0 }
    }

    fn day_pos() -> SunPosition {
        SunPosition { altitude: 30.0, azimuth: 180.0 }
    }

    /// show_stars=true かつ深夜（altitude=-20°）で星が描画されること
    #[test]
    fn test_stars_drawn_at_night() {
        let cfg = test_cfg();
        let pos = night_pos();
        let mut base = render_sky(&pos, &cfg);
        let before = base.clone();
        render_stars(&pos, &cfg, &mut base);
        // 少なくとも 1 ピクセルが変化していること
        let changed = before
            .pixels()
            .zip(base.pixels())
            .any(|(a, b)| a != b);
        assert!(changed, "深夜に星が描画されていない");
    }

    /// show_stars=false なら呼び出し元で抑制されるので、
    /// render_stars 単体を altitude >= -6.0 で呼ばないことを確認するため
    /// altitude = -5.0 のとき night_depth=0 なので何も変化しないことを確認
    #[test]
    fn test_stars_not_drawn_above_threshold() {
        let cfg = test_cfg();
        // altitude=-5.0 は -6.0 より大きいので night_depth=0 → alpha=0
        let pos = SunPosition { altitude: -5.0, azimuth: 0.0 };
        let mut base = render_sky(&pos, &cfg);
        let before = base.clone();
        render_stars(&pos, &cfg, &mut base);
        let changed = before
            .pixels()
            .zip(base.pixels())
            .any(|(a, b)| a != b);
        assert!(!changed, "altitude=-5.0 で星が描画されてはいけない");
    }

    /// 夜が深いほど描画される星の数が多い（ピクセル変化数が増える）
    #[test]
    fn test_more_stars_at_deeper_night() {
        let cfg = test_cfg();
        let shallow_night = SunPosition { altitude: -8.0, azimuth: 0.0 };
        let deep_night = SunPosition { altitude: -20.0, azimuth: 0.0 };

        let mut base_shallow = render_sky(&shallow_night, &cfg);
        let before_shallow = base_shallow.clone();
        render_stars(&shallow_night, &cfg, &mut base_shallow);
        let changed_shallow = before_shallow
            .pixels()
            .zip(base_shallow.pixels())
            .filter(|(a, b)| a != b)
            .count();

        let mut base_deep = render_sky(&deep_night, &cfg);
        let before_deep = base_deep.clone();
        render_stars(&deep_night, &cfg, &mut base_deep);
        let changed_deep = before_deep
            .pixels()
            .zip(base_deep.pixels())
            .filter(|(a, b)| a != b)
            .count();

        assert!(
            changed_deep >= changed_shallow,
            "深夜の方が星の描画数が多いはず: shallow={changed_shallow}, deep={changed_deep}"
        );
    }

    /// 昼間は render_stars を呼んでも変化なし（day_pos で night_depth=0）
    #[test]
    fn test_stars_not_drawn_during_day() {
        let cfg = test_cfg();
        let pos = day_pos();
        let mut base = render_sky(&pos, &cfg);
        let before = base.clone();
        render_stars(&pos, &cfg, &mut base);
        let changed = before
            .pixels()
            .zip(base.pixels())
            .any(|(a, b)| a != b);
        assert!(!changed, "昼間に stars が描画されてはいけない");
    }
}
