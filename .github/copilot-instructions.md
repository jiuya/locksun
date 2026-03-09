# Locksun - GitHub Copilot リポジトリ共通指示

## プロジェクト概要

Windowsのロックスクリーン画像を太陽の動きでリアルタイムに変化させる常駐アプリ。  
バックエンド: **Rust + Tauri v2**、フロントエンド: **TypeScript + Vite**

---

## アーキテクチャ

```
locksun/
├── src-tauri/src/
│   ├── sun/          太陽位置計算（NOAA簡易式）
│   ├── renderer/     空の画像生成（image クレート）
│   ├── lockscreen/   Windows レジストリ操作
│   ├── scheduler/    定期更新ループ（tokio）
│   ├── config/       TOML 設定管理
│   └── commands/     Tauri IPC コマンド
├── src/              TypeScript フロントエンド
│   ├── api/          Tauri コマンドラッパー（型付き）
│   ├── pages/        設定UI画面
│   └── styles/       CSS（ダークテーマ）
└── tests/            Rust 統合テスト
```

---

## Rust コーディング規約

- エラー型は `anyhow::Result<T>` を使う（ライブラリ公開 API は `thiserror`）
- `unwrap()` / `expect()` は**テストコード以外で禁止**。必ず `?` または `map_err` で伝播させる
- `log::info!` / `log::error!` でログを出力する（`println!` は禁止）
- 非同期処理は `tokio` に統一する
- Windows 固有のコードは `#[cfg(target_os = "windows")]` で囲む
- 魔法数字はすべて定数として定義する
- 各モジュールの先頭に `// src-tauri/src/<module>/mod.rs` 形式のコメントを付ける

## TypeScript コーディング規約

- `any` 型の使用禁止。すべての型を明示する
- Tauri コマンドは必ず `src/api/tauri_commands.ts` のラッパーを経由する
- `document.getElementById` の結果は `!` アサーションを使ってよい（存在確認済みの要素のみ）
- CSS クラス名はケバブケース（例: `sky-preview`）
- `async/await` を使い、`Promise` チェーンは避ける

---

## 重要な設計上の制約

1. **ロックスクリーン変更には管理者権限が必要**  
   `lockscreen::set_lockscreen_image()` は管理者権限なしでは失敗する。  
   権限エラーはユーザーにトレイ通知で伝える（クラッシュさせない）。

2. **常駐時の軽量動作を維持する**  
   待機中のCPU使用率 1% 以下、メモリ 50MB 以下を目標にする。  
   重い処理（画像生成）は更新間隔ごとにのみ実行する。

3. **太陽計算の精度は「日の出・日の入り ±数分」で十分**  
   天文学的な高精度演算は不要。NOAA 簡易式（Spencer 1971）を使う。
   白夜・極夜は `Option<DateTime>` を返すことで対応済み。

4. **Tauri IPC の型安全性**  
   `src/api/tauri_commands.ts` の型定義は `src-tauri/src/commands/mod.rs` の  
   Rust 構造体と常に同期させる。片方を変更したら必ず両方を更新する。

---

## テスト方針

- 太陽計算のテストは実際の日時・都市で検証する（東京・2026-03-09 等）
- `#[cfg(test)]` ブロックを各モジュールに配置する
- Windows API 依存のコードはモックせず `#[cfg(target_os = "windows")]` でスキップ可
- テストの許容誤差: 日の出・日の入り ±10分

---

## 依存関係の方針

- 新しいクレートを追加する前に既存の依存で解決できないか確認する
- GUI フレームワークは Tauri に統一する（別途 egui / slint 等を追加しない）
- 画像処理は `image` クレートのみ使用する
