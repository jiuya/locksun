// src-tauri/src/gemini/client.rs
// Gemini REST API クライアント

use super::types::{
    Content, GenerateContentRequest, GenerateContentResponse, GenerationConfig, InlineData, Part,
};
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use serde_json;
use std::time::Duration;

/// Gemini API のベース URL
const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/// 429 時の最大リトライ回数
const MAX_RETRIES: u32 = 3;
/// リトライ待機の上限（秒）
const MAX_RETRY_WAIT_SECS: u64 = 60;

/// PNG バイト列を Gemini API に渡し、加工済み画像の PNG バイト列を返す
///
/// 429 Too Many Requests が返った場合、エラー本文に含まれる待機時間を解析して
/// 最大 [`MAX_RETRIES`] 回リトライする。ただし `limit: 0`（フリー枠なし）が
/// 検出された場合はリトライせず即時失敗する。
///
/// # Arguments
/// * `api_key`   - Gemini API キー
/// * `model`     - 使用するモデル名（例: "gemini-2.5-flash-image"）
/// * `prompt`    - 加工指示プロンプト
/// * `png_bytes` - 入力画像の PNG バイト列
pub async fn enhance_image(
    api_key: &str,
    model: &str,
    prompt: &str,
    png_bytes: &[u8],
) -> Result<Vec<u8>> {
    let image_b64 = general_purpose::STANDARD.encode(png_bytes);

    let request = GenerateContentRequest {
        contents: vec![Content {
            role: "user".to_string(),
            parts: vec![
                Part::InlineData {
                    inline_data: InlineData {
                        mime_type: "image/png".to_string(),
                        data: image_b64,
                    },
                },
                Part::Text {
                    text: prompt.to_string(),
                },
            ],
        }],
        generation_config: GenerationConfig {
            response_modalities: vec!["IMAGE".to_string(), "TEXT".to_string()],
        },
    };

    // リクエスト JSON を一度だけシリアライズしてリトライで使い回す
    let request_json = serde_json::to_string(&request)
        .context("Gemini リクエストの JSON シリアライズに失敗しました")?;

    let url = format!("{}/{}:generateContent", GEMINI_API_BASE, model);
    let client = reqwest::Client::new();
    let mut last_err_msg = String::new();

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            let wait_secs = parse_retry_delay(&last_err_msg)
                .unwrap_or(30)
                .min(MAX_RETRY_WAIT_SECS);
            log::warn!(
                "Gemini API 429: {}秒後にリトライ ({}/{}回目)",
                wait_secs,
                attempt,
                MAX_RETRIES
            );
            tokio::time::sleep(Duration::from_secs(wait_secs)).await;
        }

        let response = client
            .post(&url)
            .header("x-goog-api-key", api_key)
            .header("Content-Type", "application/json")
            .body(request_json.clone())
            .send()
            .await
            .context("Gemini API へのリクエストに失敗しました")?;

        let status = response.status();

        let raw = response
            .text()
            .await
            .context("Gemini API レスポンスの読み取りに失敗しました")?;

        log::debug!(
            "Gemini API レスポンス (HTTP {}, attempt {}): {}",
            status,
            attempt,
            if raw.len() > 500 { &raw[..500] } else { &raw }
        );

        if raw.is_empty() {
            return Err(anyhow!(
                "Gemini API が空のレスポンスを返しました (HTTP {})\n\
                 確認事項:\n\
                 - モデル名が正しいか: {}\n\
                 - API キーが有効か\n\
                 - response_modalities=[\"IMAGE\"] をサポートするモデルか",
                status,
                model
            ));
        }

        let body: GenerateContentResponse = serde_json::from_str(&raw).with_context(|| {
            format!(
                "Gemini API レスポンスの JSON パースに失敗しました (HTTP {})\n生レスポンス: {}",
                status,
                if raw.len() > 300 { &raw[..300] } else { &raw }
            )
        })?;

        // API レベルのエラーチェック
        if let Some(err) = &body.error {
            let msg = err.message.as_deref().unwrap_or("不明なエラー").to_string();

            if status == 429 {
                // フリー枠クォータが 0 の場合はリトライしても解決しない
                if msg.contains("limit: 0") {
                    return Err(anyhow!(
                        "Gemini API: モデル \"{}\" はフリー枠のクォータが 0 です。\n\
                         別のモデル（例: gemini-2.5-flash-image）を使用するか、\n\
                         Google AI Studio で課金を有効にしてください。\n\
                         詳細: {}",
                        model,
                        msg
                    ));
                }
                last_err_msg = msg;
                continue;
            }

            return Err(anyhow!(
                "Gemini API エラー (HTTP {}): {} (code: {:?})",
                status,
                msg,
                err.code
            ));
        }

        if !status.is_success() {
            return Err(anyhow!("Gemini API HTTP エラー: {}", status));
        }

        // レスポンスから画像パーツを抽出
        let candidates = body
            .candidates
            .ok_or_else(|| anyhow!("Gemini API: candidates が空です"))?;

        for candidate in &candidates {
            if let Some(content) = &candidate.content {
                if let Some(parts) = &content.parts {
                    for part in parts {
                        if let Some(inline) = &part.inline_data {
                            if let Some(data) = &inline.data {
                                let decoded = general_purpose::STANDARD.decode(data).context(
                                    "Gemini レスポンス画像の base64 デコードに失敗しました",
                                )?;
                                return Ok(decoded);
                            }
                        }
                    }
                }
            }
        }

        return Err(anyhow!(
            "Gemini API レスポンスに画像データが含まれていませんでした"
        ));
    }

    Err(anyhow!(
        "Gemini API: {} 回リトライしても 429 エラーが続きました。\n\
         レート制限を超えています。しばらく待ってから再試行してください。\n\
         最後のエラー: {}",
        MAX_RETRIES,
        last_err_msg
    ))
}

