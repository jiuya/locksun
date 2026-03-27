// src/api/tauri_commands.ts
// Tauri バックエンドコマンドの型付きラッパー

import { invoke as tauriInvoke } from "@tauri-apps/api/core";

// Tauri webview が IPC ブリッジを注入したかを確認するためのグローバル宣言
declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
    __PLAYWRIGHT_TEST__?: boolean;
  }
}

// テスト環境用のモックデータ
const MOCK_CONFIG: AppConfig = {
  location: {
    latitude: 35.68,
    longitude: 139.69,
    name: "東京",
  },
  update: {
    interval_secs: 60, // 1分
  },
  image: {
    width: 1920,
    height: 1080,
    show_stars: true,
    show_clouds: true,
    water_depth: 0.7,
  },
  behavior: {
    autostart: false,
  },
};

// Tauri ウィンドウ外（通常ブラウザ）で開いた場合に分かりやすいエラーを出す、またはテスト時はモックを使用
function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  // テスト環境の場合はモックデータを返す
  if (window.__PLAYWRIGHT_TEST__) {
    console.log(`[MOCK ACTIVE] Command: ${cmd}`, args);
    return handleMockCommand<T>(cmd, args);
  }

  if (typeof window.__TAURI_INTERNALS__ === "undefined") {
    console.error(
      `[TAURI ERROR] __TAURI_INTERNALS__ not found for command: ${cmd}`,
    );
    return Promise.reject(
      new Error(
        `Tauri IPC が利用できません。ブラウザではなく Tauri アプリウィンドウで開いてください。(cmd: ${cmd})`,
      ),
    );
  }
  console.log(`[TAURI] Executing command: ${cmd}`, args);
  return tauriInvoke<T>(cmd, args);
}

// テスト用のモックコマンドハンドラー
function handleMockCommand<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  console.log(`[MOCK] Executing command: ${cmd}`, args);

  switch (cmd) {
    case "get_config":
      return Promise.resolve(MOCK_CONFIG as T);

    case "save_config":
      console.log("[MOCK] Config saved:", args);
      return Promise.resolve(undefined as T);

    case "preview_image":
    case "preview_image_with_config":
      // Base64エンコードされた小さなテスト画像（1x1の青いピクセル）
      const mockImage =
        "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
      return Promise.resolve(mockImage as T);

    case "calculate_sun_position":
      const mockSunPosition = {
        altitude: 45.0,
        azimuth: 180.0,
      };
      return Promise.resolve(mockSunPosition as T);

    default:
      console.warn(`[MOCK] Unhandled command: ${cmd}`);
      return Promise.resolve({} as T);
  }
}

// ---------------------------------------------------------------
// 型定義（src-tauri/src/commands/mod.rs と対応）
// ---------------------------------------------------------------

export interface LocationConfig {
  latitude: number;
  longitude: number;
  name: string;
}

export interface UpdateConfig {
  interval_secs: number;
}

export interface ImageConfig {
  width: number;
  height: number;
  show_stars: boolean;
  show_clouds: boolean;
  water_depth: number;
}

export interface BehaviorConfig {
  autostart: boolean;
}

export interface AppConfig {
  location: LocationConfig;
  update: UpdateConfig;
  image: ImageConfig;
  behavior: BehaviorConfig;
}

export interface SunPosition {
  altitude: number;
  azimuth: number;
}

export interface SunTimes {
  astronomical_dawn: string | null;
  civil_dawn: string | null;
  sunrise: string | null;
  solar_noon: string;
  sunset: string | null;
  civil_dusk: string | null;
  astronomical_dusk: string | null;
}

export interface SunInfoResponse {
  position: SunPosition;
  times: SunTimes;
  phase: string;
  location_name: string;
}

// ---------------------------------------------------------------
// コマンドラッパー
// ---------------------------------------------------------------

export const getConfig = (): Promise<AppConfig> => invoke("get_config");
export const saveConfig = (cfg: AppConfig): Promise<void> =>
  invoke("save_config", { cfg });
export const getSunInfo = (): Promise<SunInfoResponse> =>
  invoke("get_sun_info");
export const previewImage = (): Promise<string> => invoke("preview_image");
export const previewImageWithConfig = (cfg: AppConfig): Promise<string> =>
  invoke("preview_image_with_config", { cfg });
