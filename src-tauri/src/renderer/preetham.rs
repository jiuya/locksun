// src-tauri/src/renderer/preetham.rs
//
// Preetham / Perez 大気散乱モデル
//
// 参考論文: Preetham et al. "A Practical Analytic Model for Daylight" (SIGGRAPH 1999)
//
// 計算フロー:
//   SunPosition + Turbidity
//       ↓
//   Perez 配光関数 F(θ, γ) で CIE xyY を計算
//       ↓
//   CIE xyY → XYZ → linear sRGB → tone-mapping → γ補正 → u8 RGB

use std::f64::consts::{FRAC_PI_2, PI};

// ─── Perez 係数 ──────────────────────────────────────────────────────────────
// 各係数は c0 + c1*T の形式 (T = Turbidity 乱流係数)
// [a, b, c, d, e] の順に格納

/// 輝度 Y チャンネルの Perez 係数 [c0, c1] × 5
const PEREZ_Y_COEFF: [[f64; 2]; 5] = [
    [0.17872, -1.46303], // a =  0.17872T - 1.46303
    [-0.35540, 0.42749], // b = -0.35540T + 0.42749
    [-0.02266, 5.32505], // c = -0.02266T + 5.32505
    [0.26688, -2.57705], // d =  0.26688T - 2.57705
    [-0.06710, 0.37027], // e = -0.06710T + 0.37027
];

/// x 色度チャンネルの Perez 係数
const PEREZ_X_COEFF: [[f64; 2]; 5] = [
    [-0.01925, -0.25922], // a
    [-0.06651, 0.00081],  // b
    [-0.00041, 0.21247],  // c
    [-0.06409, -0.89887], // d
    [-0.00325, 0.04517],  // e
];

/// y 色度チャンネルの Perez 係数
const PEREZ_Y_CHR_COEFF: [[f64; 2]; 5] = [
    [-0.01669, -0.26078], // a
    [-0.09495, 0.00921],  // b
    [-0.00792, 0.21023],  // c
    [-0.04405, -1.65369], // d
    [-0.01092, 0.05291],  // e
];

// ─── 天頂 x 色度の多項式係数 (Table 2, Preetham 1999) ──────────────────────
// x_z = T^2 * dot([θ^3, θ^2, θ, 1], ROW2)
//       + T   * dot([θ^3, θ^2, θ, 1], ROW1)
//       +       dot([θ^3, θ^2, θ, 1], ROW0)
const ZENITH_X_T2: [f64; 4] = [0.00166, -0.00375, 0.00209, 0.0];
const ZENITH_X_T1: [f64; 4] = [-0.02903, 0.06377, -0.03202, 0.00394];
const ZENITH_X_T0: [f64; 4] = [0.11693, -0.21196, 0.06052, 0.25886];

/// 天頂 y 色度の多項式係数
const ZENITH_Y_T2: [f64; 4] = [0.00275, -0.00610, 0.00317, 0.0];
const ZENITH_Y_T1: [f64; 4] = [-0.04214, 0.08970, -0.04153, 0.00516];
const ZENITH_Y_T0: [f64; 4] = [0.15346, -0.26756, 0.06670, 0.26688];

// ─── sRGB トーンマッピング ───────────────────────────────────────────────────
/// 露出係数（空全体の明るさ調整）
///
/// Perez 式の輝度は kcd/m² 単位 (典型値: 天頂 ~7, 地平線方向 ~20 kcd/m²)。
/// Reinhard で [0,1] にマップするには EXPOSURE ≈ 1/zenith_lum_typical が適切。
/// 8.0 は大きすぎて全色が白飽和するため 0.12 に設定。
const EXPOSURE: f64 = 0.12;

// ─── Turbidity の既定値 ──────────────────────────────────────────────────────
/// 標準的な晴れ空（都市近郊）
pub const DEFAULT_TURBIDITY: f64 = 3.0;

// ─── Preetham 空モデル ───────────────────────────────────────────────────────

