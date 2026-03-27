// src/main.ts
// フロントエンドのエントリーポイント

import { renderSettings } from "./pages/settings.js";
import { waitForTauriReady, logTauriInitState } from "./utils/tauri-init.js";
// デバッグ用: IPC接続テスト機能をインポート
import "./debug/ipc-test.js";

const app = document.getElementById("app")!;

async function initializeApp() {
  console.log("=== Locksun フロントエンド初期化開始 ===");

  // 初期化前の状態をログ出力
  logTauriInitState();

  // Tauri API の初期化を待機（短時間で切り上げ）
  console.log("⏳ Tauri API の初期化を待機中...");
  const tauriReady = await waitForTauriReady(3000, 50); // 3秒間、50msごとにチェック

  if (tauriReady) {
    console.log("✅ Tauri API が正常に初期化されました");
    logTauriInitState();
  } else {
    console.warn(
      "⚠️ Tauri API の初期化がタイムアウトしました - ブラウザモードで実行します",
    );
  }

  // 設定画面をレンダリング（待機に関係なく実行）
  console.log("🎨 設定画面をレンダリング中...");
  try {
    await renderSettings(app);
    console.log("✅ 設定画面のレンダリングが完了しました");
  } catch (e) {
    console.error("❌ renderSettings の起動に失敗しました:", e);

    // エラー時はシンプルなエラー画面を表示
    app.innerHTML = `
      <div style="padding: 20px; text-align: center; color: #ff6b6b;">
        <h2>⚠️ 初期化エラー</h2>
        <p>アプリケーションの初期化に失敗しました。</p>
        <p>詳細: ${e}</p>
        <p>ブラウザの開発者ツールで詳細を確認してください。</p>
      </div>
    `;
  }

  console.log("=== Locksun フロントエンド初期化完了 ===");
}

// アプリケーション初期化を実行
initializeApp();
