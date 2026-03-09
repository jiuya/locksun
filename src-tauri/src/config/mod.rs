// src-tauri/src/config/mod.rs

pub mod types;

use anyhow::Result;
use std::path::PathBuf;
pub use types::AppConfig;

/// ユーザー設定ファイルのパスを返す
/// - リリース時: %APPDATA%\locksun\config.toml
/// - デバッグ時: workspace/config/user_settings.toml
pub fn config_path() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        PathBuf::from("config/user_settings.toml")
    }
    #[cfg(not(debug_assertions))]
    {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("locksun")
            .join("config.toml")
    }
}

/// 設定を読み込む。ファイルが存在しなければデフォルト値を返す
pub fn load() -> Result<AppConfig> {
    let path = config_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let cfg: AppConfig = toml::from_str(&content)?;
        Ok(cfg)
    } else {
        Ok(AppConfig::default())
    }
}

/// 設定をファイルに保存する
pub fn save(cfg: &AppConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(cfg)?;
    std::fs::write(&path, content)?;
    Ok(())
}
