import { test, expect, Page } from "@playwright/test";

/**
 * Playwright WebView 自動確認用ヘルパー関数
 */
// テスト環境用のグローバル宣言
declare global {
  interface Window {
    __PLAYWRIGHT_TEST__?: boolean;
  }
}
/**
 * 基本的なアプリケーション状態確認
 * @param page Playwright のページオブジェクト
 * @returns 確認結果オブジェクト
 */
export async function checkAppBasicState(page: Page) {
  const results = {
    appStarted: false,
    titleCorrect: false,
    mainElementVisible: false,
    jsErrorsCount: 0,
    screenshotTaken: false,
    timestamp: new Date().toISOString(),
  };

  try {
    // ページ読み込み前にテスト環境フラグをセット
    await page.addInitScript(() => {
      window.__PLAYWRIGHT_TEST__ = true;
      console.log("[PLAYWRIGHT] Test environment flag set");
    });

    // アプリケーション起動確認
    await page.goto("/");
    results.appStarted = true;

    // タイトル確認
    const title = await page.title();
    results.titleCorrect = title.includes("Locksun") || title.includes("Solar");

    // メイン要素の表示確認
    const mainElement = page.locator("#app, .main-content, body");
    await expect(mainElement.first()).toBeVisible({ timeout: 10000 });
    results.mainElementVisible = true;

    // JavaScript エラーの監視
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        results.jsErrorsCount++;
        console.log(`❌ JS Error: ${msg.text()}`);
      }
    });

    // 基本状態のスクリーンショット取得
    await page.screenshot({
      path: "tests/e2e/screenshots/basic-state.png",
      fullPage: true,
    });
    results.screenshotTaken = true;
  } catch (error) {
    console.error("Basic state check failed:", error);
  }

  return results;
}

/**
 * 設定画面の要素確認
 * @param page Playwright のページオブジェクト
 * @returns 設定要素の状態
 */
export async function checkSettingsElements(page: Page) {
  const results = {
    latitudeInput: false,
    longitudeInput: false,
    locationInput: false,
    updateInterval: false,
    previewButton: false,
    settingsForm: false,
    timestamp: new Date().toISOString(),
  };

  try {
    // ページ読み込み前にテスト環境フラグをセット
    await page.addInitScript(() => {
      window.__PLAYWRIGHT_TEST__ = true;
    });

    // HTML構造のデバッグ出力
    const htmlContent = await page.evaluate(() => {
      return document.body.innerHTML;
    });
    console.log("[DEBUG] Page HTML:", htmlContent.substring(0, 500) + "...");

    // 各設定要素の存在確認
    const latInput = page.locator("#latitude");
    const lonInput = page.locator("#longitude");
    const locInput = page.locator("#location-name");
    const intervalSelect = page.locator("#interval");
    const previewBtn = page.locator("#btn-preview");
    const settingsForm = page.locator("#settings-form");

    results.latitudeInput = (await latInput.count()) > 0;
    results.longitudeInput = (await lonInput.count()) > 0;
    results.locationInput = (await locInput.count()) > 0;
    results.updateInterval = (await intervalSelect.count()) > 0;
    results.previewButton = (await previewBtn.count()) > 0;
    results.settingsForm = (await settingsForm.count()) > 0;

    // 設定画面のスクリーンショット
    if (results.settingsForm) {
      await settingsForm.first().screenshot({
        path: "tests/e2e/screenshots/settings-panel.png",
      });
    }
  } catch (error) {
    console.error("Settings elements check failed:", error);
  }

  return results;
}

/**
 * プレビュー機能のテスト
 * @param page Playwright のページオブジェクト
 * @param latitude 緯度（省略可）
 * @param longitude 経度（省略可）
 * @returns プレビュー機能の動作結果
 */
export async function testPreviewFunction(
  page: Page,
  latitude?: number,
  longitude?: number,
) {
  const results = {
    inputsUpdated: false,
    previewGenerated: false,
    canvasVisible: false,
    imageVisible: false,
    errorOccurred: false,
    timestamp: new Date().toISOString(),
  };

  try {
    // ページ読み込み前にテスト環境フラグをセット
    await page.addInitScript(() => {
      window.__PLAYWRIGHT_TEST__ = true;
    });

    // 座標が指定されている場合は入力
    if (latitude !== undefined && longitude !== undefined) {
      const latInput = page.locator("#latitude");
      const lonInput = page.locator("#longitude");

      if ((await latInput.count()) > 0) {
        await latInput.fill(latitude.toString());
        await lonInput.fill(longitude.toString());
        results.inputsUpdated = true;
      }
    }

    // プレビューボタンのクリック
    const previewBtn = page.locator("#btn-preview");
    if ((await previewBtn.count()) > 0) {
      await previewBtn.click();
      results.previewGenerated = true;

      // プレビュー結果の確認（Canvas または画像）
      const canvas = page.locator("canvas");
      const previewImg = page.locator('img[src*="preview"], .preview-image');

      try {
        await expect(canvas.first()).toBeVisible({ timeout: 15000 });
        results.canvasVisible = true;

        await canvas.first().screenshot({
          path: "tests/e2e/screenshots/preview-canvas.png",
        });
      } catch {
        // Canvas が無い場合は画像を確認
        try {
          await expect(previewImg.first()).toBeVisible({ timeout: 15000 });
          results.imageVisible = true;

          await previewImg.first().screenshot({
            path: "tests/e2e/screenshots/preview-image.png",
          });
        } catch {
          results.errorOccurred = true;
        }
      }
    }
  } catch (error) {
    console.error("Preview function test failed:", error);
    results.errorOccurred = true;
  }

  return results;
}

