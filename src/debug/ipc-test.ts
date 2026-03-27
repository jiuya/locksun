// src/debug/ipc-test.ts
// IPC接続状態をデバッグするためのテストユーティリティ

import { invoke } from "@tauri-apps/api/core";

export interface IPCTestResult {
  tauriAvailable: boolean;
  internalsDetected: boolean;
  commandTestResults: {
    [command: string]: {
      success: boolean;
      result?: any;
      error?: string;
      duration: number;
    };
  };
}

/**
 * Tauri IPC の接続状態とコマンドの動作を包括的にテストする
 */
export async function testIPCConnection(): Promise<IPCTestResult> {
  const startTime = performance.now();

  const result: IPCTestResult = {
    tauriAvailable:
      typeof window !== "undefined" &&
      typeof (window as any).__TAURI_INTERNALS__ !== "undefined",
    internalsDetected:
      typeof (window as any).__TAURI_INTERNALS__ !== "undefined",
    commandTestResults: {},
  };

  // 各IPCコマンドを順番にテスト
  const commandsToTest = ["get_config", "get_sun_info", "preview_image"];

  for (const command of commandsToTest) {
    const commandStartTime = performance.now();
    try {
      const commandResult = await invoke(command, {});
      const duration = performance.now() - commandStartTime;

      result.commandTestResults[command] = {
        success: true,
        result: commandResult,
        duration,
      };
    } catch (error) {
      const duration = performance.now() - commandStartTime;
      result.commandTestResults[command] = {
        success: false,
        error: error?.toString() || "Unknown error",
        duration,
      };
    }
  }

  console.log("=== IPC Connection Test Results ===");
  console.log("Tauri Available:", result.tauriAvailable);
  console.log("Internals Detected:", result.internalsDetected);
  console.log("Command Test Results:", result.commandTestResults);

  return result;
}

/**
 * グローバルスコープでテスト関数を利用可能にする（デバッグ用）
 */
if (typeof window !== "undefined") {
  (window as any).testIPC = testIPCConnection;
  (window as any).tauriInternals = (window as any).__TAURI_INTERNALS__;
}
