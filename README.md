# Locksun
[![Auto Release](https://github.com/jiuya/locksun/actions/workflows/release.yml/badge.svg)](https://github.com/jiuya/locksun/actions/workflows/release.yml)

Windowsのロックスクリーン画像を、現在時刻と位置情報に基づく太陽の動きでリアルタイムに変化させる常駐アプリです。

## 技術スタック

| 層 | 技術 |
|----|------|
| バックエンド | Rust + Tauri v2 |
| フロントエンド | TypeScript + Vite |
| 画像生成 | `image` クレート |
| Windows API | `winreg` クレート |

## セットアップ

### 前提条件

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) >= 20
- [Tauri CLI](https://tauri.app/start/prerequisites/) の前提ライブラリ (WebView2 等)

```powershell
# 依存関係インストール
npm install

# 開発サーバー起動（ホットリロードあり）
npm run tauri dev

# リリースビルド
npm run tauri build
```

## Gemini API キーの設定

Gemini AI 画像強化機能を使う場合は、**APIキーをファイルに直書きせず**環境変数または設定ファイルで管理してください。

```powershell
# 方法1: 環境変数で設定（推奨）
$env:GEMINI_API_KEY = "AIza..."
npm run tauri dev

# 方法2: 設定ファイルで設定（.gitignore 済みのため安全）
Copy-Item src-tauri\config\user_settings.toml.example src-tauri\config\user_settings.toml
# user_settings.toml を編集して api_key = "AIza..." を設定
```

> ⚠️ `src-tauri/config/user_settings.toml` および `config/user_settings.toml` は `.gitignore` に登録されています。APIキーが含まれたファイルを誤ってコミットしないようにしてください。

## プロジェクト構造

```
locksun/
├── src-tauri/              # Rust バックエンド
│   └── src/
│       ├── sun/            # 太陽位置計算
│       ├── renderer/       # 画像生成
│       ├── lockscreen/     # Windows レジストリ操作
│       ├── scheduler/      # 定期更新
│       ├── config/         # 設定管理
│       └── commands/       # Tauri IPC コマンド
├── src/                    # TypeScript フロントエンド
│   ├── api/                # Tauri コマンドラッパー
│   ├── pages/              # 設定画面
│   └── styles/
├── config/                 # 設定ファイル
└── tests/                  # 統合テスト
```

## ロックスクリーン変更の仕組み

Windows レジストリの `PersonalizationCSP` キーに画像パスを書き込むことで変更します。
書き込みには **管理者権限** が必要です。

```
HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\PersonalizationCSP
  LockScreenImagePath   = C:\...\lockscreen.png
  LockScreenImageStatus = 1
  LockScreenImageUrl    = C:\...\lockscreen.png
```

## テスト

```powershell
cd src-tauri
cargo test
```

## ライセンス

MIT


