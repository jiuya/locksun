---
mode: agent
description: "新しい Tauri IPC コマンドをバックエンドとフロントエンドに追加する"
---

## タスク: Tauri コマンドの追加

### 1. Rust 側 (src-tauri/src/commands/mod.rs)

```rust
#[tauri::command]
pub fn <command_name>(<args>) -> Result<<ReturnType>, String> {
    // 実装
    // エラーは .map_err(|e| e.to_string()) で String に変換
}
```

### 2. lib.rs のハンドラー登録

`invoke_handler` の `tauri::generate_handler![]` マクロに追加:

```rust
.invoke_handler(tauri::generate_handler![
    commands::get_config,
    commands::save_config,
    commands::get_sun_info,
    commands::preview_image,
    commands::<command_name>,  // ← 追加
])
```

### 3. TypeScript 側 (src/api/tauri_commands.ts)

戻り値の型インターフェースを定義し、ラッパー関数を追加:

```typescript
// 型定義（Rust の構造体と一致させること）
export interface <ReturnType> {
  // フィールド（snake_case のまま）
}

// ラッパー関数
export const <commandName> = (<args>): Promise<<ReturnType>> =>
  invoke("<command_name>", { <args> });
```

### チェックリスト

- [ ] Rust の引数名とTypeScript の `invoke()` 第2引数のキー名が一致しているか
- [ ] 戻り値の型が両言語で対応しているか
- [ ] `derive(Serialize, Deserialize)` が Rust 構造体に付いているか
- [ ] エラーケースのテストを書いたか
