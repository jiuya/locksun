import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright 設定ファイル - Tauri WebView テスト用
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: "./tests/e2e",

  /* 最大テスト時間 */
  timeout: 30 * 1000,

  expect: {
    /**
     * Tauri アプリは起動が遅い場合があるため
     * アサーションのタイムアウトを長めに設定
     */
    timeout: 5000,
  },

  /* 失敗時の再試行回数 */
  retries: process.env.CI ? 2 : 0,

  /* 並列実行のワーカー数 */
  workers: process.env.CI ? 1 : undefined,

  /* レポーター設定 */
  reporter: "html",

  /* 全テストで共通の設定 */
  use: {
    /* アクション実行のタイムアウト */
    actionTimeout: 0,

    /* ベースURL（開発時は Vite サーバー） */
    baseURL: "http://localhost:1420",

    /* 失敗時のスクリーンショット */
    screenshot: "only-on-failure",

    /* テスト実行のビデオ記録（失敗時のみ） */
    video: "retain-on-failure",

    /* ブラウザコンテキストのトレース（失敗時のみ） */
    trace: "on-first-retry",
  },

  /* プロジェクト設定 */
  projects: [
    {
      name: "Desktop Chromium",
      use: {
        ...devices["Desktop Chrome"],
        // Tauri WebView は Chromium ベースのため
        channel: "chromium",
      },
    },
  ],

  /* dev サーバー設定（必要に応じて） */
  webServer: {
    command: "npm run tauri dev",
    port: 1420, // Tauri の開発サーバーデフォルトポート
    reuseExistingServer: !process.env.CI,
    timeout: 60 * 1000, // Tauri アプリの起動には時間がかかる
  },
});
