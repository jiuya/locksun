import { test } from "@playwright/test";
import {
  generateComprehensiveReport,
  checkAppBasicState,
  testPreviewFunction,
  checkFormInteractions,
} from "./webview-helpers.js";

/**
 * エージェント用包括的 WebView 確認テスト
 * このテストはエージェントが UI 状態を自動確認するために使用します
 */
test.describe("エージェント: WebView 自動確認", () => {
  test("包括的な UI 状態確認", async ({ page }) => {
    console.log("🤖 エージェント: WebView の包括的確認を開始します...");

    // 包括的なレポートを生成
    const report = await generateComprehensiveReport(page);

    // レポートをコンソールに出力（エージェントが読み取れるように）
    console.log("📊 === WebView 確認レポート ===");
    console.log(`⏰ 確認時刻: ${report.timestamp}`);
    console.log(`🌟 総合的な健全性: ${report.summary.overallHealth}`);

    // 基本状態
    console.log("\n📱 === 基本状態 ===");
    console.log(
      `✅ アプリ起動: ${report.basicState.appStarted ? "成功" : "❌失敗"}`,
    );
    console.log(
      `✅ タイトル正確: ${report.basicState.titleCorrect ? "正常" : "❌異常"}`,
    );
    console.log(
      `✅ メイン要素表示: ${report.basicState.mainElementVisible ? "表示中" : "❌非表示"}`,
    );
    console.log(`🐞 JavaScript エラー: ${report.basicState.jsErrorsCount}件`);
    console.log(
      `📷 スクリーンショット: ${report.basicState.screenshotTaken ? "取得済み" : "❌失敗"}`,
    );

    // 設定要素
    console.log("\n⚙️ === 設定要素 ===");
    console.log(
      `📍 緯度入力: ${report.settingsElements.latitudeInput ? "利用可能" : "⚠️未検出"}`,
    );
    console.log(
      `📍 経度入力: ${report.settingsElements.longitudeInput ? "利用可能" : "⚠️未検出"}`,
    );
    console.log(
      `📝 場所名入力: ${report.settingsElements.locationInput ? "利用可能" : "⚠️未検出"}`,
    );
    console.log(
      `⏰ 更新間隔: ${report.settingsElements.updateInterval ? "利用可能" : "⚠️未検出"}`,
    );
    console.log(
      `👁️ プレビューボタン: ${report.settingsElements.previewButton ? "利用可能" : "⚠️未検出"}`,
    );
    console.log(
      `📋 設定フォーム: ${report.settingsElements.settingsForm ? "検出" : "❌未検出"}`,
    );

    // フォーム動作
    console.log("\n📝 === フォーム動作 ===");
    console.log(
      `🔤 機能する入力: ${report.formInteractions.inputsFunctional}/${report.formInteractions.totalInputs}`,
    );
    console.log(
      `📋 機能するセレクト: ${report.formInteractions.selectsFunctional}/${report.formInteractions.totalSelects}`,
    );
    console.log(
      `🔘 機能するボタン: ${report.formInteractions.buttonsFunctional}/${report.formInteractions.totalButtons}`,
    );

    // プレビューテスト
    console.log("\n👁️ === プレビュー機能 ===");
    console.log(
      `📝 入力更新: ${report.previewTest.inputsUpdated ? "成功" : "⚠️スキップ"}`,
    );
    console.log(
      `🎨 プレビュー生成: ${report.previewTest.previewGenerated ? "実行" : "⚠️スキップ"}`,
    );
    console.log(
      `🖼️ Canvas表示: ${report.previewTest.canvasVisible ? "成功" : "⚠️未確認"}`,
    );
    console.log(
      `🖼️ 画像表示: ${report.previewTest.imageVisible ? "成功" : "⚠️未確認"}`,
    );
    console.log(
      `❌ エラー発生: ${report.previewTest.errorOccurred ? "あり" : "なし"}`,
    );

    // 問題点と推奨事項
    if (report.summary.criticalIssues.length > 0) {
      console.log("\n🚨 === 重要な問題 ===");
      report.summary.criticalIssues.forEach((issue, index) => {
        console.log(`${index + 1}. ${issue}`);
      });
    }

    if (report.summary.recommendations.length > 0) {
      console.log("\n💡 === 推奨事項 ===");
      report.summary.recommendations.forEach((rec, index) => {
        console.log(`${index + 1}. ${rec}`);
      });
    }

    // 取得したスクリーンショットのパス情報
    console.log("\n📷 === スクリーンショット ===");
    console.log("取得された画像ファイル:");
    console.log("- tests/e2e/screenshots/basic-state.png");
    console.log("- tests/e2e/screenshots/settings-panel.png");
    console.log(
      "- tests/e2e/screenshots/preview-canvas.png または preview-image.png",
    );

    console.log("\n🤖 エージェント: WebView の確認を完了しました。");

    // JSON形式でも出力（プログラム的に読み取りやすくするため）
    console.log("\n📋 === JSONレポート ===");
    console.log(JSON.stringify(report, null, 2));
  });

  test("クイック健全性確認（軽量版）", async ({ page }) => {
    console.log("🚀 エージェント: クイック WebView 確認を開始...");

    // 基本状態のみ確認
    const basicState = await checkAppBasicState(page);

    // デバッグ用: IPC接続状態をテスト
    console.log("🔍 === IPC 接続デバッグ ===");
    try {
      const ipcResult = await page.evaluate(async () => {
        // グローバル関数 testIPC が利用可能かチェック
        if (typeof (window as any).testIPC === "function") {
          return await (window as any).testIPC();
        } else {
          return {
            error: "testIPC function not available",
            tauriInternals:
              typeof (window as any).__TAURI_INTERNALS__ !== "undefined",
            windowKeys: Object.keys(window).filter(
              (key) => key.includes("TAURI") || key.includes("tauri"),
            ),
            tauriInternalsDetail: (window as any).__TAURI_INTERNALS__
              ? "detected"
              : "not detected",
          };
        }
      });

      console.log("📋 IPC テスト結果:", JSON.stringify(ipcResult, null, 2));
    } catch (error) {
      console.log("❌ IPC デバッグエラー:", error);
    }

    console.log("⚡ === クイック確認結果 ===");
    console.log(`📱 アプリ起動: ${basicState.appStarted ? "✅" : "❌"}`);
    console.log(
      `📺 メイン画面: ${basicState.mainElementVisible ? "✅" : "❌"}`,
    );
    console.log(`🐞 JSエラー: ${basicState.jsErrorsCount}件`);
    console.log(
      `📷 スクリーンショット: ${basicState.screenshotTaken ? "✅" : "❌"}`,
    );

    // 基本的な健全性判定
    const isHealthy =
      basicState.appStarted &&
      basicState.mainElementVisible &&
      basicState.jsErrorsCount === 0;
    console.log(`🌟 総合結果: ${isHealthy ? "✅ 正常" : "⚠️ 問題あり"}`);

    console.log("🚀 クイック確認完了");
  });

  test("特定機能の動作確認（設定とプレビュー）", async ({ page }) => {
    console.log("🎯 エージェント: 特定機能の確認を開始...");

    // まず基本状態を確認
    const basicState = await checkAppBasicState(page);
    if (!basicState.appStarted) {
      console.log("❌ アプリが起動していないため、機能確認をスキップします");
      return;
    }

    // プレビュー機能をテスト（東京の座標で）
    console.log("🌍 東京の座標でプレビュー機能をテスト中...");
    const previewResult = await testPreviewFunction(page, 35.68, 139.69);

    console.log("🎨 === プレビュー機能テスト結果 ===");
    console.log(`📍 座標入力: ${previewResult.inputsUpdated ? "✅" : "⚠️"}`);
    console.log(
      `🎨 プレビュー実行: ${previewResult.previewGenerated ? "✅" : "⚠️"}`,
    );
    console.log(
      `🖼️ 描画結果: ${previewResult.canvasVisible || previewResult.imageVisible ? "✅" : "❌"}`,
    );
    console.log(`❌ エラー: ${previewResult.errorOccurred ? "あり" : "なし"}`);

    // フォームの動作確認
    console.log("📝 フォーム要素の動作確認中...");
    const formResult = await checkFormInteractions(page);

    console.log("📋 === フォーム動作確認結果 ===");
    console.log(
      `🔤 機能する入力フィールド: ${formResult.inputsFunctional}/${formResult.totalInputs}`,
    );
    console.log(
      `📋 機能するセレクト: ${formResult.selectsFunctional}/${formResult.totalSelects}`,
    );
    console.log(
      `🔘 機能するボタン: ${formResult.buttonsFunctional}/${formResult.totalButtons}`,
    );

    // 総合判定
    const functionalityScore =
      (previewResult.inputsUpdated ? 25 : 0) +
      (previewResult.previewGenerated ? 25 : 0) +
      (previewResult.canvasVisible || previewResult.imageVisible ? 25 : 0) +
      (formResult.inputsFunctional > 0 ? 25 : 0);

    console.log(`🎯 機能性スコア: ${functionalityScore}%`);
    console.log("🎯 特定機能の確認完了");
  });
});
