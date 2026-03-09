// src-tauri/src/renderer/palette.rs
// 太陽高度角に応じたカラーパレットの定義
//
// 各フェーズの「空の最上部色」「地平線色」「環境光強度」を定義する
// 中間値は線形補間により自動計算される

use image::Rgb;

/// RGB カラー (0-255)
#[derive(Debug, Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn to_rgb(self) -> Rgb<u8> {
        Rgb([self.0, self.1, self.2])
    }

    /// 線形補間 t: 0.0=self, 1.0=other
    pub fn lerp(self, other: Color, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color(
            (self.0 as f64 + (other.0 as f64 - self.0 as f64) * t) as u8,
            (self.1 as f64 + (other.1 as f64 - self.1 as f64) * t) as u8,
            (self.2 as f64 + (other.2 as f64 - self.2 as f64) * t) as u8,
        )
    }
}

/// 空のレンダリングに使うカラーキー
#[derive(Debug, Clone, Copy)]
pub struct SkyColors {
    /// 天頂側の色
    pub zenith: Color,
    /// 地平線側の色
    pub horizon: Color,
    /// 太陽の色（ハロー）
    pub sun_halo: Color,
    /// 太陽本体の色
    pub sun_disk: Color,
    /// 環境光の強度 (0.0-1.0)
    pub ambient: f64,
}

impl SkyColors {
    /// 太陽高度角 (度) からカラーパレットを補間して返す
    pub fn from_altitude(altitude: f64) -> Self {
        // キーフレーム定義 (altitude, SkyColors)
        let keyframes: &[(f64, SkyColors)] = &[
            // 天文夜
            (
                -18.0,
                SkyColors {
                    zenith: Color(0, 0, 15),
                    horizon: Color(5, 5, 25),
                    sun_halo: Color(0, 0, 0),
                    sun_disk: Color(0, 0, 0),
                    ambient: 0.0,
                },
            ),
            // 市民薄明開始
            (
                -6.0,
                SkyColors {
                    zenith: Color(10, 10, 50),
                    horizon: Color(40, 20, 60),
                    sun_halo: Color(80, 30, 20),
                    sun_disk: Color(80, 30, 20),
                    ambient: 0.05,
                },
            ),
            // 日の出・日の入り直前
            (
                -1.0,
                SkyColors {
                    zenith: Color(20, 40, 120),
                    horizon: Color(200, 80, 20),
                    sun_halo: Color(255, 120, 0),
                    sun_disk: Color(255, 180, 50),
                    ambient: 0.2,
                },
            ),
            // 日の出・日の入り直後 (ゴールデンアワー)
            (
                2.0,
                SkyColors {
                    zenith: Color(80, 100, 180),
                    horizon: Color(255, 140, 30),
                    sun_halo: Color(255, 180, 60),
                    sun_disk: Color(255, 240, 100),
                    ambient: 0.6,
                },
            ),
            // 朝・夕 (低角度)
            (
                10.0,
                SkyColors {
                    zenith: Color(80, 140, 220),
                    horizon: Color(180, 210, 240),
                    sun_halo: Color(255, 240, 180),
                    sun_disk: Color(255, 255, 200),
                    ambient: 0.9,
                },
            ),
            // 昼間
            (
                45.0,
                SkyColors {
                    zenith: Color(20, 90, 200),
                    horizon: Color(130, 190, 240),
                    sun_halo: Color(255, 255, 220),
                    sun_disk: Color(255, 255, 245),
                    ambient: 1.0,
                },
            ),
            // 高高度
            (
                90.0,
                SkyColors {
                    zenith: Color(10, 60, 170),
                    horizon: Color(100, 160, 220),
                    sun_halo: Color(255, 255, 230),
                    sun_disk: Color(255, 255, 255),
                    ambient: 1.0,
                },
            ),
        ];

        Self::interpolate(keyframes, altitude)
    }

    fn interpolate(frames: &[(f64, SkyColors)], alt: f64) -> SkyColors {
        if alt <= frames.first().unwrap().0 {
            return frames.first().unwrap().1;
        }
        if alt >= frames.last().unwrap().0 {
            return frames.last().unwrap().1;
        }
        for i in 0..frames.len() - 1 {
            let (a0, c0) = frames[i];
            let (a1, c1) = frames[i + 1];
            if alt >= a0 && alt <= a1 {
                let t = (alt - a0) / (a1 - a0);
                return SkyColors {
                    zenith: c0.zenith.lerp(c1.zenith, t),
                    horizon: c0.horizon.lerp(c1.horizon, t),
                    sun_halo: c0.sun_halo.lerp(c1.sun_halo, t),
                    sun_disk: c0.sun_disk.lerp(c1.sun_disk, t),
                    ambient: c0.ambient + (c1.ambient - c0.ambient) * t,
                };
            }
        }
        frames.last().unwrap().1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_night_is_dark() {
        let c = SkyColors::from_altitude(-20.0);
        assert!(c.ambient < 0.1);
        assert!(c.zenith.0 < 20 && c.zenith.1 < 20);
    }

    #[test]
    fn test_midday_is_blue() {
        let c = SkyColors::from_altitude(60.0);
        assert!(c.zenith.2 > c.zenith.0, "昼間の天頂は青いはず");
        assert!(c.ambient > 0.9);
    }
}
