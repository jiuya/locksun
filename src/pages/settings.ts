// src/pages/settings.ts
// 設定ページのレンダリングと操作

import {
  type AppConfig,
  getConfig,
  getSunInfo,
  previewImage,
  previewImageEnhanced,
  previewImageWithConfig,
  saveConfig,
} from "../api/tauri_commands.js";

// 自動プレビューデバウンスタイマー (ms)
const PREVIEW_DEBOUNCE_MS = 600;
let previewDebounceTimer: ReturnType<typeof setTimeout> | null = null;
// 読み込み済みの Gemini API キー（UI では非表示にするため別途保持）
let loadedGeminiApiKey = "";

/** フォームの現在値から AppConfig を構築する */
function buildConfigFromForm(): AppConfig {
  return {
    location: {
      name: (document.getElementById("location-name") as HTMLInputElement).value,
      latitude: parseFloat(
        (document.getElementById("latitude") as HTMLInputElement).value,
      ),
      longitude: parseFloat(
        (document.getElementById("longitude") as HTMLInputElement).value,
      ),
    },
    update: {
      interval_secs: parseInt(
        (document.getElementById("interval") as HTMLSelectElement).value,
      ),
    },
    image: {
      width: 1920,
      height: 1080,
      show_stars: (document.getElementById("show-stars") as HTMLInputElement).checked,
      show_clouds: (document.getElementById("show-clouds") as HTMLInputElement).checked,
      water_depth: parseFloat(
        (document.getElementById("water-depth") as HTMLInputElement).value,
      ),
    },
    behavior: {
      autostart: (document.getElementById("autostart") as HTMLInputElement).checked,
    },
    gemini: {
      // API キー入力欄が空の場合は既存のキーを保持する
      api_key: (document.getElementById("gemini-api-key") as HTMLInputElement).value || loadedGeminiApiKey,
      model_name: (document.getElementById("gemini-model") as HTMLInputElement).value,
      enhance_prompt: (document.getElementById("gemini-prompt") as HTMLTextAreaElement).value,
      enabled: (document.getElementById("gemini-enabled") as HTMLInputElement).checked,
    },
  };
}

/** デバウンス付きプレビュー自動更新 */
function schedulePreviewRefresh(): void {
  if (previewDebounceTimer !== null) {
    clearTimeout(previewDebounceTimer);
  }
  previewDebounceTimer = setTimeout(async () => {
    previewDebounceTimer = null;
    const cfg = buildConfigFromForm();
    const el = document.getElementById("sky-preview");
    if (!el) return;
    el.innerHTML = `<div class="preview-loading">生成中...</div>`;
    try {
      const dataUrl = await previewImageWithConfig(cfg);
      el.innerHTML = `<img src="${dataUrl}" alt="空のプレビュー" />`;
    } catch (e) {
      console.error("プレビュー自動更新エラー:", e);
    }
  }, PREVIEW_DEBOUNCE_MS);
}

/** プレビュー関連のフォームイベント（水深・星・雲）を登録する */
function bindPreviewSettingsEvents(): void {
  const waterDepthSlider = document.getElementById(
    "water-depth",
  ) as HTMLInputElement;
  const waterDepthValue = document.getElementById(
    "water-depth-value",
  ) as HTMLSpanElement;

  waterDepthSlider.addEventListener("input", () => {
    waterDepthValue.textContent = waterDepthSlider.value;
    schedulePreviewRefresh();
  });

  (document.getElementById("show-stars") as HTMLInputElement).addEventListener(
    "change",
    schedulePreviewRefresh,
  );
  (document.getElementById("show-clouds") as HTMLInputElement).addEventListener(
    "change",
    schedulePreviewRefresh,
  );
}

// プリセット都市データ
interface PresetCity {
  name: string;
  latitude: number;
  longitude: number;
}

