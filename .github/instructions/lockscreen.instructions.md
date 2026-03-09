---
applyTo: "src-tauri/src/lockscreen/**"
---

# ロックスクリーン操作モジュール (lockscreen/)

## 役割

生成した PNG 画像を Windows のロックスクリーンに適用する。  
Windows レジストリの `PersonalizationCSP` キーへ書き込む。

## レジストリ操作の仕様

```
HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\PersonalizationCSP
  LockScreenImageStatus  REG_DWORD  1
  LockScreenImagePath    REG_SZ     <画像ファイルのフルパス>
  LockScreenImageUrl     REG_SZ     <同じパス>
```

> この書き込みには **管理者権限** が必要。  
> 権限不足の場合は `anyhow::Error` を返し、呼び出し側でトレイ通知に変換する。

## 実装ルール

- Windows 専用コードは `#[cfg(target_os = "windows")]` で囲む
- 非 Windows では `log::warn!` を出力して `Ok(())` を返す（クロスコンパイルのため）
- `winreg::RegKey::predef(HKEY_LOCAL_MACHINE)` からキーを開く
- `create_subkey()` で "作成/オープン" を1回の呼び出しで行う
- 各 `set_value()` には `.context("...")` でわかりやすいエラー文を付ける

## 権限確認

`check_permission() -> bool` を提供する。  
スケジューラーの起動時に呼んで、権限がない場合は早期にユーザーへ通知する。

## テスト

- `#[cfg(target_os = "windows")]` のテストは CI でスキップ可
- `check_permission()` のユニットテストは記述不要（副作用が大きいため）
- 実機での動作確認は管理者権限の PowerShell で `cargo run` して確認する
