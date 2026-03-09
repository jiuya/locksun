// src-tauri/src/renderer/sky.rs
// 空のグラデーション背景を生成する

use super::palette::{Color, SkyColors};
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

/// 夜間と判定する ambient 閾値
const AMBIENT_NIGHT_THRESHOLD: f64 = 0.05;
/// 夜間の雲の色（薄グレー）
const NIGHT_CLOUD_COLOR: Color = Color(50, 55, 70);
/// 昼間の雲の色（白）
const DAY_CLOUD_COLOR: Color = Color(245, 245, 250);
/// 雲を描画する上端からの割合（画面上部 65% のみ）
const CLOUD_COVERAGE_FRACTION: f64 = 0.65;
/// 雲ノイズの出現閾値（これ以上の値を雲とみなす）
const CLOUD_NOISE_THRESHOLD: f64 = 0.58;
/// 閾値からの最大変動幅（= 1.0 - CLOUD_NOISE_THRESHOLD）
const CLOUD_NOISE_RANGE: f64 = 1.0 - CLOUD_NOISE_THRESHOLD;
/// 雲の最大不透明度
const MAX_CLOUD_ALPHA: f64 = 0.75;

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

/// 地面エリア（下 25%）と地平線グローを描画する
pub fn render_ground(pos: &SunPosition, cfg: &ImageConfig, base: &mut RgbImage) {
    let colors = SkyColors::from_altitude(pos.altitude);
    let (w, h) = (cfg.width, cfg.height);

    // 地平線のY座標: 画像の上から 75% の位置
    let ground_y = (h as f64 * 0.75) as u32;

    // 地平線グローのフェード幅 (px)
    let glow_width: u32 = (h as f64 * 0.04).max(8.0) as u32;

    for y in 0..h {
        for x in 0..w {
            if y >= ground_y {
                // 地面エリア: 地面色で塗りつぶす
                base.put_pixel(x, y, colors.ground.to_rgb());
            } else if y + glow_width >= ground_y {
                // 地平線グロー帯: horizon 色を alpha ブレンドして境界を滑らかにする
                let dist = (ground_y - y) as f64; // ground_y までの距離 (px)
                let t = 1.0 - (dist / glow_width as f64).clamp(0.0, 1.0);
                // ガウス的減衰で自然なグロー
                let alpha = (t * t) * 0.75;
                let base_pixel = base.get_pixel(x, y).0;
                let h_color = colors.horizon;
                let blended = Rgb([
                    blend(base_pixel[0], h_color.0, alpha),
                    blend(base_pixel[1], h_color.1, alpha),
                    blend(base_pixel[2], h_color.2, alpha),
                ]);
                base.put_pixel(x, y, blended);
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

/// 雲エフェクトを描画する
/// 固定シードの sin 多重和ノイズで雲のシルエットを生成する
/// 雲の色は高度角に応じて変化: 昼は白、夕焼けはオレンジ〜ピンク、夜は薄グレー
///
/// # 引数
/// - `pos`: 太陽位置（高度角で雲の色を決定）
/// - `cfg`: 画像設定（幅・高さ）
/// - `base`: 描画先の画像（インプレースで上書きする）

pub fn render_clouds(pos: &SunPosition, cfg: &ImageConfig, base: &mut RgbImage) {
    let colors = SkyColors::from_altitude(pos.altitude);
    let (w, h) = (cfg.width, cfg.height);

    // 雲の色を決定: 夜は薄グレー、夕焼けはオレンジ〜ピンク、昼は白
    let cloud_color = if colors.ambient < AMBIENT_NIGHT_THRESHOLD {
        NIGHT_CLOUD_COLOR
    } else {
        colors
            .horizon
            .lerp(DAY_CLOUD_COLOR, colors.ambient.powf(0.5).clamp(0.0, 1.0))
    };

    // 雲が描画される最大高さ（画面上部 CLOUD_COVERAGE_FRACTION の範囲）
    let cloud_max_y = (h as f64 * CLOUD_COVERAGE_FRACTION) as u32;

    for y in 0..cloud_max_y {
        // 地平線に近いほど雲を薄くする
        let y_factor = 1.0 - (y as f64 / cloud_max_y as f64).powf(1.5);

        for x in 0..w {
            let noise = cloud_noise(x as f64, y as f64);

            if noise < CLOUD_NOISE_THRESHOLD {
                continue;
            }

            // 雲の濃さ（0.0 〜 MAX_CLOUD_ALPHA）
            let cloud_alpha =
                ((noise - CLOUD_NOISE_THRESHOLD) / CLOUD_NOISE_RANGE).clamp(0.0, 1.0)
                    * y_factor
                    * MAX_CLOUD_ALPHA;

            if cloud_alpha < 0.01 {
                continue;
            }

            let base_pixel = base.get_pixel(x, y).0;
            let blended = Rgb([
                blend(base_pixel[0], cloud_color.0, cloud_alpha),
                blend(base_pixel[1], cloud_color.1, cloud_alpha),
                blend(base_pixel[2], cloud_color.2, cloud_alpha),
            ]);
            base.put_pixel(x, y, blended);
        }
    }
}

/// 64-bit LCG 乱数（Knuth の乗算定数 + 奇数加算定数）
fn lcg_rand(seed: &mut u64) -> u32 {
    *seed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    (*seed >> 32) as u32
}

/// 固定シードの sin 多重和ノイズ (0.0 〜 1.0)
///
/// # 引数
/// - `x`, `y`: ピクセル座標（整数値をそのまま渡す）
///
/// 4 つの異なる周波数・位相・重みを持つ sin/cos 波の合成。
/// 重みの合計が 2.5 なので、オフセット 2.5 を加えて 5.0 で割ることで [0, 1] に正規化する。
fn cloud_noise(x: f64, y: f64) -> f64 {
    // 各オクターブ: (x 周波数, x 位相, y 周波数, y 位相, 重み)
    let v0 = (x * 0.007 + 1.3).sin() * (y * 0.009 + 0.7).sin();        // 重み 1.0
    let v1 = (x * 0.013 + 3.1).sin() * (y * 0.005 + 2.3).cos() * 0.7; // 重み 0.7
    let v2 = (x * 0.003 + 5.7).cos() * (y * 0.011 + 4.1).sin() * 0.5; // 重み 0.5
    let v3 = (x * 0.019 + 0.9).sin() * (y * 0.017 + 1.5).cos() * 0.3; // 重み 0.3
    // 理論最大絶対値: 1.0 + 0.7 + 0.5 + 0.3 = 2.5
    // → [−2.5, 2.5] を [0, 1] へ正規化
    const NOISE_OFFSET: f64 = 2.5;
    const NOISE_SCALE: f64 = 5.0; // = 2 * NOISE_OFFSET
    ((v0 + v1 + v2 + v3) + NOISE_OFFSET) / NOISE_SCALE
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

    #[test]
    fn test_clouds_modify_pixels() {
        // show_clouds=true で描画したとき、一部ピクセルが変化することを確認
        let pos_day = SunPosition {
            altitude: 45.0,
            azimuth: 180.0,
        };
        let cfg = ImageConfig {
            width: 64,
            height: 64,
            show_stars: false,
            show_clouds: true,
        };
        let mut base = render_sky(&pos_day, &cfg);
        let before = base.clone();
        render_clouds(&pos_day, &cfg, &mut base);
        // 少なくとも 1 ピクセル以上変化しているはず
        let changed = before
            .pixels()
            .zip(base.pixels())
            .filter(|(a, b)| a != b)
            .count();
        assert!(changed > 0, "雲描画後にピクセルが 1 つも変化していない");
    }

    #[test]
    fn test_cloud_noise_range() {
        // cloud_noise の出力が [0, 1] に収まることを確認
        for y in (0..1080u32).step_by(100) {
            for x in (0..1920u32).step_by(100) {
                let v = cloud_noise(x as f64, y as f64);
                assert!(
                    (0.0..=1.0).contains(&v),
                    "cloud_noise({x},{y}) = {v} が範囲外"
                );
            }
        }
    }

    #[test]
    fn test_cloud_color_changes_with_altitude() {
        // 昼と夜で雲の色が異なることを確認
        let cfg = ImageConfig {
            width: 64,
            height: 64,
            show_stars: false,
            show_clouds: true,
        };

        let pos_day = SunPosition { altitude: 45.0, azimuth: 180.0 };
        let pos_night = SunPosition { altitude: -20.0, azimuth: 0.0 };

        let mut day_img = render_sky(&pos_day, &cfg);
        let mut night_img = render_sky(&pos_night, &cfg);

        render_clouds(&pos_day, &cfg, &mut day_img);
        render_clouds(&pos_night, &cfg, &mut night_img);

        // 同じ (x, y) のピクセルが昼と夜で異なるはず
        let differ = day_img
            .pixels()
            .zip(night_img.pixels())
            .any(|(d, n)| d != n);
        assert!(differ, "昼と夜の雲描画結果が完全に一致している");
    }

    #[test]
    fn test_clouds_stay_in_upper_area() {
        // 画面下部（65% 以下）には雲ピクセルが存在しないことを確認
        let pos = SunPosition { altitude: 45.0, azimuth: 180.0 };
        let cfg = ImageConfig {
            width: 64,
            height: 64,
            show_stars: false,
            show_clouds: true,
        };
        let cloud_max_y = (cfg.height as f64 * 0.65) as u32;

        let mut base = render_sky(&pos, &cfg);
        let before_lower: Vec<_> = base
            .enumerate_pixels()
            .filter(|(_, y, _)| *y >= cloud_max_y)
            .map(|(x, y, p)| (x, y, *p))
            .collect();

        render_clouds(&pos, &cfg, &mut base);

        for (x, y, before_pixel) in &before_lower {
            let after_pixel = base.get_pixel(*x, *y);
            assert_eq!(
                before_pixel, after_pixel,
                "y={y} (cloud_max_y={cloud_max_y}) の下部に雲が描画された"
            );
        }
    }
}
