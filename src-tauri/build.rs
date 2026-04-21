fn main() {
    #[cfg(target_os = "windows")]
    {
        let profile = std::env::var("PROFILE").unwrap_or_default();
        let mut windows = tauri_build::WindowsAttributes::new();
        // リリースビルドのみ requireAdministrator マニフェストを適用する。
        // デバッグビルドで適用すると `cargo run` が os error 740 で失敗するため。
        if profile == "release" {
            windows = windows.app_manifest(include_str!("locksun.exe.manifest"));
        }
        let attrs = tauri_build::Attributes::new().windows_attributes(windows);
        tauri_build::try_build(attrs).expect("failed to run tauri-build");
        return;
    }
    tauri_build::build();
}