const PRESET_CITIES: PresetCity[] = [
  { name: "東京",     latitude: 35.6762, longitude: 139.6503 },
  { name: "大阪",     latitude: 34.6937, longitude: 135.5023 },
  { name: "名古屋",   latitude: 35.1815, longitude: 136.9066 },
  { name: "札幌",     latitude: 43.0618, longitude: 141.3545 },
  { name: "仙台",     latitude: 38.2688, longitude: 140.8721 },
  { name: "広島",     latitude: 34.3853, longitude: 132.4553 },
  { name: "福岡",     latitude: 33.5904, longitude: 130.4017 },
  { name: "那覇",     latitude: 26.2124, longitude: 127.6809 },
  { name: "ニューヨーク", latitude: 40.7128, longitude: -74.0060 },
  { name: "ロンドン",  latitude: 51.5074, longitude: -0.1278 },
  { name: "パリ",     latitude: 48.8566, longitude: 2.3522 },
  { name: "シドニー",  latitude: -33.8688, longitude: 151.2093 },
];

const CUSTOM_VALUE = "__custom__";

export async function renderSettings(root: HTMLElement): Promise<void> {
  root.innerHTML = `
    <header class="app-header">
      <h1>🌤 Locksun</h1>
      <p class="subtitle">太陽光でロックスクリーンを彩る</p>
    </header>

    <section class="preview-section">
      <div id="sky-preview" class="sky-preview">
        <div class="preview-loading">読み込み中...</div>
      </div>
      <div id="sun-info" class="sun-info"></div>
    </section>

    <form id="settings-form" class="settings-form">
      <section class="form-section">
        <h2>📍 位置情報</h2>
        <div class="field-row">
          <label>地域を選択</label>
          <select id="city-preset">
            ${PRESET_CITIES.map((c) => `<option value="${c.name}">${c.name}</option>`).join("\n            ")}
            <option value="${CUSTOM_VALUE}">カスタム入力...</option>
          </select>
        </div>
        <div class="field-row">
          <label>場所の名前</label>
          <input type="text" id="location-name" placeholder="東京" />
        </div>
        <div class="field-row">
          <label>緯度</label>
          <input type="number" id="latitude" step="0.0001" min="-90" max="90" />
        </div>
        <div class="field-row">
          <label>経度</label>
          <input type="number" id="longitude" step="0.0001" min="-180" max="180" />
        </div>
      </section>

      <section class="form-section">
        <h2>🔄 更新設定</h2>
        <div class="field-row">
          <label>更新間隔</label>
          <select id="interval">
            <option value="60">1分</option>
            <option value="300">5分</option>
            <option value="600">10分</option>
            <option value="1800">30分</option>
            <option value="3600">1時間</option>
          </select>
        </div>
      </section>

      <section class="form-section">
        <h2>🖼 画像設定</h2>
        <div class="field-row">
          <label for="water-depth">水深 (0.0 = 浅い青緑, 1.0 = 深い青)</label>
          <div class="slider-container">
            <input type="range" id="water-depth" min="0" max="1" step="0.1" value="0.7" />
            <span class="slider-value" id="water-depth-value">0.7</span>
          </div>
        </div>
        <div class="field-row checkbox-row">
          <label>
            <input type="checkbox" id="show-stars" />
            夜間に星を表示
          </label>
        </div>
        <div class="field-row checkbox-row">
          <label>
            <input type="checkbox" id="show-clouds" />
            雲エフェクト (実験的)
          </label>
        </div>
      </section>

      <section class="form-section">
        <h2>⚙️ 動作設定</h2>
        <div class="field-row checkbox-row">
          <label>
            <input type="checkbox" id="autostart" />
            Windows 起動時に自動起動
          </label>
        </div>
      </section>

      <section class="form-section">
        <h2>✨ Gemini AI 強化</h2>
        <div class="field-row checkbox-row">
          <label>
            <input type="checkbox" id="gemini-enabled" />
            AI による画像強化を有効にする
          </label>
        </div>
        <div class="field-row">
          <label>API キー</label>
          <input type="password" id="gemini-api-key" placeholder="AIzaSy..." autocomplete="off" />
        </div>
        <div class="field-row">
          <label>モデル名</label>
          <input type="text" id="gemini-model" placeholder="gemini-2.0-flash-exp" />
        </div>
        <div class="field-row">
          <label>強化プロンプト</label>
          <textarea id="gemini-prompt" rows="3" placeholder="Enhance this sky image to look photorealistic..."></textarea>
        </div>
        <div class="field-row">
          <button type="button" id="btn-ai-preview" class="btn btn-secondary">AI強化プレビュー</button>
        </div>
      </section>

      <div class="form-actions">
        <button type="button" id="btn-preview" class="btn btn-secondary">プレビュー更新</button>
        <button type="submit" class="btn btn-primary">設定を保存</button>
      </div>
      <div id="status-msg" class="status-msg"></div>
    </form>
  `;

  try {
    await loadAndBindConfig();
  } catch (e) {
    console.error("設定の読み込みに失敗しました:", e);
    const status = document.getElementById("status-msg");
    if (status) {
      // Tauri IPC が利用できない場合の特別なメッセージ
      if (e && e.toString().includes("Tauri IPC が利用できません")) {
        status.textContent = `⚠️ 設定読み込み失敗: Error: Tauri IPC が利用できません。ブラウザではなく Tauri アプリウィンドウで開いてください。(cmd: get_config)`;
      } else {
        status.textContent = `⚠️ 設定読み込み失敗: ${e}`;
      }
      status.className = "status-msg error";
    }

    // デフォルト値で初期化
    await loadDefaultConfig();
  }

  await refreshPreview();
  await refreshSunInfo();

  document
    .getElementById("btn-preview")!
    .addEventListener("click", refreshPreview);

  document
    .getElementById("btn-ai-preview")!
    .addEventListener("click", refreshAiPreview);

  const form = document.getElementById("settings-form")!;
  form.addEventListener("submit", onSave);
  // input 要素での Enter キーによるフォーム送信（ページリロード）を防ぐ
  form.addEventListener("keydown", (e: KeyboardEvent) => {
    if (e.key === "Enter" && (e.target as HTMLElement).tagName !== "BUTTON") {
      e.preventDefault();
    }
  });
}

