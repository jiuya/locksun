// src-tauri/tests/sun_calculator_test.rs
// 太陽計算の統合テスト

use chrono::{Local, TimeZone};
use locksun_lib::sun::{SunCalculator, SunPhase};

fn jst(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> chrono::DateTime<Local> {
    chrono::FixedOffset::east_opt(9 * 3600)
        .unwrap()
        .with_ymd_and_hms(y, mo, d, h, mi, 0)
        .unwrap()
        .with_timezone(&Local)
}

#[test]
fn test_midnight_is_night() {
    let dt = jst(2026, 3, 9, 0, 0);
    let phase = SunCalculator::phase(&dt, 35.68, 139.69);
    assert_eq!(phase, SunPhase::AstronomicalNight);
}

#[test]
fn test_noon_is_daytime() {
    let dt = jst(2026, 3, 9, 12, 0);
    let phase = SunCalculator::phase(&dt, 35.68, 139.69);
    assert_eq!(phase, SunPhase::Daytime);
}

#[test]
fn test_sunrise_is_golden_hour() {
    // 日の出直後 (06:10 JST) はゴールデンアワーか Daytime
    let dt = jst(2026, 3, 9, 6, 10);
    let phase = SunCalculator::phase(&dt, 35.68, 139.69);
    assert!(
        phase == SunPhase::GoldenHour
            || phase == SunPhase::Daytime
            || phase == SunPhase::CivilTwilight,
        "日の出直後のフェーズが期待外: {phase:?}"
    );
}

#[test]
fn test_sun_times_order() {
    // 日の出 < 南中 < 日の入り の順序確認
    let dt = jst(2026, 3, 9, 12, 0);
    let times = SunCalculator::times(&dt, 35.68, 139.69);
    assert!(times.sunrise.is_some(), "日の出が None");
    assert!(times.sunset.is_some(), "日の入りが None");
    assert!(times.sunrise.unwrap() < times.solar_noon);
    assert!(times.solar_noon < times.sunset.unwrap());
}

#[test]
fn test_sapporo_winter_solstice() {
    // 札幌 (43.06°N) 冬至 - 日の出・日の入りが存在することを確認
    let dt = jst(2025, 12, 22, 12, 0);
    let times = SunCalculator::times(&dt, 43.06, 141.35);
    assert!(times.sunrise.is_some());
    assert!(times.sunset.is_some());
}
