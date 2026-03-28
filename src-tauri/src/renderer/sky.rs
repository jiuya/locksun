// src-tauri/src/renderer/sky.rs
// 空のグラデーション背景を生成する
//
// render_sky: Preetham / Perez 大気散乱モデルで物理ベースの空を描画する
// その他レイヤー（地面, 太陽, 星, 雲）は従来通り

use super::palette::{Color, SkyColors};
use super::preetham::{PreethamSky, DEFAULT_TURBIDITY};
use crate::config::ImageConfig;
use crate::sun::SunPosition;
use image::{ImageBuffer, Rgb, RgbImage};
use std::f64::consts::FRAC_PI_2;

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

/// 画像上端 (y=0) を天頂、下端 (y=h) を地平線とみなしたときの天頂角 (rad) を返す
#[inline]
fn pixel_theta(y: u32, h: u32) -> f64 {
    // sky エリアは上 75% を使用（下 25% は render_ground が地面で上書き）
    // それ以上でも地平線色として扱う
    ((y as f64 / h as f64) * FRAC_PI_2).min(FRAC_PI_2 - 1e-6)
}

/// 画像 x 座標を方位角 (rad) に変換する (0 = 左端, 2π = 右端)
#[inline]
fn pixel_phi(x: u32, w: u32) -> f64 {
    (x as f64 / w as f64) * std::f64::consts::TAU
}

/// 空のグラデーション画像を Preetham / Perez 大気散乱モデルで生成する
///
/// 太陽が地平線以上の場合は物理ベースの計算、
/// 地平線以下（薄明・夜間）はパレット補間にフェードアウトする。
pub fn render_sky(pos: &SunPosition, cfg: &ImageConfig) -> RgbImage {
    let (w, h) = (cfg.width, cfg.height);
    let mut img: RgbImage = ImageBuffer::new(w, h);

    // ── Preetham モデルを構築 ─────────────────────────────────────────────
    let preetham = PreethamSky::new(pos.altitude, DEFAULT_TURBIDITY).with_azimuth(pos.azimuth);

    // ── パレット補間（薄明・夜間フォールバック + 地面色取得用）────────────
    let palette = SkyColors::from_altitude(pos.altitude);

    // 太陽高度角によるブレンド比率を決める
    //   altitude >= 2°  → Preetham 100%
    //   altitude <= -5° → Palette 100%
    //   その間        → 線形ブレンド
    let preetham_weight = ((pos.altitude - (-5.0)) / (2.0 - (-5.0))).clamp(0.0, 1.0);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let theta = pixel_theta(y, h);
        let phi = pixel_phi(x, w);

        // ── Preetham 空色 ──────────────────────────────────────────────
        let (pr, pg, pb) = preetham.sky_rgb(theta, phi);

        // ── パレット補間色（t_curved によるグラデーション）────────────
        let t = y as f64 / h as f64;
        let t_curved = t.powf(2.0);
        let pal = palette.zenith.lerp(palette.horizon, t_curved);

        // ── ブレンド ──────────────────────────────────────────────────
        // Preethamモデルの出力をさらに抑制して白飛びを防ぐ
        let preetham_factor = 0.75; // Preethamの強度を75%に抑制（さらに減少）
        let r = blend_f64(pal.0, (pr as f64 * preetham_factor) as u8, preetham_weight);
        let g = blend_f64(pal.1, (pg as f64 * preetham_factor) as u8, preetham_weight);
        let b = blend_f64(pal.2, (pb as f64 * preetham_factor) as u8, preetham_weight);

        *pixel = Rgb([r, g, b]);
    }

    img
}

/// palette の u8 値と Preetham の u8 値を weight でブレンドする
#[inline]
fn blend_f64(pal: u8, phys: u8, weight: f64) -> u8 {
    (pal as f64 * (1.0 - weight) + phys as f64 * weight)
        .clamp(0.0, 255.0)
        .round() as u8
}

/// 太陽が地面レイヤーに被らないよう地平線 Y 座標を返す（render_ground と一致させる）
#[inline]
fn ground_line_y(h: u32) -> i32 {
    (h as f64 * 0.75) as i32
}