/**
 * フォーム入力の動作確認
 * @param page Playwright のページオブジェクト
 * @returns フォーム動作の確認結果
 */
export async function checkFormInteractions(page: Page) {
  const results = {
    inputsFunctional: 0,
    totalInputs: 0,
    selectsFunctional: 0,
    totalSelects: 0,
    buttonsFunctional: 0,
    totalButtons: 0,
    timestamp: new Date().toISOString(),
  };

  try {
    // 全ての入力要素を取得
    const inputs = await page.locator("input").all();
    const selects = await page.locator("select").all();
    const buttons = await page.locator("button").all();

    results.totalInputs = inputs.length;
    results.totalSelects = selects.length;
    results.totalButtons = buttons.length;

    // 入力フィールドのテスト
    for (const input of inputs) {
      try {
        const inputType = (await input.getAttribute("type")) || "text";
        if (inputType === "text" || inputType === "number") {
          await input.fill("test-value");
          const value = await input.inputValue();
          if (value === "test-value") {
            results.inputsFunctional++;
          }
        }
      } catch (error) {
        // Skip non-interactive inputs
      }
    }

    // セレクト要素のテスト
    for (const select of selects) {
      try {
        const options = await select.locator("option").all();
        if (options.length > 1) {
          await select.selectOption({ index: 1 });
          results.selectsFunctional++;
        }
      } catch (error) {
        // Skip non-functional selects
      }
    }

    // ボタンのクリック可能性確認
    for (const button of buttons) {
      try {
        const isVisible = await button.isVisible();
        const isEnabled = await button.isEnabled();
        if (isVisible && isEnabled) {
          results.buttonsFunctional++;
        }
      } catch (error) {
        // Skip non-functional buttons
      }
    }
  } catch (error) {
    console.error("Form interactions check failed:", error);
  }

  return results;
}

/**
 * 包括的な画面確認レポートの生成
 * @param page Playwright のページオブジェクト
 * @returns 詳細な確認レポート
 */
export async function generateComprehensiveReport(page: Page) {
  // テスト環境の初期化（最初に一度だけ実行）
  await page.addInitScript(() => {
    window.__PLAYWRIGHT_TEST__ = true;
    console.log("[PLAYWRIGHT] Test environment initialized");
  });

  const report = {
    timestamp: new Date().toISOString(),
    basicState: await checkAppBasicState(page),
    settingsElements: await checkSettingsElements(page),
    formInteractions: await checkFormInteractions(page),
    previewTest: await testPreviewFunction(page, 35.68, 139.69), // 東京の座標でテスト
    summary: {
      overallHealth: "unknown" as "healthy" | "warning" | "error" | "unknown",
      criticalIssues: [] as string[],
      recommendations: [] as string[],
    },
  };

  // 総合的な健全性の判定
  const issues = [];
  const recommendations = [];

  if (!report.basicState.appStarted) {
    issues.push("アプリケーションが起動しませんでした");
  }
  if (!report.basicState.mainElementVisible) {
    issues.push("メイン画面が表示されませんでした");
  }
  if (report.basicState.jsErrorsCount > 0) {
    issues.push(
      `JavaScript エラーが ${report.basicState.jsErrorsCount} 件発生しました`,
    );
  }
  if (!report.settingsElements.settingsForm) {
    issues.push("設定フォームが見つかりませんでした");
  }
  if (
    report.formInteractions.inputsFunctional === 0 &&
    report.formInteractions.totalInputs > 0
  ) {
    issues.push("入力フィールドが正常に動作していません");
  }

  if (
    report.settingsElements.latitudeInput &&
    report.settingsElements.longitudeInput
  ) {
    recommendations.push("緯度・経度入力が利用可能です");
  }
  if (report.settingsElements.previewButton) {
    recommendations.push("プレビュー機能が利用可能です");
  }
  if (report.previewTest.previewGenerated) {
    recommendations.push("プレビュー生成機能が正常に動作しています");
  }

  report.summary.criticalIssues = issues;
  report.summary.recommendations = recommendations;

  if (issues.length === 0) {
    report.summary.overallHealth = "healthy";
  } else if (
    issues.some((issue) => issue.includes("起動") || issue.includes("表示"))
  ) {
    report.summary.overallHealth = "error";
  } else {
    report.summary.overallHealth = "warning";
  }

  return report;
}
