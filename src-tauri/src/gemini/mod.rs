// src-tauri/src/gemini/mod.rs
// Gemini AI 画像強化モジュール
// PNG バイト列を Gemini マルチモーダル API に渡し、写真的に加工した画像を返す

pub mod client;
pub mod types;

use crate::config::GeminiConfig;
use anyhow::{anyhow, Result};

/// PNG バイト列を受け取り、Gemini API で加工した PNG バイト列を返す
///
/// `config.enabled == false` または `api_key` が空の場合は
/// 元の `png_bytes` をそのまま返す（既存動作を維持）。
pub async fn enhance_image(config: &GeminiConfig, png_bytes: Vec<u8>) -> Result<Vec<u8>> {
    if !config.enabled {
        log::info!("Gemini AI 強化は無効です。元の画像を使用します");
        return Ok(png_bytes);
    }

    if config.api_key.is_empty() {
        return Err(anyhow!(
            "Gemini API キーが設定されていません。設定画面で API キーを入力してください"
        ));
    }

    log::info!("Gemini AI 強化を実行中 (model: {})", config.model_name);

    client::enhance_image(
        &config.api_key,
        &config.model_name,
        &config.enhance_prompt,
        &png_bytes,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GeminiConfig;

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

    /// enabled=false の場合は元の画像をそのまま返すこと
    #[tokio::test]
    async fn test_disabled_returns_original() {
        let original = vec![1u8, 2, 3, 4];
        let cfg = disabled_config();
        let result = enhance_image(&cfg, original.clone()).await.unwrap();
        assert_eq!(result, original, "無効時は元の画像を返すべき");
    }

    /// enabled=true かつ api_key が空の場合はエラーを返すこと
    #[tokio::test]
    async fn test_enabled_empty_key_returns_error() {
        let cfg = enabled_no_key_config();
        let result = enhance_image(&cfg, vec![0u8; 10]).await;
        assert!(result.is_err(), "APIキーが空の場合はエラーを返すべき");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("API キー"),
            "エラーメッセージに 'API キー' が含まれること: {err_msg}"
        );
    }
}