/// 太陽方向の大気グローを描画する
///
/// 太陽ディスクは描画しない（人間は太陽を直視しないため）。
/// 代わりに、大きくぼんやりとした方向ヒントグローのみを重ねる。
/// このグローは後続の apply_blur でさらに拡散する。
/// 高度角が -5° 以下の場合は描画しない。
/// 地面レイヤー（下 25%）より下のピクセルには描画しない。
///
/// # 注意
/// 関数名を `render_sun` のままにして呼び出し側の変更を最小化している。
pub fn render_sun(pos: &SunPosition, cfg: &ImageConfig, base: &mut RgbImage) {
    if pos.altitude < -5.0 {
        return;
    }

    let colors = SkyColors::from_altitude(pos.altitude);
    let (w, h) = (cfg.width, cfg.height);

    let ground_y = ground_line_y(h);

    // 太陽方向の画面座標（地平線ライン = ground_y, 天頂 = 0）
    let altitude_frac = (pos.altitude / 90.0).clamp(0.0, 1.0);
    let sun_y = ground_y as f64 * (1.0 - altitude_frac);
    let sun_x = w as f64 * (pos.azimuth / 360.0);

    // グロー半径: 画面短辺の 45%（大きめにして後続ブラーで自然に溶け込む）
    let glow_radius: i32 = (w.min(h) as f64 * 0.45) as i32;
    let cx = sun_x as i32;
    let cy = sun_y as i32;

    for dy in -glow_radius..=glow_radius {
        for dx in -glow_radius..=glow_radius {
            let px = cx + dx;
            let py = cy + dy;
            if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 || py >= ground_y {
                continue;
            }
            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            if dist > glow_radius as f64 {
                continue;
            }
            // 中心ほど強く、外周に向けて 4 乗で急速に減衰
            // → 「ぼんやりした輝き」を表現。最大不透明度を0.20に抑制。
            let t = 1.0 - (dist / glow_radius as f64);
            let alpha = t.powi(4) * 0.20; // 0.30 → 0.20 に減少
            if alpha < 1e-4 {
                continue;
            }
            let base_pixel = base.get_pixel(px as u32, py as u32).0;
            let glow = colors.sun_halo;
            let blended = Rgb([
                blend(base_pixel[0], glow.0, alpha),
                blend(base_pixel[1], glow.1, alpha),
                blend(base_pixel[2], glow.2, alpha),
            ]);
            base.put_pixel(px as u32, py as u32, blended);
        }
    }
}

// ─── ガウシアンブラー ─────────────────────────────────────────────────────────

/// ボックスブラーの半径（ピクセル）
/// 3 パス繰り返すことで中央極限定理によりガウシアンに近似する。
/// 解像度に対して相対的に決める: 短辺の約 0.7%、最低 3px
fn blur_radius(w: u32, h: u32) -> u32 {
    ((w.min(h) as f64 * 0.007) as u32).max(3)
}

/// 水平方向のボックスブラー（1 パス）
fn box_blur_h(src: &RgbImage, radius: u32) -> RgbImage {
    let (w, h) = src.dimensions();
    let r = radius as i32;
    let div = (2 * r + 1) as f64;
    let mut dst = RgbImage::new(w, h);

    for y in 0..h {
        // 先頭ピクセルの初期ウィンドウを積算
        let mut sum = [0.0_f64; 3];
        for dx in -r..=r {
            let sx = dx.clamp(0, w as i32 - 1) as u32;
            let px = src.get_pixel(sx, y).0;
            sum[0] += px[0] as f64;
            sum[1] += px[1] as f64;
            sum[2] += px[2] as f64;
        }
        // スライディングウィンドウで右に走査
        for x in 0..w {
            dst.put_pixel(
                x,
                y,
                Rgb([
                    (sum[0] / div).round() as u8,
                    (sum[1] / div).round() as u8,
                    (sum[2] / div).round() as u8,
                ]),
            );
            // 次のピクセルに進むため左端を引き、右端を加える
            let leave = (x as i32 - r).clamp(0, w as i32 - 1) as u32;
            let enter = (x as i32 + r + 1).clamp(0, w as i32 - 1) as u32;
            let lp = src.get_pixel(leave, y).0;
            let ep = src.get_pixel(enter, y).0;
            sum[0] += ep[0] as f64 - lp[0] as f64;
            sum[1] += ep[1] as f64 - lp[1] as f64;
            sum[2] += ep[2] as f64 - lp[2] as f64;
        }
    }
    dst
}

/// 垂直方向のボックスブラー（1 パス）
fn box_blur_v(src: &RgbImage, radius: u32) -> RgbImage {
    let (w, h) = src.dimensions();
    let r = radius as i32;
    let div = (2 * r + 1) as f64;
    let mut dst = RgbImage::new(w, h);

    for x in 0..w {
        let mut sum = [0.0_f64; 3];
        for dy in -r..=r {
            let sy = dy.clamp(0, h as i32 - 1) as u32;
            let px = src.get_pixel(x, sy).0;
            sum[0] += px[0] as f64;
            sum[1] += px[1] as f64;
            sum[2] += px[2] as f64;
        }
        for y in 0..h {
            dst.put_pixel(
                x,
                y,
                Rgb([
                    (sum[0] / div).round() as u8,
                    (sum[1] / div).round() as u8,
                    (sum[2] / div).round() as u8,
                ]),
            );
            let leave = (y as i32 - r).clamp(0, h as i32 - 1) as u32;
            let enter = (y as i32 + r + 1).clamp(0, h as i32 - 1) as u32;
            let lp = src.get_pixel(x, leave).0;
            let ep = src.get_pixel(x, enter).0;
            sum[0] += ep[0] as f64 - lp[0] as f64;
            sum[1] += ep[1] as f64 - lp[1] as f64;
            sum[2] += ep[2] as f64 - lp[2] as f64;
        }
    }
    dst
}