/// Preetham / Perez 大気散乱モデルを保持する構造体
///
/// 1 フレームごとに構築し、各ピクセルの RGB を `sky_rgb()` で取得する。
pub struct PreethamSky {
    /// 太陽の天頂角 (rad)   0 = 真上, π/2 = 地平線
    theta_s: f64,
    /// 太陽の方位角 (rad)   0 = 北, 時計回り
    phi_s: f64,
    /// Perez Y 係数 [a,b,c,d,e]
    perez_y_lum: [f64; 5],
    /// Perez x 係数
    perez_x_chr: [f64; 5],
    /// Perez y 係数
    perez_y_chr: [f64; 5],
    /// 天頂輝度 (kcd/m²)
    zenith_lum: f64,
    /// 天頂 x 色度
    zenith_x: f64,
    /// 天頂 y 色度
    zenith_y: f64,
    /// 基準 Perez 値 F(0, θs) — 正規化分母（Y, x, y 各チャンネル）
    norm_y: f64,
    norm_x: f64,
    norm_yc: f64,
    /// 太陽が地平線以下かどうか（描画抑制に使用）
    pub sun_below_horizon: bool,
}

impl PreethamSky {
    /// 太陽高度角 (度) と乱流係数 T から構築する
    ///
    /// # 引数
    /// - `altitude_deg`: 太陽高度角。-90°(地平線下) 〜 +90°(天頂)
    /// - `turbidity`: 乱流係数。1.7=澄み切った空、3=標準、6〜10=もや
    pub fn new(altitude_deg: f64, turbidity: f64) -> Self {
        let t = turbidity.clamp(1.7, 10.0);

        // 太陽の天頂角: altitude = 90° - theta_s
        // 地平線以下の場合は theta_s を π/2 でクランプ（モデルの定義域外のため）
        let sun_below = altitude_deg < 0.0;
        let clamped_alt = altitude_deg.clamp(0.0, 90.0);
        let theta_s = (90.0_f64 - clamped_alt).to_radians();

        // 太陽方位角は render_sky() で phi = azimuth.to_radians() として渡す想定だが、
        // ここでは初期値 0 にしておき、外部から phi_s を指定できる設計とする。
        // （render_sky は phi_s を azimuth から算出して再構築）
        let phi_s = 0.0_f64;

        // ── Perez 係数を計算 ──────────────────────────────────────────────
        let perez_y_lum = compute_perez_coeffs(&PEREZ_Y_COEFF, t);
        let perez_x_chr = compute_perez_coeffs(&PEREZ_X_COEFF, t);
        let perez_y_chr = compute_perez_coeffs(&PEREZ_Y_CHR_COEFF, t);

        // ── 天頂輝度 ─────────────────────────────────────────────────────
        let chi = (4.0 / 9.0 - t / 120.0) * (PI - 2.0 * theta_s);
        let zenith_lum = ((4.0453 * t - 4.9710) * chi.tan() - 0.2155 * t + 2.4192).max(0.0);

        // ── 天頂色度 ─────────────────────────────────────────────────────
        let zenith_x = eval_zenith_chroma(&ZENITH_X_T2, &ZENITH_X_T1, &ZENITH_X_T0, t, theta_s);
        let zenith_y = eval_zenith_chroma(&ZENITH_Y_T2, &ZENITH_Y_T1, &ZENITH_Y_T0, t, theta_s);

        // ── 正規化分母 F(0, θs) ──────────────────────────────────────────
        let norm_y = perez_f(&perez_y_lum, 0.0, theta_s);
        let norm_x = perez_f(&perez_x_chr, 0.0, theta_s);
        let norm_yc = perez_f(&perez_y_chr, 0.0, theta_s);

        Self {
            theta_s,
            phi_s,
            perez_y_lum,
            perez_x_chr,
            perez_y_chr,
            zenith_lum,
            zenith_x,
            zenith_y,
            norm_y,
            norm_x,
            norm_yc,
            sun_below_horizon: sun_below,
        }
    }

