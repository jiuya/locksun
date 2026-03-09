// src/pages/settings.ts
// 設定ページのレンダリングと操作

import {
  type AppConfig,
  getConfig,
  getSunInfo,
  previewImage,
  saveConfig,
} from "../api/tauri_commands.js";

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

      <div class="form-actions">
        <button type="button" id="btn-preview" class="btn btn-secondary">プレビュー更新</button>
        <button type="submit" class="btn btn-primary">設定を保存</button>
      </div>
      <div id="status-msg" class="status-msg"></div>
    </form>
  `;

  await loadAndBindConfig();
  await refreshPreview();
  await refreshSunInfo();

  document.getElementById("btn-preview")!.addEventListener("click", refreshPreview);
  document.getElementById("settings-form")!.addEventListener("submit", onSave);
}

async function loadAndBindConfig(): Promise<void> {
  const cfg = await getConfig();
  (document.getElementById("location-name") as HTMLInputElement).value = cfg.location.name;
  (document.getElementById("latitude")      as HTMLInputElement).value = String(cfg.location.latitude);
  (document.getElementById("longitude")     as HTMLInputElement).value = String(cfg.location.longitude);
  (document.getElementById("interval")      as HTMLSelectElement).value = String(cfg.update.interval_secs);
  (document.getElementById("show-stars")    as HTMLInputElement).checked = cfg.image.show_stars;
  (document.getElementById("show-clouds")   as HTMLInputElement).checked = cfg.image.show_clouds;
  (document.getElementById("autostart")     as HTMLInputElement).checked = cfg.behavior.autostart;
}

async function refreshPreview(): Promise<void> {
  const el = document.getElementById("sky-preview")!;
  el.innerHTML = `<div class="preview-loading">生成中...</div>`;
  try {
    const dataUrl = await previewImage();
    el.innerHTML = `<img src="${dataUrl}" alt="空のプレビュー" />`;
  } catch (e) {
    el.innerHTML = `<div class="preview-error">プレビュー生成失敗: ${e}</div>`;
  }
}

async function refreshSunInfo(): Promise<void> {
  const el = document.getElementById("sun-info")!;
  try {
    const info = await getSunInfo();
    const sr = info.times.sunrise ? new Date(info.times.sunrise).toLocaleTimeString("ja-JP") : "---";
    const ss = info.times.sunset  ? new Date(info.times.sunset).toLocaleTimeString("ja-JP")  : "---";
    el.innerHTML = `
      <span>📍 ${info.location_name}</span>
      <span>高度 ${info.position.altitude.toFixed(1)}°</span>
      <span>🌅 ${sr}</span>
      <span>🌇 ${ss}</span>
    `;
  } catch (e) {
    el.textContent = "太陽情報の取得に失敗";
  }
}

async function onSave(e: SubmitEvent): Promise<void> {
  e.preventDefault();
  const status = document.getElementById("status-msg")!;

  const cfg: AppConfig = {
    location: {
      name:      (document.getElementById("location-name") as HTMLInputElement).value,
      latitude:  parseFloat((document.getElementById("latitude")  as HTMLInputElement).value),
      longitude: parseFloat((document.getElementById("longitude") as HTMLInputElement).value),
    },
    update: {
      interval_secs: parseInt((document.getElementById("interval") as HTMLSelectElement).value),
    },
    image: {
      width:       1920,
      height:      1080,
      show_stars:  (document.getElementById("show-stars")  as HTMLInputElement).checked,
      show_clouds: (document.getElementById("show-clouds") as HTMLInputElement).checked,
    },
    behavior: {
      autostart: (document.getElementById("autostart") as HTMLInputElement).checked,
    },
  };

  try {
    await saveConfig(cfg);
    status.textContent = "✅ 設定を保存しました";
    status.className   = "status-msg success";
    setTimeout(() => { status.textContent = ""; }, 3000);
    await refreshPreview();
    await refreshSunInfo();
  } catch (err) {
    status.textContent = `❌ 保存失敗: ${err}`;
    status.className   = "status-msg error";
  }
}