/// ガウシアンブラーの近似（ボックスブラー 3 パス）を画像全体に適用する
///
/// ボックスブラーを 3 回繰り返すと中央極限定理によりガウシアンに近似される。
/// スライディングウィンドウにより O(W×H) で動作する。
pub fn apply_blur(img: RgbImage) -> RgbImage {
    let (w, h) = img.dimensions();
    let r = blur_radius(w, h);
    // 3 パスでガウシアン近似
    let pass1 = box_blur_v(&box_blur_h(&img, r), r);
    let pass2 = box_blur_v(&box_blur_h(&pass1, r), r);
    box_blur_v(&box_blur_h(&pass2, r), r)
}

/// 湖面エリア（下 25%）と地平線グローを描画する
/// 水面の反射効果、波紋、太陽の映り込みを含む
pub fn render_ground(pos: &SunPosition, cfg: &ImageConfig, base: &mut RgbImage) {
    let colors = SkyColors::from_altitude(pos.altitude);
    let (w, h) = (cfg.width, cfg.height);

    // 地平線のY座標: 画像の上から 75% の位置
    let ground_y = (h as f64 * 0.75) as u32;

    // 地平線グローのフェード幅 (px)
    let glow_width: u32 = (h as f64 * 0.04).max(8.0) as u32;

    // 太陽の方位角（ラジアン）: pos.azimuth は度なので変換する
    let sun_azimuth = pos.azimuth.to_radians();

    for y in 0..h {
        for x in 0..w {
            if y >= ground_y {
                // 湖面エリア: 空の反射 + 水面特性 + 太陽反射
                let base_water_color = colors.ground; // 基本的な水の色（深度による）

                // 1. 距離による深度効果（強化版）
                let depth_ratio = (y - ground_y) as f64 / (h - ground_y) as f64;
                let distance_factor = 1.0 - depth_ratio * 0.3;

                // 2. 水深による色調整（新機能！）
                // 0.0=浅い青緑, 1.0=深い青
                let water_depth = cfg.water_depth.clamp(0.0, 1.0);
                let shallow_color = Color(40, 80, 140); // 浅い水の青緑（暗め調整）
                let deep_color = Color(15, 40, 90); // 深い水の濃紺（暗め調整）
                let depth_adjusted_color = shallow_color.lerp(deep_color, water_depth);

                // 基本水色と溶け合わせ（白飛び防止のため水色優先に）
                let water_base = base_water_color.lerp(depth_adjusted_color, 0.8); // 0.6→0.8で水色を強化

                // 3. 空の色を水面に反映（重要な改良！）
                // 水面の各点で、対応する空の色を計算
                let sky_reflection_y = (ground_y as f64 * (1.0 - depth_ratio * 0.6)) as u32; // 遠くほど地平線近くを反映
                let sky_reflection_y = sky_reflection_y.min(ground_y.saturating_sub(1));

                let sky_pixel = base.get_pixel(x, sky_reflection_y).0;
                let reflected_sky = Color(sky_pixel[0], sky_pixel[1], sky_pixel[2]);

                // 4. 水の基本色と空の反射を混合
                // 水面反射率: 白飛び防止のため大幅に抑制
                let reflection_ratio = 0.15 + depth_ratio * 0.1; // 0.3→0.15にさらに削減
                let mixed_water_color = water_base.lerp(reflected_sky, reflection_ratio);

                // 5. 波紋効果（改良版）
                let ripple_x = x as f64 * 0.015; // 波長を少し長く
                let ripple_y = (y - ground_y) as f64 * 0.025;

                // 複数周波数の波を組み合わせてより自然な波紋を生成
                let wave1 = (ripple_x.sin() * ripple_y.cos()) * 0.4;
                let wave2 = ((ripple_x * 1.7).cos() * (ripple_y * 1.3).sin()) * 0.3;
                let wave3 = ((ripple_x * 0.6).sin() * (ripple_y * 2.1).cos()) * 0.2;

                let combined_wave = wave1 + wave2 + wave3;

                // 距離による波の減衰（遠くほど波が小さく見える）
                let wave_attenuation = 1.0 - depth_ratio * 0.4;
                let ripple_noise = combined_wave * wave_attenuation;
                let ripple_factor = 1.0 + ripple_noise * 0.06; // 波紋の影響を少し抑制

                // 6. 太陽の反射効果（既存のまま・抑制済み）
                let pixel_azimuth = (x as f64 / w as f64) * std::f64::consts::TAU;
                let sun_reflection_intensity = if pos.altitude > -6.0 {
                    // 太陽方向への反射強度を計算（円周上の最短角度距離）
                    let mut azimuth_diff = (pixel_azimuth - sun_azimuth).abs();
                    if azimuth_diff > std::f64::consts::PI {
                        azimuth_diff = std::f64::consts::TAU - azimuth_diff;
                    }

                    // 反射の幅を大幅に狭くして限定的な反射に
                    let reflection_width = 0.15 + depth_ratio * 1.5; // より狭い反射範囲（0.15→0.08）

                    let reflection_strength = if azimuth_diff < reflection_width {
                        let reflection_t = 1.0 - (azimuth_diff / reflection_width);

                        // 太陽高度による最大反射強度（さらに大幅に抑制）
                        let altitude_factor = pos.altitude;

                        // 距離による減衰を強化
                        let distance_attenuation = 1.0 - depth_ratio; // 強化

                        // 最終的な反射強度
                        reflection_t * altitude_factor * distance_attenuation
                    } else {
                        0.0
                    };
                    reflection_strength
                } else {
                    0.0
                };

                // 7. 最終的な水面色の合成（全体的に暗めに調整）
                let base_r = (mixed_water_color.0 as f64 * distance_factor * ripple_factor * 0.8) // 全体を20%暗く
                    .clamp(0.0, 255.0);
                let base_g = (mixed_water_color.1 as f64 * distance_factor * ripple_factor * 0.8)
                    .clamp(0.0, 255.0);
                let base_b = (mixed_water_color.2 as f64 * distance_factor * ripple_factor * 0.8)
                    .clamp(0.0, 255.0);

                // 太陽反射を極めて控えめに加算（白飛び防止のため大幅削減）

                let reflection_add = sun_reflection_intensity * 3.0; // さらに少なめに
                let final_r = (base_r + reflection_add).clamp(0.0, 255.0) as u8;
                let final_g = (base_g + reflection_add * 0.9).clamp(0.0, 255.0) as u8;
                let final_b = (base_b + reflection_add * 0.7).clamp(0.0, 255.0) as u8;

                base.put_pixel(x, y, Rgb([final_r, final_g, final_b]));
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
            let cloud_alpha = ((noise - CLOUD_NOISE_THRESHOLD) / CLOUD_NOISE_RANGE).clamp(0.0, 1.0)
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
    let v0 = (x * 0.007 + 1.3).sin() * (y * 0.009 + 0.7).sin(); // 重み 1.0
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
            water_depth: 0.7,
        }
    }

    fn night_pos() -> SunPosition {
        SunPosition {
            altitude: -20.0,
            azimuth: 0.0,
        }
    }

    fn day_pos() -> SunPosition {
        SunPosition {
            altitude: 30.0,
            azimuth: 180.0,
        }
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
        let changed = before.pixels().zip(base.pixels()).any(|(a, b)| a != b);
        assert!(changed, "深夜に星が描画されていない");
    }

    /// show_stars=false なら呼び出し元で抑制されるので、
    /// render_stars 単体を altitude >= -6.0 で呼ばないことを確認するため
    /// altitude = -5.0 のとき night_depth=0 なので何も変化しないことを確認
    #[test]
    fn test_stars_not_drawn_above_threshold() {
        let cfg = test_cfg();
        // altitude=-5.0 は -6.0 より大きいので night_depth=0 → alpha=0
        let pos = SunPosition {
            altitude: -5.0,
            azimuth: 0.0,
        };
        let mut base = render_sky(&pos, &cfg);
        let before = base.clone();
        render_stars(&pos, &cfg, &mut base);
        let changed = before.pixels().zip(base.pixels()).any(|(a, b)| a != b);
        assert!(!changed, "altitude=-5.0 で星が描画されてはいけない");
    }

    /// 夜が深いほど描画される星の数が多い（ピクセル変化数が増える）
    #[test]
    fn test_more_stars_at_deeper_night() {
        let cfg = test_cfg();
        let shallow_night = SunPosition {
            altitude: -8.0,
            azimuth: 0.0,
        };
        let deep_night = SunPosition {
            altitude: -20.0,
            azimuth: 0.0,
        };

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
        let changed = before.pixels().zip(base.pixels()).any(|(a, b)| a != b);
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
            water_depth: 0.7,
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
            water_depth: 0.7,
        };

        let pos_day = SunPosition {
            altitude: 45.0,
            azimuth: 180.0,
        };
        let pos_night = SunPosition {
            altitude: -20.0,
            azimuth: 0.0,
        };

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
        let pos = SunPosition {
            altitude: 45.0,
            azimuth: 180.0,
        };
        let cfg = ImageConfig {
            width: 64,
            height: 64,
            show_stars: false,
            show_clouds: true,
            water_depth: 0.7,
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
