// src-tauri/src/main.rs
// Tauri アプリのエントリーポイント
// Windows では GUI サブシステムを使用し、コンソールウィンドウを非表示にする

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    locksun_lib::run();
}
