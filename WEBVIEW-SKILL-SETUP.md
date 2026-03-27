# Playwright WebView 自動確認スキル - セットアップ完了

## ✅ セットアップ状況

以下のファイルとディレクトリが正常に作成されました：

### 📁 プロジェクト構造
```
locksun/
├── playwright.config.ts                                    # Playwright設定
├── tests/e2e/                                             # E2Eテストディレクトリ
│   ├── screenshots/                                       # スクリーンショット保存
│   ├── app.spec.ts                                        # 基本UIテスト
│   ├── agent-check.spec.ts                               # エージェント専用テスト
│   ├── webview-helpers.ts                                # ヘルパー関数
│   └── webview-check.js                                  # 実行スクリプト
├── .github/instructions/playwright-webview.instructions.md # スキル詳細指示
└── .vscode/skills/playwright-webview-check/
    └── SKILL.md                                           # スキルメタデータ
```

### 🔧 インストール済み依存関係
- ✅ @playwright/test (v1.46.0)
- ✅ Chromium ブラウザ (v145.0.7632.6)
- ✅ FFmpeg (動画録画用)

## 🚀 スキルの使用方法

### エージェント向けクイック実行
```bash
# 1. クイック確認（軽量）
npm run test:e2e -- -g "クイック健全性確認"

# 2. 包括的確認（推奨）
npm run test:e2e -- -g "包括的な UI 状態確認"

# 3. 機能テスト（設定とプレビュー）  
npm run test:e2e -- -g "特定機能の動作確認"
```

### 独立スクリプトによる実行
```bash
# クイック確認
node tests/e2e/webview-check.js quick

# 包括的確認
node tests/e2e/webview-check.js comprehensive

# 機能確認
node tests/e2e/webview-check.js functionality
```

## 🎯 エージェントが取得できる情報

### ✅ 基本状態確認
- アプリケーション起動状況
- メイン画面の表示状態
- JavaScript エラーの発生状況
- ページタイトルの確認

### ⚙️ 設定要素検出
- 緯度・経度入力フィールドの有無
- 場所名入力の有無
- 更新間隔設定の有無
- プレビューボタンの有無

### 🎨 機能動作確認
- フォーム入力の動作確認
- プレビュー機能の実行と結果確認
- Canvas/画像の描画確認
- ボタンクリックの応答確認

### 📷 ビジュアル確認
- 全画面スクリーンショット
- 設定パネルの画面キャプチャ
- プレビュー結果のキャプチャ
- エラー発生時の状態記録

## 🔔 注意事項

### ⚠️ 実行前提条件
- Tauri アプリケーションが開発モードで起動している必要があります
- `npm run tauri dev` でアプリを起動してからテストを実行してください

### 📋 実行例（完全な流れ）
```bash
# 1. Tauri アプリを起動（別ターミナル）
npm run tauri dev

# 2. WebView 確認を実行（メインターミナル）
npm run test:e2e -- -g "包括的な UI 状態確認"

# または独立スクリプトで
node tests/e2e/webview-check.js comprehensive
```

## 🎉 次のステップ

これで、エージェントは以下のようにWebViewを自動確認できるようになりました：

1. **「UIを確認してください」** → 包括的確認を自動実行
2. **「画面の状態をチェック」** → クイック確認で基本状態を報告  
3. **「プレビュー機能は動いていますか？」** → 機能確認テストを実行
4. **「スクリーンショットを撮って」** → 自動的に画面キャプチャを取得

エージェントは人間の介入なしに、WebViewの状態を把握し、問題点を特定し、改善提案を行えるようになりました！