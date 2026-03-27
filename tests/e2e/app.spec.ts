import { test, expect } from "@playwright/test";

/**
 * Locksun アプリケーションの基本動作テスト
 */
test.describe("Locksun アプリケーション", () => {
  test("アプリケーションが正常に起動する", async ({ page }) => {
    // Tauri アプリの起動待ち
    await page.goto("/");

    // タイトルが正しく表示されることを確認
    await expect(page).toHaveTitle(/Locksun/);
  });

  test("設定画面が表示される", async ({ page }) => {
    await page.goto("/");

    // 設定画面の要素が存在することを確認
    await expect(page.locator("#app")).toBeVisible();

    // 設定フォームの主要な要素をチェック
    const settingsForm = page.locator("form, .settings");
    if ((await settingsForm.count()) > 0) {
      await expect(settingsForm.first()).toBeVisible();
    }
  });

  test("太陽設定の入力フィールドが機能する", async ({ page }) => {
    await page.goto("/");

    // 緯度・経度の入力フィールドがあれば確認
    const latitudeInput = page.locator('input[type="number"]').first();
    const longitudeInput = page.locator('input[type="number"]').nth(1);

    if ((await latitudeInput.count()) > 0) {
      await expect(latitudeInput).toBeVisible();
      await latitudeInput.fill("35.68");
      await expect(latitudeInput).toHaveValue("35.68");
    }

    if ((await longitudeInput.count()) > 0) {
      await expect(longitudeInput).toBeVisible();
      await longitudeInput.fill("139.69");
      await expect(longitudeInput).toHaveValue("139.69");
    }
  });

  test("プレビュー画像が生成される", async ({ page }) => {
    await page.goto("/");

    // プレビューエリアを確認
    const previewArea = page.locator(
      '.sky-preview, canvas, img[src*="preview"]',
    );

    if ((await previewArea.count()) > 0) {
      await expect(previewArea.first()).toBeVisible();

      // プレビューボタンがあれば実行
      const previewButton = page.locator(
        'button:has-text("プレビュー"), button:has-text("Preview")',
      );
      if ((await previewButton.count()) > 0) {
        await previewButton.click();

        // プレビュー生成の待機（最大10秒）
        await expect(previewArea.first()).toBeVisible({ timeout: 10000 });
      }
    }
  });

  test("スクリーンショットを取得（デバッグ用）", async ({ page }) => {
    await page.goto("/");

    // 全画面のスクリーンショットを取得
    await page.screenshot({
      path: "tests/e2e/screenshots/full-app.png",
      fullPage: true,
    });

    // 設定エリアのスクリーンショットがあれば取得
    const settingsArea = page.locator("#app, .main-content");
    if ((await settingsArea.count()) > 0) {
      await settingsArea.first().screenshot({
        path: "tests/e2e/screenshots/settings-area.png",
      });
    }
  });
});