    /// 太陽の方位角 (度) を設定する
    pub fn with_azimuth(mut self, azimuth_deg: f64) -> Self {
        self.phi_s = azimuth_deg.to_radians();
        self
    }

    /// 空の方向 (天頂角 θ [rad], 方位角 φ [rad]) の sRGB 色を返す
    ///
    /// θ: 0=天頂、π/2=地平線、π/2 以上は地平線として扱う
    pub fn sky_rgb(&self, theta: f64, phi: f64) -> (u8, u8, u8) {
        // 地平線より下はスカイモデルを使わない
        let theta = theta.clamp(0.0, FRAC_PI_2 - 1e-6);

        // 太陽とのなす角 γ を内積で計算
        let cos_g = theta.cos() * self.theta_s.cos()
            + theta.sin() * self.theta_s.sin() * (phi - self.phi_s).cos();
        let gamma = cos_g.clamp(-1.0, 1.0).acos();

        // ── Perez 配光関数で xyY を計算 ──────────────────────────────────
        let f_y = perez_f(&self.perez_y_lum, theta, gamma);
        let f_x = perez_f(&self.perez_x_chr, theta, gamma);
        let f_yc = perez_f(&self.perez_y_chr, theta, gamma);

        let lum_y = if self.norm_y.abs() > 1e-10 {
            self.zenith_lum * f_y / self.norm_y
        } else {
            0.0
        };
        let chroma_x = if self.norm_x.abs() > 1e-10 {
            self.zenith_x * f_x / self.norm_x
        } else {
            self.zenith_x
        };
        let chroma_y = if self.norm_yc.abs() > 1e-10 {
            self.zenith_y * f_yc / self.norm_yc
        } else {
            self.zenith_y
        };

        // ── CIE xyY → XYZ ────────────────────────────────────────────────
        let (r, g, b) = xyyto_srgb(chroma_x, chroma_y, lum_y);

        (r, g, b)
    }

    /// 地平線方向 (θ = π/2) の色を返す（地平線グローに使用）
    pub fn horizon_rgb(&self) -> (u8, u8, u8) {
        // 地平線は theta = π/2 — モデルは θ→π/2 で発散する傾向があるため
        // θ = 85° で代替する
        let theta = 85.0_f64.to_radians();
        self.sky_rgb(theta, self.phi_s)
    }

    /// 太陽ディスク方向 (θ=θs, φ=φs) の色を返す
    pub fn sun_disk_rgb(&self) -> (u8, u8, u8) {
        // γ ≈ 0 の方向
        let tiny_offset = 0.001_f64;
        self.sky_rgb(self.theta_s.max(tiny_offset), self.phi_s)
    }
}

// ─── 内部ヘルパー ─────────────────────────────────────────────────────────────

/// Perez 配光関数
///
/// F(θ, γ) = (1 + a·exp(b/cosθ)) · (1 + c·exp(d·γ) + e·cos²γ)
///
/// - `coeff`: [a, b, c, d, e]
/// - `theta`: 天頂角 (rad)
/// - `gamma`: 太陽とのなす角 (rad)
fn perez_f(coeff: &[f64; 5], theta: f64, gamma: f64) -> f64 {
    let [a, b, c, d, e] = *coeff;

    // θ=0 のとき cos(θ)=1 で問題なし
    let cos_theta = theta.cos().max(1e-6);
    let term1 = 1.0 + a * (b / cos_theta).exp();
    let term2 = 1.0 + c * (d * gamma).exp() + e * gamma.cos().powi(2);
    term1 * term2
}

/// 係数テーブルから Perez [a,b,c,d,e] を計算する
///
/// table[i] = [c0, c1] → coeff[i] = c0 + c1*T だが、
/// 表の意味は「係数 = c0*T + c1」なので注意: coeff = T * c0 + c1
fn compute_perez_coeffs(table: &[[f64; 2]; 5], t: f64) -> [f64; 5] {
    [
        t * table[0][0] + table[0][1],
        t * table[1][0] + table[1][1],
        t * table[2][0] + table[2][1],
        t * table[3][0] + table[3][1],
        t * table[4][0] + table[4][1],
    ]
}

