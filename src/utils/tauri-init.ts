// src/utils/tauri-init.ts
// Tauri API の初期化を待機するユーティリティ

/**
 * Tauri API が利用可能になるまで待機する
 * @param timeoutMs タイムアウト時間（ミリ秒）
 * @param intervalMs チェック間隔（ミリ秒）
 * @returns Promise<boolean> - 成功時 true、タイムアウト時 false
 */
export async function waitForTauriReady(
  timeoutMs = 10000,
  intervalMs = 100,
): Promise<boolean> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    // Tauri API が利用可能かチェック
    if (typeof (window as any).__TAURI_INTERNALS__ !== "undefined") {
      console.log("✅ Tauri API が利用可能になりました");
      return true;
    }

    // 指定された間隔で待機
    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }

  console.warn(`⚠️ Tauri API の初期化がタイムアウトしました (${timeoutMs}ms)`);
  return false;
}

/**
 * Tauri API の初期化状態をログに出力する
 */
export function logTauriInitState(): void {
  console.log("=== Tauri 初期化状態 ===");
  console.log(
    "__TAURI_INTERNALS__:",
    typeof (window as any).__TAURI_INTERNALS__,
  );
  console.log("window.__TAURI__:", typeof (window as any).__TAURI__);

  if (typeof (window as any).__TAURI_INTERNALS__ !== "undefined") {
    console.log("✅ Tauri API は利用可能です");
  } else {
    console.log("❌ Tauri API は利用できません");

    // デバッグ情報: Tauriに関するプロパティを検索
    const tauriKeys = Object.keys(window).filter((key) =>
      key.toLowerCase().includes("tauri"),
    );
    console.log("Tauri関連のwindowプロパティ:", tauriKeys);
  }
}
