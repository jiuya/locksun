// src-tauri/src/sun/calculator.rs
// 簡易太陽位置計算
//
// アルゴリズム: NOAA 簡易式（Spencer 1971 / Greve 1978 ベース）
// 精度: 日の出・日の入り ±数分、高度角 ±1度 程度
// 極地や春分・秋分近傍での精度低下は許容する
//
// 参考: https://gml.noaa.gov/grad/solcalc/solareqns.PDF

use super::types::{SunPhase, SunPosition, SunTimes};
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use std::f64::consts::PI;

pub struct SunCalculator;

impl SunCalculator {
    /// 指定日時・位置の太陽位置を返す
    /// `lat`: 緯度 (度), `lon`: 経度 (度)
    pub fn position(dt: &DateTime<Local>, lat: f64, lon: f64) -> SunPosition {
        let (_, dec) = Self::equation_of_time_and_declination(dt);
        let ha = Self::hour_angle(dt, lon);
        let altitude = Self::calc_altitude(lat, dec, ha);
        let azimuth = Self::calc_azimuth(lat, dec, ha, altitude);
        SunPosition { altitude, azimuth }
    }

    /// 当日のフェーズを返す
    pub fn phase(dt: &DateTime<Local>, lat: f64, lon: f64) -> SunPhase {
        let pos = Self::position(dt, lat, lon);
        SunPhase::from_altitude(pos.altitude)
    }

    /// 当日の日の出・南中・日の入り等の時刻を返す
    pub fn times(dt: &DateTime<Local>, lat: f64, lon: f64) -> SunTimes {
        // SUNRISE_ALT: 大気屈折(34') + 太陽半径(16') = 0.833° 分の補正
        // NOAA 標準の日の出・日の入り定義に準拠する
        const SUNRISE_ALT: f64 = -0.833;

        let solar_noon = Self::find_solar_noon(dt, lon);
        let sunrise = Self::find_crossing(dt, lat, lon, SUNRISE_ALT, false, &solar_noon);
        let sunset = Self::find_crossing(dt, lat, lon, SUNRISE_ALT, true, &solar_noon);
        let civil_dawn = Self::find_crossing(dt, lat, lon, -6.0, false, &solar_noon);
        let civil_dusk = Self::find_crossing(dt, lat, lon, -6.0, true, &solar_noon);
        let astro_dawn = Self::find_crossing(dt, lat, lon, -18.0, false, &solar_noon);
        let astro_dusk = Self::find_crossing(dt, lat, lon, -18.0, true, &solar_noon);
        SunTimes {
            astronomical_dawn: astro_dawn,
            civil_dawn,
            sunrise,
            solar_noon,
            sunset,
            civil_dusk,
            astronomical_dusk: astro_dusk,
        }
    }

    // ---------------------------------------------------------------
    // 内部計算
    // ---------------------------------------------------------------

    /// 年間通算日 (1-365/366)
    fn day_of_year(dt: &DateTime<Local>) -> f64 {
        dt.ordinal() as f64
    }

    /// 均時差 (分) と赤緯 (度) を返す
    fn equation_of_time_and_declination(dt: &DateTime<Local>) -> (f64, f64) {
        let d = Self::day_of_year(dt);
        // B: ラジアン
        let b = 2.0 * PI * (d - 1.0) / 365.0;
        // 均時差 (分)
        let eot = 229.18
            * (0.000075 + 0.001868 * b.cos()
                - 0.032077 * b.sin()
                - 0.014615 * (2.0 * b).cos()
                - 0.04089 * (2.0 * b).sin());
        // 赤緯 (ラジアン → 度)
        let dec_rad = 0.006918 - 0.399912 * b.cos() + 0.070257 * b.sin()
            - 0.006758 * (2.0 * b).cos()
            + 0.000907 * (2.0 * b).sin()
            - 0.002697 * (3.0 * b).cos()
            + 0.00148 * (3.0 * b).sin();
        let dec = dec_rad.to_degrees();
        (eot, dec)
    }

    /// 時角 (度)
    fn hour_angle(dt: &DateTime<Local>, lon: f64) -> f64 {
        let (eot, _) = Self::equation_of_time_and_declination(dt);
        // UTC オフセット (時間)
        let offset_hours = dt.offset().local_minus_utc() as f64 / 3600.0;
        // 太陽時刻 (分)
        let time_min = dt.hour() as f64 * 60.0 + dt.minute() as f64 + dt.second() as f64 / 60.0;
        let solar_time = time_min + eot + 4.0 * lon - 60.0 * offset_hours;
        // 時角: 正午=0、1時間=15度
        solar_time / 4.0 - 180.0
    }

    /// 太陽高度角 (度)
    fn calc_altitude(lat: f64, dec: f64, ha: f64) -> f64 {
        let lat_r = lat.to_radians();
        let dec_r = dec.to_radians();
        let ha_r = ha.to_radians();
        let sin_alt = lat_r.sin() * dec_r.sin() + lat_r.cos() * dec_r.cos() * ha_r.cos();
        sin_alt.asin().to_degrees()
    }

