// src-tauri/src/config/types.rs

use serde::{Deserialize, Serialize};

/// アプリ全体の設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 位置情報
    pub location: LocationConfig,
    /// 更新設定
    pub update: UpdateConfig,
    /// 画像設定
    pub image: ImageConfig,
    /// 動作設定
    pub behavior: BehaviorConfig,
    /// Gemini AI 強化設定
    #[serde(default)]
    pub gemini: GeminiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationConfig {
    /// 緯度 (-90.0 〜 +90.0)
    pub latitude: f64,
    /// 経度 (-180.0 〜 +180.0)
    pub longitude: f64,
    /// 位置名称（表示用）
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// 更新間隔 (秒)
    pub interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    /// 出力画像の幅 (px)
    pub width: u32,
    /// 出力画像の高さ (px)
    pub height: u32,
    /// 星を表示するか（夜間）
    pub show_stars: bool,
    /// 雲エフェクトを表示するか
    pub show_clouds: bool,
    /// 水の深さ (0.0-1.0: 0=浅い青緑, 1=深い青)
    #[serde(default = "default_water_depth")]
    pub water_depth: f64,
}

fn default_water_depth() -> f64 {
    0.7
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// Windows スタートアップに登録するか
    pub autostart: bool,
}

/// Gemini AI 強化設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    /// Gemini API キー（空文字の場合は機能を無効化）
    pub api_key: String,
    /// 使用モデル名
    pub model_name: String,
    /// 画像加工プロンプト
    pub enhance_prompt: String,
    /// AI 強化を有効にするか
    pub enabled: bool,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model_name: "gemini-2.5-flash-image".to_string(),
            enhance_prompt: "Enhance this sky image to look photorealistic, like a high-quality photograph. Preserve the sun position and sky colors but add natural cloud textures, atmospheric haze, and photographic quality.".to_string(),
            enabled: false,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            location: LocationConfig {
                latitude: 35.6762,
                longitude: 139.6503,
                name: "東京".to_string(),
            },
            update: UpdateConfig { interval_secs: 300 },
            image: ImageConfig {
                width: 1920,
                height: 1080,
                show_stars: true,
                show_clouds: false,
                water_depth: 0.7, // 中程度の深さ（標準的な湖の深さ）
            },
            behavior: BehaviorConfig { autostart: false },
            gemini: GeminiConfig {
                api_key: String::new(),
                model_name: "gemini-2.5-flash-image".to_string(),
                // NOTE: この文字列は src/api/tauri_commands.ts の MOCK_CONFIG と同期すること
                enhance_prompt: "Enhance this sky image to look photorealistic, like a high-quality photograph. Preserve the sun position and sky colors but add natural cloud textures, atmospheric haze, and photographic quality.".to_string(),
                enabled: false,
            },
        }
    }
}
