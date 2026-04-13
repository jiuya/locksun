// src-tauri/src/gemini/types.rs
// Gemini API リクエスト / レスポンスの serde モデル

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------
// リクエスト型
// ---------------------------------------------------------------

/// generateContent API のトップレベルリクエスト
#[derive(Debug, Serialize)]
pub struct GenerateContentRequest {
    pub contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    pub generation_config: GenerationConfig,
}

/// メッセージコンテンツ（ロール + パーツ）
#[derive(Debug, Serialize)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

/// パーツ: テキスト または インライン画像
///
/// - `Text`: テキストプロンプト（指示文など）に使用する
/// - `InlineData`: base64 エンコードされた画像データに使用する
///
/// `#[serde(untagged)]` により、JSON シリアライズ時にバリアント名は付与されない。
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Part {
    Text { text: String },
    InlineData { inline_data: InlineData },
}

/// インライン画像データ（base64 エンコード）
#[derive(Debug, Serialize)]
pub struct InlineData {
    pub mime_type: String,
    pub data: String,
}

/// 生成設定
#[derive(Debug, Serialize)]
pub struct GenerationConfig {
    pub response_modalities: Vec<String>,
}

// ---------------------------------------------------------------
// レスポンス型
// ---------------------------------------------------------------

/// generateContent API のトップレベルレスポンス
#[derive(Debug, Deserialize)]
pub struct GenerateContentResponse {
    pub candidates: Option<Vec<Candidate>>,
    pub error: Option<ApiError>,
}

/// 候補（生成結果）
#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub content: Option<ResponseContent>,
}

/// レスポンスコンテンツ
#[derive(Debug, Deserialize)]
pub struct ResponseContent {
    pub parts: Option<Vec<ResponsePart>>,
}

/// レスポンスパーツ（テキスト or 画像）
#[derive(Debug, Deserialize)]
pub struct ResponsePart {
    pub text: Option<String>,
    pub inline_data: Option<ResponseInlineData>,
}

/// レスポンスインライン画像
#[derive(Debug, Deserialize)]
pub struct ResponseInlineData {
    pub mime_type: Option<String>,
    pub data: Option<String>,
}

/// API エラー情報
#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub code: Option<u32>,
    pub message: Option<String>,
}
