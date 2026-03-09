---
applyTo: "src-tauri/src/renderer/**"
---

# 画像生成モジュール (renderer/)

## 役割

`SunPosition` を受け取り、ロックスクリーン用の PNG 画像を生成する。  
依存するのは `sun::SunPosition` と `config::ImageConfig` の型定義のみ。

## ファイル構成と責務

| ファイル      | 責務                                                               |
| ------------- | ------------------------------------------------------------------ |
| `palette.rs`  | 高度角 → `SkyColors`（天頂色・地平線色・太陽色）のキーフレーム補間 |
| `sky.rs`      | グラデーション背景描画 / 太陽ディスク+ハロー描画                   |
| `composer.rs` | レイヤーを順番に合成して最終 `RgbImage` を返す                     |
| `mod.rs`      | `render_and_save()` の公開エントリーポイント                       |

## palette.rs のルール

- カラーパレットは `keyframes: &[(f64, SkyColors)]` の形式で定義する
- 補間は線形（`Color::lerp`）。高度角の範囲外は端点値を返す
- 新しいフェーズ（雲・霧など）を追加する際もキーフレームを追加するだけでよい

## sky.rs のルール

- `render_sky()`: 縦方向グラデーション。`y` が大きいほど地平線色に近づく
- `render_sun()`: 高度角 < -5.0° のときは描画しない（地平線以下）
- 太陽の画面座標は `azimuth / 360 * width`、`(1 - alt/95) * height` で計算
- ハローはガウス的減衰で alpha ブレンドする

## composer.rs のルール

- レイヤーの順序: 空背景 → 太陽 → 星（TODO） → 雲（TODO）
- TODO の機能は `// TODO` コメントで残し、`cfg.show_stars`/`cfg.show_clouds` フラグで後から有効化する
- エラーは `anyhow::Result` で返す

## パフォーマンス制約

- 1920×1080 の画像生成は **200ms 以内** に完了させる
- ピクセルループは `img.enumerate_pixels_mut()` のみ使用（unsafe 不使用）
- 不要なヒープアロケーションを避ける（`Vec` の事前 `with_capacity` など）