async function loadAndBindConfig(): Promise<void> {
  const cfg = await getConfig();
  (document.getElementById("location-name") as HTMLInputElement).value =
    cfg.location.name;
  (document.getElementById("latitude") as HTMLInputElement).value = String(
    cfg.location.latitude,
  );
  (document.getElementById("longitude") as HTMLInputElement).value = String(
    cfg.location.longitude,
  );
  (document.getElementById("interval") as HTMLSelectElement).value = String(
    cfg.update.interval_secs,
  );
  (document.getElementById("show-stars") as HTMLInputElement).checked =
    cfg.image.show_stars;
  (document.getElementById("show-clouds") as HTMLInputElement).checked =
    cfg.image.show_clouds;
  (document.getElementById("autostart") as HTMLInputElement).checked =
    cfg.behavior.autostart;

  // Gemini 設定の復元
  (document.getElementById("gemini-enabled") as HTMLInputElement).checked =
    cfg.gemini.enabled;
  // API キーはセキュリティのため表示しない（プレースホルダーで存在を示す）
  loadedGeminiApiKey = cfg.gemini.api_key;
  const apiKeyInput = document.getElementById("gemini-api-key") as HTMLInputElement;
  apiKeyInput.placeholder = cfg.gemini.api_key ? "••••••••（設定済み）" : "AIzaSy...";
  apiKeyInput.value = "";
  (document.getElementById("gemini-model") as HTMLInputElement).value =
    cfg.gemini.model_name;
  (document.getElementById("gemini-prompt") as HTMLTextAreaElement).value =
    cfg.gemini.enhance_prompt;

  // 水深スライダーの設定
  const waterDepthSlider = document.getElementById(
    "water-depth",
  ) as HTMLInputElement;
  const waterDepthValue = document.getElementById(
    "water-depth-value",
  ) as HTMLSpanElement;
  waterDepthSlider.value = String(cfg.image.water_depth);
  waterDepthValue.textContent = String(cfg.image.water_depth);

  bindPreviewSettingsEvents();

  // プリセット都市の照合と選択
  syncCityPreset(cfg.location.latitude, cfg.location.longitude);
  bindCityPresetEvents();
}

