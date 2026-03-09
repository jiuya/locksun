---
mode: agent
description: "新しいレンダリング機能（星・雲など）を renderer/ に追加する"
---

## タスク: レンダリング機能の追加

以下の手順で `src-tauri/src/renderer/` に新しいレイヤーを追加してください。

### 1. palette.rs の更新

新機能に必要なカラー情報を `SkyColors` 構造体に追加する場合、  
既存のキーフレームをすべて更新すること。

### 2. 新しいレイヤーファイルの作成

`src-tauri/src/renderer/<layer_name>.rs` を作成:

- `render_<layer_name>(pos: &SunPosition, cfg: &ImageConfig, base: &mut RgbImage)` を実装する
- `cfg.show_<layer_name>` が `false` の場合は即 `return` する

### 3. composer.rs への組み込み

`compose()` 関数のレイヤー順序:

1. `render_sky()` — 背景グラデーション
2. `render_sun()` — 太陽ディスク
3. `render_stars()` — 星（夜間）← ここに追加
4. `render_clouds()` — 雲 ← またはここ

### 4. config/types.rs の更新

`ImageConfig` に `show_<layer_name>: bool` フィールドを追加し、  
`src/api/tauri_commands.ts` の `ImageConfig` インターフェースも同期すること。

### 5. テストの追加

`renderer/` モジュール内の `#[cfg(test)]` ブロックに:

- 機能が有効な場合のスモークテスト
- 機能が無効な場合に base 画像が変化しないことの確認
