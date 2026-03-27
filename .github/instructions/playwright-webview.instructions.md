---
applyTo: "tests/e2e/**"
---

# Playwright Web View テスト自動確認スキル

## 概要

Playwright CLI を使用してエージェントが Tauri WebView を自動確認するためのスキル。  
UI の動作確認、スクリーンショット取得、要素検証を自動化する。

## セットアップ

### 1. 依存関係のインストール

```bash
npm install @playwright/test --save-dev
npx playwright install chromium
```

### 2. テスト実行コマンド

```bash
# 通常実行
npm run test:e2e

# UI モードで実行（ブラウザ画面表示）
npm run test:e2e:ui

# ヘッドレスモードで実行
npm run test:e2e:headed
```

## スキルの用途

### ✅ UI 動作確認

- 設定画面の表示確認
- フォーム入力の動作確認
- ボタンクリックの応答確認
- プレビュー画像の生成確認

### ✅ スクリーンショット取得

- 全画面キャプチャ
- 特定要素のキャプチャ
- 異なる設定での表示比較
- デバッグ用の画面状態記録

### ✅ 要素検証

- DOM 要素の存在確認
- CSS スタイルの適用確認
- データの表示確認
- レスポンシブデザインの確認

## テストファイル構造

```
tests/e2e/
├── app.spec.ts              # メイン UI テスト
├── settings.spec.ts         # 設定画面専用テスト
├── preview.spec.ts          # プレビュー機能テスト
└── screenshots/             # スクリーンショット保存ディレクトリ
    ├── full-app.png
    ├── settings-area.png
    └── preview-canvas.png
```

## エージェント使用方法

### 1. 手動確認スクリプト作成

エージェントが以下のようなテストスクリプトを動的に生成・実行:

```typescript
// 自動生成されるテストスクリプト例
test("エージェント: UI状態確認", async ({ page }) => {
  await page.goto("/");

  // 現在の画面をキャプチャ
  await page.screenshot({ path: "current-state.png" });

  // 特定の要素や値を確認
  const locationInput = page.locator('input[name="location_name"]');
  if ((await locationInput.count()) > 0) {
    console.log("位置情報入力欄:", await locationInput.inputValue());
  }

  // 設定値の確認
  const settings = await page.evaluate(() => {
    return Object.fromEntries(
      Array.from(document.querySelectorAll("input")).map((input: any) => [
        input.name || input.id,
        input.value,
      ]),
    );
  });
  console.log("現在の設定:", settings);
});
```

### 2. 要素状態のプログラム的アクセス

```typescript
// DOM 情報取得
const domStructure = await page.evaluate(() => {
  return document.querySelector("#app")?.innerHTML;
});

// スタイル情報取得
const styling = await page.evaluate(() => {
  const app = document.querySelector("#app");
  return app ? getComputedStyle(app) : null;
});

// フォームデータ取得
const formData = await page.evaluate(() => {
  const forms = document.querySelectorAll("form");
  return Array.from(forms).map((form) => new FormData(form));
});
```

### 3. インタラクティブ動作確認

```typescript
// 設定変更のテスト
await page.fill('input[name="latitude"]', "43.06");
await page.fill('input[name="longitude"]', "141.35");
await page.click('button:has-text("プレビュー")');

// 結果確認
await page.waitForSelector("canvas, .preview-image");
await page.screenshot({ path: "after-settings-change.png" });
```

## Tauri WebView 特有の考慮事項

### 1. 起動時間

- Tauri アプリは通常のWebアプリより起動が遅い
- `timeout: 30000` で十分な待機時間を設ける

### 2. IPC 通信の確認

```typescript
// Tauri コマンドの実行確認
const result = await page.evaluate(async () => {
  // @ts-ignore
  return window.__TAURI__?.invoke?.("get_current_settings");
});
```

### 3. ネイティブ機能のモック

```typescript
// ファイルシステムアクセス等はモックが必要
await page.addInitScript(() => {
  // @ts-ignore
  window.__TAURI__ = {
    invoke: (cmd: string, args?: any) => {
      // モック実装
      return Promise.resolve({});
    },
  };
});
```

## 自動確認パイプライン仕様

### 1. Basic Health Check

```bash
# 基本動作確認
npx playwright test app.spec.ts --reporter=json > health-check.json
```

### 2. Visual Regression

```bash
# スクリーンショット比較
npx playwright test --update-snapshots  # ベースライン作成
npx playwright test                      # 差分確認
```

### 3. Performance Monitoring

```typescript
test("パフォーマンス監視", async ({ page }) => {
  const startTime = Date.now();
  await page.goto("/");
  await page.waitForSelector("#app");
  const loadTime = Date.now() - startTime;

  expect(loadTime).toBeLessThan(5000); // 5秒以内の起動
});
```

## GitHub Actions 連携

`.github/workflows/e2e-tests.yml` に以下を設定:

```yaml
name: E2E テスト
on: [push, pull_request]

jobs:
  test:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "18"
      - run: npm ci
      - run: npx playwright install chromium
      - run: npm run test:e2e
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: playwright-report
          path: playwright-report/
```

## トラブルシューティング

### 1. Tauri アプリが起動しない

```bash
# 手動起動確認
npm run tauri dev

# ログ確認
npx playwright test --headed --debug
```

### 2. 要素が見つからない

```typescript
// より柔軟なセレクタを使用
await page.waitForSelector("#app, .main, body", { timeout: 10000 });
```

### 3. スクリーンショットが空白

```bash
# ブラウザを表示して確認
npx playwright test --headed
```

## エージェント向け推奨フロー

1. **開始前確認**: `npm run tauri dev` でアプリが起動することを確認
2. **基本テスト実行**: `npm run test:e2e` で基本動作をテスト
3. **スクリーンショット取得**: UI の状態をビジュアルで確認
4. **詳細確認**: 特定の要素や機能に対してカスタムテストを実行
5. **レポート生成**: テスト結果とスクリーンショットをまとめてレポート

このスキルにより、エージェントは人間の確認無しにUI状態を把握し、問題点を特定できます。
