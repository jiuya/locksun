// src/main.ts
// フロントエンドのエントリーポイント

import { renderSettings } from "./pages/settings.js";

const app = document.getElementById("app")!;
renderSettings(app);