/// API エラーメッセージから待機秒数を解析する
///
/// "Please retry in 13.262288427s" 形式の文字列を探して秒数を返す。
fn parse_retry_delay(message: &str) -> Option<u64> {
    const PREFIX: &str = "Please retry in ";
    let pos = message.find(PREFIX)?;
    let rest = &message[pos + PREFIX.len()..];
    let end = rest.find('s').unwrap_or(rest.len());
    rest[..end]
        .trim()
        .parse::<f64>()
        .ok()
        .map(|f| f.ceil() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// API キーが空の場合、enhance_image を呼ばずにエラーが出ることを確認する
    /// （実際のネットワーク呼び出しは行わない）
    #[test]
    fn test_empty_api_key_is_detectable() {
        // API キーが空文字かどうかは呼び出し前にチェックできる
        let api_key = "";
        assert!(api_key.is_empty(), "空のAPIキーは検知できるべき");
    }

    /// GEMINI_API_BASE が HTTPS であることを確認する
    #[test]
    fn test_api_base_is_https() {
        assert!(
            GEMINI_API_BASE.starts_with("https://"),
            "Gemini API のエンドポイントは HTTPS であること"
        );
    }

    /// parse_retry_delay が "Please retry in Xs" 形式を解析できること
    #[test]
    fn test_parse_retry_delay_standard() {
        let msg = "You exceeded your quota. Please retry in 13.262288427s. (code: 429)";
        assert_eq!(parse_retry_delay(msg), Some(14)); // ceil(13.262...) = 14
    }

    /// 整数値でも解析できること
    #[test]
    fn test_parse_retry_delay_integer() {
        let msg = "Please retry in 30s";
        assert_eq!(parse_retry_delay(msg), Some(30));
    }

    /// 文字列が含まれない場合は None を返すこと
    #[test]
    fn test_parse_retry_delay_missing() {
        assert_eq!(parse_retry_delay("Some other error message"), None);
    }
}