/**
 * Tauri IPC が利用できない場合のデフォルト設定を読み込む
 */
async function loadDefaultConfig(): Promise<void> {
  console.log("🔧 デフォルト設定を読み込み中...");

  // デフォルト値で初期化
  (document.getElementById("location-name") as HTMLInputElement).value = "東京";
  (document.getElementById("latitude") as HTMLInputElement).value = "35.6762";
  (document.getElementById("longitude") as HTMLInputElement).value = "139.6503";
  (document.getElementById("interval") as HTMLSelectElement).value = "300"; // 5分
  (document.getElementById("show-stars") as HTMLInputElement).checked = true;
  (document.getElementById("show-clouds") as HTMLInputElement).checked = false;
  (document.getElementById("autostart") as HTMLInputElement).checked = false;

  // Gemini のデフォルト値
  (document.getElementById("gemini-enabled") as HTMLInputElement).checked = false;
  (document.getElementById("gemini-api-key") as HTMLInputElement).value = "";
  (document.getElementById("gemini-model") as HTMLInputElement).value = "gemini-2.0-flash-exp";
  (document.getElementById("gemini-prompt") as HTMLTextAreaElement).value =
    "Enhance this sky image to look photorealistic, like a high-quality photograph. Preserve the sun position and sky colors but add natural cloud textures, atmospheric haze, and photographic quality.";

  // 水深のデフォルト値
  const waterDepthSlider = document.getElementById(
    "water-depth",
  ) as HTMLInputElement;
  const waterDepthValue = document.getElementById(
    "water-depth-value",
  ) as HTMLSpanElement;
  waterDepthSlider.value = "0.7";
  waterDepthValue.textContent = "0.7";

  bindPreviewSettingsEvents();

  // プリセット都市の照合と選択
  syncCityPreset(35.6762, 139.6503);
  bindCityPresetEvents();

  console.log("✅ デフォルト設定の読み込みが完了しました");
}

/** 現在の緯度・経度がプリセットに一致するか確認し、ドロップダウンを同期する */
function syncCityPreset(lat: number, lon: number): void {
  const select = document.getElementById("city-preset") as HTMLSelectElement;
  const matched = PRESET_CITIES.find(
    (c) =>
      Math.abs(c.latitude - lat) < 0.001 &&
      Math.abs(c.longitude - lon) < 0.001,
  );
  select.value = matched ? matched.name : CUSTOM_VALUE;
}

/** 都市選択ドロップダウンと手動入力フィールドのイベントを登録する */
function bindCityPresetEvents(): void {
  const select = document.getElementById("city-preset") as HTMLSelectElement;
  const nameInput = document.getElementById("location-name") as HTMLInputElement;
  const latInput = document.getElementById("latitude") as HTMLInputElement;
  const lonInput = document.getElementById("longitude") as HTMLInputElement;

  // 都市を選択したとき → フィールドを自動入力 → プレビュー更新
  select.addEventListener("change", () => {
    if (select.value === CUSTOM_VALUE) return;
    const city = PRESET_CITIES.find((c) => c.name === select.value);
    if (!city) return;
    nameInput.value = city.name;
    latInput.value = String(city.latitude);
    lonInput.value = String(city.longitude);
    schedulePreviewRefresh();
  });

  // 手動入力したとき → 「カスタム入力...」に切り替え → プレビュー更新
  const setCustomAndRefresh = (): void => {
    select.value = CUSTOM_VALUE;
    schedulePreviewRefresh();
  };
  latInput.addEventListener("change", setCustomAndRefresh);
  lonInput.addEventListener("change", setCustomAndRefresh);
  nameInput.addEventListener("change", setCustomAndRefresh);
}

