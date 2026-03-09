// src-tauri/src/sun/mod.rs
// 太陽位置計算モジュール
// 精度目標: 日の出・日の入り時刻が現実と ±数分以内であること

pub mod calculator;
pub mod types;

pub use calculator::SunCalculator;
pub use types::{SunPhase, SunPosition, SunTimes};