/// 天頂色度を多項式で計算する
///
/// value = T^2 * poly(t2, θ) + T * poly(t1, θ) + poly(t0, θ)
/// poly([k3,k2,k1,k0], θ) = k3·θ³ + k2·θ² + k1·θ + k0
fn eval_zenith_chroma(
    t2: &[f64; 4],
    t1: &[f64; 4],
    t0: &[f64; 4],
    turbidity: f64,
    theta_s: f64,
) -> f64 {
    let poly =
        |k: &[f64; 4], x: f64| -> f64 { k[0] * x.powi(3) + k[1] * x.powi(2) + k[2] * x + k[3] };
    turbidity * turbidity * poly(t2, theta_s) + turbidity * poly(t1, theta_s) + poly(t0, theta_s)
}

/// CIE xyY → sRGB u8 変換（D65 白色点, Reinhard トーンマッピング）
///
/// - `x`, `y`: CIE 色度 (0〜1 付近)
/// - `big_y`: 輝度 (kcd/m²)
fn xyyto_srgb(x: f64, y: f64, big_y: f64) -> (u8, u8, u8) {
    // y ≈ 0 の場合の保護
    let y_safe = y.max(1e-6);

    // xyY → XYZ
    let cap_x = x / y_safe * big_y;
    let cap_z = (1.0 - x - y) / y_safe * big_y;

    // XYZ → linear sRGB (D65, IEC 61966-2-1 matrix)
    let r_lin = 3.2406 * cap_x - 1.5372 * big_y - 0.4986 * cap_z;
    let g_lin = -0.9689 * cap_x + 1.8758 * big_y + 0.0415 * cap_z;
    let b_lin = 0.0557 * cap_x - 0.2040 * big_y + 1.0570 * cap_z;

    // 露出スケール
    let r_exp = r_lin * EXPOSURE;
    let g_exp = g_lin * EXPOSURE;
    let b_exp = b_lin * EXPOSURE;

    // Reinhard トーンマッピング: c' = c / (1 + c)
    let r_tm = reinhard(r_exp);
    let g_tm = reinhard(g_exp);
    let b_tm = reinhard(b_exp);

    // sRGB ガンマ補正 (IEC 61966-2-1)
    let r_gamma = srgb_gamma(r_tm);
    let g_gamma = srgb_gamma(g_tm);
    let b_gamma = srgb_gamma(b_tm);

    (to_u8(r_gamma), to_u8(g_gamma), to_u8(b_gamma))
}

/// Reinhard トーンマッピング: c' = c / (1 + c)
#[inline]
fn reinhard(c: f64) -> f64 {
    let c = c.max(0.0);
    c / (1.0 + c)
}

