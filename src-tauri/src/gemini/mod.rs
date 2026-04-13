// src-tauri/src/gemini/mod.rs
// Gemini AI 画像強化モジュール
// PNG バイト列を Gemini マルチモーダル API に渡し、写真的に加工した画像を返す

pub mod client;
pub mod types;

use crate::config::GeminiConfig;
use crate::sun::{SunPhase, SunPosition};
use anyhow::{anyhow, Result};

/// 太陽位置に基づいた時間帯コンテキストをプロンプトの先頭に付加して返す
///
/// Gemini が夜景に青空を生成しないよう、フェーズごとに明示的な制約を追加する。
fn build_context_prompt(base_prompt: &str, pos: &SunPosition) -> String {
    let phase = SunPhase::from_altitude(pos.altitude);
    let context = match phase {
        SunPhase::AstronomicalNight => format!(
            "CRITICAL CONSTRAINT: This is a NIGHT scene. \
             The sun is far below the horizon (altitude: {:.1}°). \
             The sky MUST be dark/black. Absolutely NO blue sky, NO daylight, NO sun visible in the sky. \
             Stars should be visible. Only dark night colors are appropriate.",
            pos.altitude
        ),
        SunPhase::AstronomicalTwilight => format!(
            "CRITICAL CONSTRAINT: This is an astronomical twilight scene \
             (very late night or very early pre-dawn). \
             The sun is far below the horizon (altitude: {:.1}°). \
             The sky must be very dark, nearly black. \
             A very faint gradient may appear only near the exact horizon. \
             Absolutely NO blue daytime sky.",
            pos.altitude
        ),
        SunPhase::CivilTwilight => format!(
            "CRITICAL CONSTRAINT: This is a civil twilight scene \
             (just before sunrise or just after sunset). \
             The sun is just below the horizon (altitude: {:.1}°). \
             The sky should be deep dark blue-purple with a subtle warm glow only near the horizon. \
             NO bright blue daytime sky.",
            pos.altitude
        ),
        SunPhase::GoldenHour => format!(
            "IMPORTANT: This is a golden hour scene \
             (just after sunrise or just before sunset). \
             The sun is near the horizon (altitude: {:.1}°, azimuth: {:.1}°). \
             The sky should feature warm golden, orange, amber, and pink tones with dramatic lighting.",
            pos.altitude, pos.azimuth
        ),
        SunPhase::Daytime => format!(
            "Scene context: This is a daytime scene. \
             The sun is above the horizon (altitude: {:.1}°, azimuth: {:.1}°). \
             Natural blue sky is appropriate.",
            pos.altitude, pos.azimuth
        ),
    };
    format!("{context} {base_prompt}")
}

/// PNG バイト列を受け取り、Gemini API で加工した PNG バイト列を返す
///
/// `pos` を元に時間帯コンテキストをプロンプトに自動付加する。
/// `config.enabled == false` または `api_key` が空の場合は
/// 元の `png_bytes` をそのまま返す（既存動作を維持）。
pub async fn enhance_image(
    config: &GeminiConfig,
    pos: &SunPosition,
    png_bytes: Vec<u8>,
) -> Result<Vec<u8>> {
    if !config.enabled {
        log::info!("Gemini AI 強化は無効です。元の画像を使用します");
        return Ok(png_bytes);
    }

    if config.api_key.is_empty() {
        return Err(anyhow!(
            "Gemini API キーが設定されていません。設定画面で API キーを入力してください"
        ));
    }

    let prompt = build_context_prompt(&config.enhance_prompt, pos);
    log::info!(
        "Gemini AI 強化を実行中 (model: {}, phase: {:?}, altitude: {:.1}°)",
        config.model_name,
        SunPhase::from_altitude(pos.altitude),
        pos.altitude
    );
    log::debug!("Gemini プロンプト: {prompt}");

    client::enhance_image(&config.api_key, &config.model_name, &prompt, &png_bytes).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GeminiConfig;
    use crate::sun::SunPosition;

    fn disabled_config() -> GeminiConfig {
        GeminiConfig {
            api_key: String::new(),
            model_name: "gemini-2.5-flash-image".to_string(),
            enhance_prompt: "test prompt".to_string(),
            enabled: false,
        }
    }

    fn enabled_no_key_config() -> GeminiConfig {
        GeminiConfig {
            api_key: String::new(),
            model_name: "gemini-2.5-flash-image".to_string(),
            enhance_prompt: "test prompt".to_string(),
            enabled: true,
        }
    }

    fn night_pos() -> SunPosition {
        SunPosition {
            altitude: -30.0,
            azimuth: 0.0,
        }
    }

    fn day_pos() -> SunPosition {
        SunPosition {
            altitude: 45.0,
            azimuth: 180.0,
        }
    }

    /// enabled=false の場合は元の画像をそのまま返すこと
    #[tokio::test]
    async fn test_disabled_returns_original() {
        let original = vec![1u8, 2, 3, 4];
        let cfg = disabled_config();
        let result = enhance_image(&cfg, &night_pos(), original.clone())
            .await
            .unwrap();
        assert_eq!(result, original, "無効時は元の画像を返すべき");
    }

    /// enabled=true かつ api_key が空の場合はエラーを返すこと
    #[tokio::test]
    async fn test_enabled_empty_key_returns_error() {
        let cfg = enabled_no_key_config();
        let result = enhance_image(&cfg, &day_pos(), vec![0u8; 10]).await;
        assert!(result.is_err(), "APIキーが空の場合はエラーを返すべき");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("API キー"),
            "エラーメッセージに 'API キー' が含まれること: {err_msg}"
        );
    }

    /// 夜間フェーズではプロンプトにナイト制約が含まれること
    #[test]
    fn test_build_context_prompt_night() {
        let pos = SunPosition {
            altitude: -25.0,
            azimuth: 0.0,
        };
        let prompt = build_context_prompt("base prompt", &pos);
        assert!(
            prompt.contains("NIGHT"),
            "夜間には NIGHT 制約が含まれること"
        );
        assert!(
            prompt.contains("base prompt"),
            "ベースプロンプトが含まれること"
        );
    }

    /// 昼間フェーズではプロンプトに daytime コンテキストが含まれること
    #[test]
    fn test_build_context_prompt_daytime() {
        let pos = SunPosition {
            altitude: 30.0,
            azimuth: 180.0,
        };
        let prompt = build_context_prompt("base prompt", &pos);
        assert!(
            prompt.contains("daytime"),
            "昼間には daytime コンテキストが含まれること"
        );
        assert!(
            prompt.contains("base prompt"),
            "ベースプロンプトが含まれること"
        );
    }
}
