| name | playwright-webview-check |
| --- | --- |
| description | Automates Tauri WebView testing and verification. Use when the user needs to check UI state, take screenshots, test form interactions, verify preview functionality, or investigate WebView issues in Tauri applications. |
| allowed-tools | run_in_terminal(node tests/e2e/webview-check.js *), run_in_terminal(npm run test:e2e*), view_image, read_file, create_file |

# Tauri WebView 自動確認

## Quick start

```bash
# クイック健全性確認（軽量版）
node tests/e2e/webview-check.js quick

# 包括的UI状態確認（推奨）
node tests/e2e/webview-check.js comprehensive

# 機能テスト（設定とプレビュー）
node tests/e2e/webview-check.js functionality
```

## Commands

### Core

```bash
# 基本的なWebView確認
node tests/e2e/webview-check.js quick

# 包括的な確認（全項目）
node tests/e2e/webview-check.js comprehensive

# npm経由での実行
npm run test:e2e -- -g "クイック健全性確認"
npm run test:e2e -- -g "包括的な UI 状態確認"
```

### Screenshots

```bash
# 自動スクリーンショット取得（テスト実行時）
node tests/e2e/webview-check.js comprehensive

# 手動Playwrightテスト（UIモード）
npm run test:e2e:ui

# ヘッド付きブラウザで実行
npm run test:e2e:headed
```

### Debugging

```bash
# デバッグモードで実行
npx playwright test --headed --debug

# HTML レポート生成
npx playwright show-report
```

## 自動確認内容

### 基本状態確認
- アプリケーション起動状況の検証
- メイン画面表示状態の確認
- JavaScript エラーの監視と報告
- ページタイトルの正確性確認

### 設定画面要素
- 📍 緯度・経度入力フィールドの検出
- 📝 場所名入力フィールドの検出
- ⏰ 更新間隔設定の検出
- 🔘 プレビューボタンの検出

### フォーム動作確認
- 入力フィールドの機能テスト
- セレクトボックスの動作確認
- ボタンのクリック可能性検証

### プレビュー機能
- プレビュー生成の実行テスト
- Canvas/画像の描画確認
- 座標入力反映の確認

## 使用タイミング

**USE FOR:**
- ユーザーが「UIを確認して」「画面をチェックして」と要求した場合
- フロントエンド変更後の動作確認
- バグ報告や問題調査時の状態確認
- 設定画面や機能テストの自動実行

**DO NOT USE FOR:**
- バックエンド（Rust）のみのテスト
- ファイルシステム操作のテスト
- ネットワーク通信のテスト（WebView以外）
- パフォーマンステストのような複雑な計測

## 出力形式

実行後は以下の形式でレポートを提供します:

### 基本状態レポート
```
📊 === WebView 確認レポート ===
⏰ 確認時刻: 2026-03-27T09:00:59.651Z
🌟 総合的な健全性: healthy/warning/error

📱 === 基本状態 ===
✅ アプリ起動: 成功/❌失敗
✅ タイトル正確: 正常/❌異常
✅ メイン要素表示: 表示中/❌非表示
🐞 JavaScript エラー: X件
📷 スクリーンショット: 取得済み/❌失敗
```

### スクリーンショットファイル
- `tests/e2e/screenshots/basic-state.png` - 全画面キャプチャ
- `tests/e2e/screenshots/settings-panel.png` - 設定画面
- `tests/e2e/screenshots/preview-canvas.png` - プレビュー結果

### JSONレポート
包括的確認時は詳細なJSONレポートも生成されます。

## Tauri固有の考慮事項

### IPC通信の問題
- Tauri APIが利用できない環境では自動的にモック機能を使用
- `__PLAYWRIGHT_TEST__` フラグによる環境判定
- エラー時は適切なフォールバック動作

### 起動時間
- Tauri アプリは通常のWebアプリより起動時間が長い
- タイムアウト設定: 30秒（基本）、15秒（プレビュー待機）

### セキュリティ
- CSP制約下での動作対応
- WebView限定API の適切な処理

## プロジェクト固有設定（Locksun）

### 重点確認要素
- 太陽位置計算の設定画面要素
- 空画像プレビューの生成機能  
- ロックスクリーン画像設定の動作
- Windows自動起動設定の状態

### 期待される座標値
- 東京: 緯度 35.68°N、経度 139.69°E（テストデフォルト）
- 札幌: 緯度 43.06°N、経度 141.35°E（高緯度テスト）
- 那覇: 緯度 26.21°N、経度 127.68°E（低緯度テスト）

