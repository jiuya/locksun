// src-tauri/src/sun/types.rs
// 太陽計算結果の型定義

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// 太陽の位置（ある瞬間）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SunPosition {
    /// 高度角 (度): -90〜+90  正値=地平線より上
    pub altitude: f64,
    /// 方位角 (度): 0=北, 90=東, 180=南, 270=西
    pub azimuth: f64,
}

/// 当日の日の出・南中・日の入り時刻
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SunTimes {
    /// 天文薄明開始（太陽高度 -18°）
    pub astronomical_dawn: Option<DateTime<Local>>,
    /// 市民薄明開始（太陽高度 -6°）
    pub civil_dawn: Option<DateTime<Local>>,
    /// 日の出（太陽高度 0°）
    pub sunrise: Option<DateTime<Local>>,
    /// 南中（太陽が最も高い時刻）
    pub solar_noon: DateTime<Local>,
    /// 日の入り（太陽高度 0°）
    pub sunset: Option<DateTime<Local>>,
    /// 市民薄暮終了（太陽高度 -6°）
    pub civil_dusk: Option<DateTime<Local>>,
    /// 天文薄暮終了（太陽高度 -18°）
    pub astronomical_dusk: Option<DateTime<Local>>,
}

/// 現在の空の状態フェーズ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SunPhase {
    /// 天文夜（-18° 以下）
    AstronomicalNight,
    /// 天文薄明（-18° 〜 -6°）
    AstronomicalTwilight,
    /// 市民薄明（-6° 〜 0°）
    CivilTwilight,
    /// 夜明け・夕暮け（0° 〜 6°）
    GoldenHour,
    /// 昼間（6° 以上）
    Daytime,
}

impl SunPhase {
    pub fn from_altitude(alt: f64) -> Self {
        match alt {
            a if a >= 6.0  => Self::Daytime,
            a if a >= 0.0  => Self::GoldenHour,
            a if a >= -6.0 => Self::CivilTwilight,
            a if a >= -18.0 => Self::AstronomicalTwilight,
            _              => Self::AstronomicalNight,
        }
    }
}
