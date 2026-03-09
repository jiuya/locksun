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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// Windows スタートアップに登録するか
    pub autostart: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            location: LocationConfig {
                // デフォルト: 東京
                latitude:  35.6895,
                longitude: 139.6917,
                name: "東京".to_string(),
            },
            update: UpdateConfig {
                interval_secs: 300, // 5分
            },
            image: ImageConfig {
                width:       1920,
                height:      1080,
                show_stars:  true,
                show_clouds: false,
            },
            behavior: BehaviorConfig {
                autostart: false,
            },
        }
    }
}