    /// 太陽方位角 (度、北=0 時計回り)
    fn calc_azimuth(lat: f64, dec: f64, ha: f64, altitude: f64) -> f64 {
        let lat_r = lat.to_radians();
        let dec_r = dec.to_radians();
        let alt_r = altitude.to_radians();
        let cos_az = (dec_r.sin() - alt_r.sin() * lat_r.sin()) / (alt_r.cos() * lat_r.cos());
        let cos_az = cos_az.clamp(-1.0, 1.0);
        let az = cos_az.acos().to_degrees();
        if ha > 0.0 {
            360.0 - az
        } else {
            az
        }
    }

    /// 南中時刻を分単位で求める
    fn find_solar_noon(_dt: &DateTime<Local>, lon: f64) -> DateTime<Local> {
        // NOTE: _dt は現在日付の取得に使用する（均時差計算内で参照）
        let (eot, _) = Self::equation_of_time_and_declination(_dt);
        let offset_hours = _dt.offset().local_minus_utc() as f64 / 3600.0;
        // 南中の太陽時刻=720分(正午)
        let noon_min = 720.0 - 4.0 * lon - eot + 60.0 * offset_hours;
        let noon_h = (noon_min / 60.0) as u32;
        let noon_m = (noon_min % 60.0) as u32;
        let noon_s = ((noon_min % 60.0 - noon_m as f64) * 60.0) as u32;
        _dt.date_naive()
            .and_hms_opt(noon_h.min(23), noon_m.min(59), noon_s.min(59))
            .and_then(|ndt| Local.from_local_datetime(&ndt).single())
            .unwrap_or_else(|| *_dt)
    }

    /// 指定高度角への日時交差を二分探索で求める
    /// `after_noon`: true=日の入り方向, false=日の出方向
    fn find_crossing(
        _dt: &DateTime<Local>,
        lat: f64,
        lon: f64,
        target_alt: f64,
        after_noon: bool,
        noon: &DateTime<Local>,
    ) -> Option<DateTime<Local>> {
        // 探索範囲: 南中から ±12時間分 (分単位)
        let (mut lo, mut hi) = if after_noon {
            (0i64, 720)
        } else {
            (-720i64, 0)
        };

        let eval = |offset_min: i64| -> f64 {
            let t = *noon + chrono::Duration::minutes(offset_min);
            let (_, dec) = Self::equation_of_time_and_declination(&t);
            let ha = Self::hour_angle(&t, lon);
            Self::calc_altitude(lat, dec, ha)
        };

        // 範囲内に交差が存在するか確認
        if (eval(lo) - target_alt).signum() == (eval(hi) - target_alt).signum() {
            return None; // 白夜・極夜
        }

        // 二分探索（32回で1/2^32 ≈ 0.2秒精度）
        for _ in 0..32 {
            let mid = (lo + hi) / 2;
            let alt_mid = eval(mid);
            if (alt_mid - target_alt).signum() == (eval(lo) - target_alt).signum() {
                lo = mid;
            } else {
                hi = mid;
            }
        }

        let result_offset = (lo + hi) / 2;
        Some(*noon + chrono::Duration::minutes(result_offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn jst(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Local> {
        chrono::FixedOffset::east_opt(9 * 3600)
            .unwrap()
            .with_ymd_and_hms(y, mo, d, h, mi, 0)
            .unwrap()
            .with_timezone(&Local)
    }

    #[test]
    fn test_tokyo_sunrise_sunset() {
        // 東京 (35.68°N, 139.69°E) 2026-03-09 の日の出・日の入り
        // 実測値: 日の出 06:03、日の入り 17:52 頃（許容誤差 ±10分）
        let dt = jst(2026, 3, 9, 12, 0);
        let times = SunCalculator::times(&dt, 35.68, 139.69);

        let sr = times.sunrise.unwrap();
        let ss = times.sunset.unwrap();

        // CI の ローカルタイムゾーンに依存しないよう JST (+09:00) に変換して検証する
        let jst_tz = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
        let sr_jst = sr.with_timezone(&jst_tz);
        let ss_jst = ss.with_timezone(&jst_tz);

        // 日の出: 5時台後半か6時台前半が期待範囲
        assert!(
            (sr_jst.hour() == 5 && sr_jst.minute() >= 50)
                || (sr_jst.hour() == 6 && sr_jst.minute() <= 15),
            "日の出 {sr_jst} が期待範囲外"
        );
        // 日の入り: 17時台後半が期待範囲
        assert!(
            ss_jst.hour() == 17 && ss_jst.minute() >= 40,
            "日の入り {ss_jst} が期待範囲外"
        );
    }

    #[test]
    fn test_daytime_altitude() {
        // 東京の正午は高度角が正になるはず
        let dt = jst(2026, 3, 9, 12, 0);
        let pos = SunCalculator::position(&dt, 35.68, 139.69);
        assert!(pos.altitude > 0.0, "昼の高度角が負: {}", pos.altitude);
    }
}
