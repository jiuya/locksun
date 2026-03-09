// src/main.ts
// フロントエンドのエントリーポイント

import { renderSettings } from "./pages/settings.js";

const app = document.getElementById("app")!;
renderSettings(app).catch((e: unknown) => {
  console.error("renderSettings の起動に失敗しました:", e);
});
