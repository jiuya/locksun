---
applyTo: "src-tauri/src/scheduler/**"
---

# 定期更新スケジューラー (scheduler/)

## 役割

`tokio` の非同期ループで定期的に 1回の更新サイクルを実行する。  
更新サイクル: 設定読み込み → 太陽位置計算 → 画像生成 → ロックスクリーン適用

## 実装ルール

- `start(app: AppHandle)` は `async fn` として定義し、`loop` で永続実行する
- 各サイクルで設定を **毎回 `config::load()` で再読み込み** する（動的反映のため）
- エラーは `log::error!` で記録するが、ループを止めない（`if let Err(e) = ... { log::error! }` パターン）
- 待機は `tokio::time::sleep(Duration::from_secs(interval))` のみ使用
- `run_once()` は同期関数 `fn run_once(app: &AppHandle) -> anyhow::Result<()>` として分離する（テスト容易性のため）

## 更新間隔の最小値

`interval_secs` が 30 未満の場合は 30 秒に切り上げる（過負荷防止）。

## 画像出力先パス

```rust
app.path().app_cache_dir() / "lockscreen.png"
```

パス取得に失敗した場合は `PathBuf::from(".")` にフォールバックし、
`log::warn!` でパス解決失敗を記録する。

## 即時更新トリガー

トレイメニューの「今すぐ更新」は `commands::trigger_update()` で  
共有フラグを立て、スケジューラーのループ内でフラグを検出して即時実行する。  
（将来実装: 現在は `sleep` をキャンセルする方式に変更予定）
