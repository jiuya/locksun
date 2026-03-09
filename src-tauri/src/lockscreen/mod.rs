// src-tauri/src/lockscreen/mod.rs
// Windows ロックスクリーン画像の変更
//
// 方式: HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\PersonalizationCSP
// 必要権限: SYSTEM または Administrator
// 参考: https://learn.microsoft.com/en-us/windows/client-management/mdm/personalization-csp

use anyhow::{Context, Result};
use std::path::Path;

#[cfg(target_os = "windows")]
use winreg::{enums::*, RegKey};

/// ロックスクリーン画像を指定パスの画像に変更する
pub fn set_lockscreen_image(image_path: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        set_lockscreen_windows(image_path)
    }
    #[cfg(not(target_os = "windows"))]
    {
        log::warn!("ロックスクリーン変更は Windows のみ対応");
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn set_lockscreen_windows(image_path: &Path) -> Result<()> {
    let path_str = image_path
        .to_str()
        .context("画像パスの文字列変換に失敗")?;

    const SUBKEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\PersonalizationCSP";

    // HKLM への書き込みは管理者権限が必要
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (key, disposition) = hklm
        .create_subkey(SUBKEY)
        .context("レジストリキーの作成/オープンに失敗 (管理者権限が必要な場合があります)")?;

    if disposition == REG_CREATED_NEW_KEY {
        log::info!("PersonalizationCSP キーを新規作成しました");
    }

    key.set_value("LockScreenImageStatus", &1u32)
        .context("LockScreenImageStatus の設定に失敗")?;
    key.set_value("LockScreenImagePath", &path_str)
        .context("LockScreenImagePath の設定に失敗")?;
    key.set_value("LockScreenImageUrl", &path_str)
        .context("LockScreenImageUrl の設定に失敗")?;

    log::info!("ロックスクリーン画像を設定: {path_str}");
    Ok(())
}

/// 現在の権限でレジストリへ書き込み可能か確認する
pub fn check_permission() -> bool {
    #[cfg(target_os = "windows")]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        hklm.create_subkey(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\PersonalizationCSP",
        )
        .is_ok()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}
