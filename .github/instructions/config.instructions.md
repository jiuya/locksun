---
applyTo: "src-tauri/src/config/**"
---

# 設定管理モジュール (config/)

## 役割

TOML 形式の設定ファイルの読み書きと、`AppConfig` 型の定義。

## ファイル構成

| ファイル   | 責務                                             |
| ---------- | ------------------------------------------------ |
| `types.rs` | `AppConfig` とネスト構造体の定義、`Default` 実装 |
| `mod.rs`   | `load()` / `save()` / `config_path()` の実装     |

## types.rs のルール

- すべての構造体に `#[derive(Debug, Clone, Serialize, Deserialize)]` を付ける
- フィールドのデフォルト値は `impl Default for AppConfig` にまとめる
- コメントで単位・範囲を明記する（例: `/// 更新間隔 (秒)`）
- **フィールドを追加・変更したら `src/api/tauri_commands.ts` も必ず同期すること**

## Default 値

```
location: 東京 (35.6895°N, 139.6917°E)
update.interval_secs: 300 (5分)
image: 1920×1080, show_stars=true, show_clouds=false
behavior.autostart: false
```

## config_path() のルール

- `debug_assertions` が有効（開発時）: `config/user_settings.toml`（リポジトリ直下）
- リリース時: `%APPDATA%\locksun\config.toml`
- `dirs::config_dir()` が `None` の場合は `PathBuf::from(".")` にフォールバック

## load() / save() のルール

- `load()`: ファイルが存在しない場合は `Ok(AppConfig::default())` を返す（エラーにしない）
- `save()`: 親ディレクトリが存在しない場合は `create_dir_all()` で作成する
- TOML パースエラーは `?` で伝播させる（破損設定ファイルはそのまま失敗させる）