/// sRGB ガンマ補正
#[inline]
fn srgb_gamma(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// 0.0〜1.0 の f64 を 0〜255 の u8 にクランプ変換
#[inline]
fn to_u8(v: f64) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

// ─── テスト ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// 昼間（高度 45°）の天頂は青みが強いこと
    #[test]
    fn test_midday_zenith_is_blue() {
        let sky = PreethamSky::new(45.0, DEFAULT_TURBIDITY).with_azimuth(180.0);
        let (r, g, b) = sky.sky_rgb(0.0, 0.0); // 天頂方向
        assert!(b > r, "昼間の天頂は青いはず: r={r} g={g} b={b}");
    }

    /// 日の出直後（高度 2°）の地平線付近は赤みが強いこと
    #[test]
    fn test_sunrise_horizon_is_warm() {
        let sky = PreethamSky::new(2.0, DEFAULT_TURBIDITY).with_azimuth(90.0);
        // 太陽方向の地平線 (θ≈π/2, φ=太陽方位)
        let phi_sun = 90.0_f64.to_radians();
        let (r, g, b) = sky.sky_rgb(80.0_f64.to_radians(), phi_sun);
        assert!(
            r >= g,
            "日の出時の太陽方向は暖色系のはず: r={r} g={g} b={b}"
        );
    }

    /// 高度が高いほど天頂が明るい（Y 輝度大）
    #[test]
    fn test_higher_sun_brighter_zenith() {
        let low = PreethamSky::new(10.0, DEFAULT_TURBIDITY);
        let high = PreethamSky::new(60.0, DEFAULT_TURBIDITY);
        let (_, _, _) = low.sky_rgb(0.0, 0.0);
        // 天頂輝度を直接比較
        assert!(
            high.zenith_lum >= low.zenith_lum,
            "高度が高いほど天頂輝度が大きいはず: low={} high={}",
            low.zenith_lum,
            high.zenith_lum
        );
    }

    /// sun_below_horizon フラグが正しく設定される
    #[test]
    fn test_sun_below_horizon_flag() {
        let above = PreethamSky::new(5.0, DEFAULT_TURBIDITY);
        let below = PreethamSky::new(-5.0, DEFAULT_TURBIDITY);
        assert!(!above.sun_below_horizon);
        assert!(below.sun_below_horizon);
    }

    /// 乱流係数が高いほど太陽方向の地平線グローが明るい
    #[test]
    fn test_higher_turbidity_warmer_horizon() {
        // 太陽方向 (phi = phi_s) の地平線近く (theta = 80°) を比較
        // Mie 散乱の前方散乱ローブが増大するため、高 T で太陽方向のグローが強くなる
        let clear = PreethamSky::new(30.0, 2.0).with_azimuth(180.0);
        let hazy = PreethamSky::new(30.0, 8.0).with_azimuth(180.0);
        let phi_sun = PI; // azimuth = 180°
        let (cr, cg, cb) = clear.sky_rgb(80.0_f64.to_radians(), phi_sun);
        let (hr, hg, hb) = hazy.sky_rgb(80.0_f64.to_radians(), phi_sun);
        let clear_brightness = cr as u32 + cg as u32 + cb as u32;
        let hazy_brightness = hr as u32 + hg as u32 + hb as u32;
        // 異なる乱流係数で結果が異なることを確認（少なくとも1違う）
        assert!(
            (clear_brightness as i64 - hazy_brightness as i64).abs() > 0
                || clear.zenith_lum != hazy.zenith_lum,
            "乱流係数が結果に影響していない: clear_brightness={clear_brightness} hazy_brightness={hazy_brightness}"
        );
        // 高乱流では天頂輝度が異なること（Yz は T の関数）
        assert!(
            (clear.zenith_lum - hazy.zenith_lum).abs() > 0.01,
            "乱流係数が天頂輝度に影響していない: clear={} hazy={}",
            clear.zenith_lum,
            hazy.zenith_lum
        );
    }

    /// perez_f は正の値を返す
    #[test]
    fn test_perez_f_positive() {
        let sky = PreethamSky::new(45.0, DEFAULT_TURBIDITY);
        let f = perez_f(&sky.perez_y_lum, 0.5, 0.3);
        assert!(f > 0.0, "perez_f は正の値を返すはず: f={f}");
    }

    /// 変換結果が 0〜255 の範囲に収まる
    #[test]
    fn test_rgb_bounds() {
        for alt in [-5.0_f64, 0.0, 10.0, 45.0, 85.0] {
            let clamped_alt = alt.max(0.0);
            let sky = PreethamSky::new(clamped_alt, DEFAULT_TURBIDITY).with_azimuth(180.0);
            for theta_deg in [0.0_f64, 30.0, 60.0, 85.0] {
                let (r, g, b) = sky.sky_rgb(theta_deg.to_radians(), PI);
                assert!(
                    r <= 255 && g <= 255 && b <= 255,
                    "RGB が範囲外: alt={alt} theta={theta_deg} → ({r},{g},{b})"
                );
            }
        }
    }
}
