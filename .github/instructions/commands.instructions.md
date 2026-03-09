---
applyTo: "src-tauri/src/commands/**"
---

# Tauri IPC コマンドモジュール (commands/)

## 役割

TypeScript フロントエンドが `invoke()` で呼び出す Rust 関数群を定義する。

## 命名規則

- コマンド名はスネークケース（例: `get_config`, `save_config`）
- 対応する TypeScript 関数は `src/api/tauri_commands.ts` に同名で定義する

## コマンド一覧と仕様

| コマンド        | 引数             | 戻り値                    | 用途                                   |
| --------------- | ---------------- | ------------------------- | -------------------------------------- |
| `get_config`    | なし             | `AppConfig`               | 現在の設定を取得                       |
| `save_config`   | `cfg: AppConfig` | `()`                      | 設定を保存                             |
| `get_sun_info`  | なし             | `SunInfoResponse`         | 現在の太陽情報を取得（プレビュー更新） |
| `preview_image` | なし             | `String` (base64 DataURL) | プレビュー用 PNG を base64 で返す      |

## エラーハンドリング

- コマンドの戻り値は `Result<T, String>` を使う
- Rust の `anyhow::Error` は `.map_err(|e| e.to_string())` で `String` に変換する
- TypeScript 側は `try/catch` でエラーを捕捉する

## SunInfoResponse 型の同期

```rust
// Rust 側 (commands/mod.rs)
pub struct SunInfoResponse {
    pub position:      SunPosition,
    pub times:         SunTimes,
    pub phase:         String,
    pub location_name: String,
}
```

```typescript
// TypeScript 側 (api/tauri_commands.ts) と必ず一致させる
export interface SunInfoResponse {
  position: SunPosition;
  times: SunTimes;
  phase: string;
  location_name: string;
}
```

## preview_image の実装ガイド

```rust
// PNG → Vec<u8> → base64 の変換パターン
let mut buf = std::io::Cursor::new(Vec::new());
img.write_to(&mut buf, image::ImageFormat::Png)?;
let encoded = base64::engine::general_purpose::STANDARD.encode(buf.into_inner());
Ok(format!("data:image/png;base64,{encoded}"))
```
