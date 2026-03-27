#!/usr/bin/env node

/**
 * Playwright WebView 自動確認スクリプト
 * エージェントがコマンドライン用で簡単に実行できるスクリプト
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import fs from 'fs/promises';
import path from 'path';

const execAsync = promisify(exec);

// コマンドライン引数の解析
const args = process.argv.slice(2);
const command = args[0] || 'comprehensive';

async function executeCheck(testType) {
  console.log(`🤖 WebView 自動確認を開始します (タイプ: ${testType})`);
  
  try {
    let playwrightCommand;
    
    switch (testType) {
      case 'quick':
        playwrightCommand = 'npx playwright test agent-check.spec.ts -g "クイック健全性確認" --reporter=line';
        break;
      case 'functionality':
        playwrightCommand = 'npx playwright test agent-check.spec.ts -g "特定機能の動作確認" --reporter=line';
        break;
      case 'comprehensive':
      default:
        playwrightCommand = 'npx playwright test agent-check.spec.ts -g "包括的な UI 状態確認" --reporter=line';
        break;
    }

    console.log(`🔧 実行コマンド: ${playwrightCommand}`);
    
    // Playwright テストの実行
    const { stdout, stderr } = await execAsync(playwrightCommand);
    
    console.log('✅ テスト実行完了');
    console.log('\n📋 === 実行結果 ===');
    console.log(stdout);
    
    if (stderr) {
      console.log('\n⚠️ === 警告・エラー ===');
      console.log(stderr);
    }
    
    // スクリーンショットの確認
    await checkScreenshots();
    
    console.log('\n🎉 WebView 自動確認が完了しました');
    
  } catch (error) {
    console.error('❌ テスト実行中にエラーが発生しました:', error.message);
    
    // エラー詳細を表示
    if (error.stdout) {
      console.log('\n📋 === 標準出力 ===');
      console.log(error.stdout);
    }
    
    if (error.stderr) {
      console.log('\n⚠️ === エラー出力 ===');
      console.log(error.stderr);
    }
    
    process.exit(1);
  }
}

async function checkScreenshots() {
  const screenshotDir = path.join(process.cwd(), 'tests', 'e2e', 'screenshots');
  
  try {
    const files = await fs.readdir(screenshotDir);
    
    if (files.length > 0) {
      console.log('\n📷 === 取得されたスクリーンショット ===');
      for (const file of files) {
        const filePath = path.join(screenshotDir, file);
        const stats = await fs.stat(filePath);
        const sizeKB = Math.round(stats.size / 1024);
        console.log(`  📸 ${file} (${sizeKB}KB)`);
      }
      
      console.log('\n💡 スクリーンショットは以下で確認できます:');
      console.log(`   ${screenshotDir}`);
    } else {
      console.log('\n⚠️ スクリーンショットが取得されませんでした');
    }
  } catch (error) {
    console.log('\n⚠️ スクリーンショット確認中にエラーが発生しました');
  }
}

function showUsage() {
  console.log(`
🤖 Playwright WebView 自動確認スクリプト

使用方法:
  node webview-check.js [command]

コマンド:
  quick          - クイック確認（軽量、基本的な状態のみ）
  functionality  - 機能確認（設定とプレビュー機能）
  comprehensive  - 包括的確認（全ての項目を詳細チェック）【デフォルト】

例:
  node webview-check.js quick
  node webview-check.js comprehensive

📷 スクリーンショットは tests/e2e/screenshots/ に保存されます
📊 詳細なレポートはコンソール出力を確認してください
`);
}

// メイン実行
if (args.includes('--help') || args.includes('-h')) {
  showUsage();
  process.exit(0);
}

// 有効なコマンドかチェック
const validCommands = ['quick', 'functionality', 'comprehensive'];
if (command && !validCommands.includes(command)) {
  console.error(`❌ 無効なコマンド: ${command}`);
  console.error(`有効なコマンド: ${validCommands.join(', ')}`);
  process.exit(1);
}

executeCheck(command);