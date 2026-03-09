---
applyTo: "src/**"
---

# TypeScript フロントエンド (src/)

## 役割

Tauri の WebView で表示される設定UI画面。  
バックエンドとの通信は `src/api/tauri_commands.ts` のみを経由する。

## ファイル構成

| ファイル                | 責務                                                         |
| ----------------------- | ------------------------------------------------------------ |
| `main.ts`               | エントリーポイント。ページコンポーネントを `#app` にマウント |
| `api/tauri_commands.ts` | すべての `invoke()` 呼び出しをラップした型付き関数           |
| `pages/settings.ts`     | 設定画面のレンダリングとイベントハンドリング                 |
| `styles/main.css`       | ダークテーマの CSS 変数ベーススタイル                        |

## api/tauri_commands.ts のルール

- `invoke()` の直接呼び出しはこのファイル以外で行わない
- すべての引数・戻り値に TypeScript の型を付ける
- Rust の `snake_case` フィールドはそのまま使う（`location_name` など）
- **Rust 側の構造体が変わったら必ずここも更新する**

## pages/settings.ts のルール

- `innerHTML` でテンプレートを構築し、イベントは `addEventListener` で後付けする
- フォーム入力値の取得: `document.getElementById("id")! as HTMLInputElement` パターン
- 非同期関数はすべて `async function` + `await` で書く（`.then()` チェーン禁止）
- エラーは `#status-msg` 要素に表示し、3秒後に自動クリアする

## styles/main.css のルール

- カラー変数は `:root` に `--bg`, `--surface`, `--accent` 等で定義する
- クラス名はケバブケース（`sky-preview`, `form-section` など）
- メディアクエリは不要（ウィンドウサイズ固定: 480×600）

## プレビュー更新のフロー

```
ボタンクリック
  → previewImage() (Tauri invoke)
    → Rust: 太陽位置計算 → 画像生成 → base64 エンコード
  → <img src="data:image/png;base64,..."> を差し替え
```

プレビュー中は `preview-loading` テキストを表示し、完了後に差し替える。