async function refreshPreview(): Promise<void> {
  const el = document.getElementById("sky-preview")!;
  el.innerHTML = `<div class="preview-loading">生成中...</div>`;
  try {
    const dataUrl = await previewImage();
    el.innerHTML = `<img src="${dataUrl}" alt="空のプレビュー" />`;
  } catch (e) {
    console.error("プレビュー生成エラー:", e);

    if (e && e.toString().includes("Tauri IPC が利用できません")) {
      el.innerHTML = `
        <div class="preview-error">
          <div>📱 プレビュー生成失敗</div>
          <div style="font-size: 0.9em; margin-top: 8px; color: #888;">
            Tauri IPC が利用できません。<br>
            ブラウザではなく Tauri アプリウィンドウで開いてください。
          </div>
        </div>
      `;
    } else {
      el.innerHTML = `<div class="preview-error">プレビュー生成失敗: ${e}</div>`;
    }
  }
}

async function refreshAiPreview(): Promise<void> {
  const el = document.getElementById("sky-preview")!;
  el.innerHTML = `<div class="preview-loading">AI 強化中...</div>`;
  try {
    const dataUrl = await previewImageEnhanced();
    el.innerHTML = `<img src="${dataUrl}" alt="AI 強化済み空のプレビュー" />`;
  } catch (e) {
    console.error("AI 強化プレビューエラー:", e);
    el.innerHTML = `<div class="preview-error">AI 強化プレビュー失敗: ${e}</div>`;
  }
}

async function refreshSunInfo(): Promise<void> {
  const el = document.getElementById("sun-info")!;
  try {
    const info = await getSunInfo();
    const sr = info.times.sunrise
      ? new Date(info.times.sunrise).toLocaleTimeString("ja-JP")
      : "---";
    const ss = info.times.sunset
      ? new Date(info.times.sunset).toLocaleTimeString("ja-JP")
      : "---";
    el.innerHTML = `
      <span>📍 ${info.location_name}</span>
      <span>高度 ${info.position.altitude.toFixed(1)}°</span>
      <span>🌅 ${sr}</span>
      <span>🌇 ${ss}</span>
    `;
  } catch (e) {
    console.error("太陽情報取得エラー:", e);

    if (e && e.toString().includes("Tauri IPC が利用できません")) {
      el.innerHTML = `<span style="color: #888;">太陽情報取得失敗 (Tauri IPC エラー)</span>`;
    } else {
      el.textContent = "太陽情報の取得に失敗";
    }
  }
}

function validateConfig(cfg: AppConfig): string | null {
  if (
    isNaN(cfg.location.latitude) ||
    cfg.location.latitude < -90 ||
    cfg.location.latitude > 90
  ) {
    return "緯度は -90 〜 90 の数値を入力してください";
  }
  if (
    isNaN(cfg.location.longitude) ||
    cfg.location.longitude < -180 ||
    cfg.location.longitude > 180
  ) {
    return "経度は -180 〜 180 の数値を入力してください";
  }
  if (!cfg.location.name.trim()) {
    return "場所の名前を入力してください";
  }
  return null;
}

async function onSave(e: SubmitEvent): Promise<void> {
  e.preventDefault();
  const status = document.getElementById("status-msg")!;

  const cfg = buildConfigFromForm();

  const validationError = validateConfig(cfg);
  if (validationError !== null) {
    status.textContent = `⚠️ ${validationError}`;
    status.className = "status-msg error";
    setTimeout(() => {
      status.textContent = "";
    }, 3000);
    return;
  }

  try {
    await saveConfig(cfg);
    status.textContent = "✅ 設定を保存しました";
    status.className = "status-msg success";
    setTimeout(() => {
      status.textContent = "";
    }, 3000);
    await refreshPreview();
    await refreshSunInfo();
  } catch (err) {
    status.textContent = `❌ 保存失敗: ${err}`;
    status.className = "status-msg error";
  }
}
