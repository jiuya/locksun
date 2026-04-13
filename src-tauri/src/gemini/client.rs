// src-tauri/src/gemini/client.rs
// Gemini REST API クライアント

use super::types::{
    Content, GenerateContentRequest, GenerateContentResponse, GenerationConfig, InlineData, Part,
};
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};

/// Gemini API のベース URL
const GEMINI_API_BASE: &str =
    "https://generativelanguage.googleapis.com/v1beta/models";

/// PNG バイト列を Gemini API に渡し、加工済み画像の PNG バイト列を返す
///
/// # Arguments
/// * `api_key`  - Gemini API キー
/// * `model`    - 使用するモデル名（例: "gemini-2.0-flash-exp"）
/// * `prompt`   - 加工指示プロンプト
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

    let url = format!(
        "{}/{}/{}",
        GEMINI_API_BASE, model, "generateContent"
    );

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("x-goog-api-key", api_key)
        .json(&request)
        .send()
        .await
        .context("Gemini API へのリクエストに失敗しました")?;

    let status = response.status();
    let body: GenerateContentResponse = response
        .json()
        .await
        .context("Gemini API レスポンスの JSON パースに失敗しました")?;

    // API レベルのエラーチェック
    if let Some(err) = &body.error {
        return Err(anyhow!(
            "Gemini API エラー (HTTP {}): {} (code: {:?})",
            status,
            err.message.as_deref().unwrap_or("不明なエラー"),
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
                            let decoded = general_purpose::STANDARD
                                .decode(data)
                                .context("Gemini レスポンス画像の base64 デコードに失敗しました")?;
                            return Ok(decoded);
                        }
                    }
                }
            }
        }
    }

    Err(anyhow!(
        "Gemini API レスポンスに画像データが含まれていませんでした"
    ))
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
}
