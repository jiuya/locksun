// src/api/tauri_commands.ts
// Tauri バックエンドコマンドの型付きラッパー

import { invoke as tauriInvoke } from "@tauri-apps/api/core";

// Tauri webview が IPC ブリッジを注入したかを確認するためのグローバル宣言
declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

// Tauri ウィンドウ外（通常ブラウザ）で開いた場合に分かりやすいエラーを出す
function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (typeof window.__TAURI_INTERNALS__ === "undefined") {
    return Promise.reject(
      new Error(
        `Tauri IPC が利用できません。ブラウザではなく Tauri アプリウィンドウで開いてください。(cmd: ${cmd})`
      )
    );
  }
  return tauriInvoke<T>(cmd, args);
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

export const getConfig  = (): Promise<AppConfig>        => invoke("get_config");
export const saveConfig = (cfg: AppConfig): Promise<void> => invoke("save_config", { cfg });
export const getSunInfo = (): Promise<SunInfoResponse>  => invoke("get_sun_info");
export const previewImage = (): Promise<string>         => invoke("preview_image");
