// src-tauri/examples/generate_test_image.rs
// テスト用画像生成プログラム

use chrono::Local;
use locksun_lib::{config, renderer, sun::SunCalculator};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // デフォルト設定で画像生成
    let cfg = config::load().unwrap_or_else(|_| config::AppConfig {
        location: config::LocationConfig {
            name: "東京".to_string(),
            latitude: 35.6762,
            longitude: 139.6503,
        },
        update: config::UpdateConfig { interval_secs: 300 },
        image: config::ImageConfig {
            width: 1920,
            height: 1080,
            show_stars: true,
            show_clouds: false,
            water_depth: 0.7, // 中程度の深さ
        },
        behavior: config::BehaviorConfig { autostart: false },
    });

    let now = Local::now();
    let pos = SunCalculator::position(&now, cfg.location.latitude, cfg.location.longitude);

    println!("太陽位置: 高度={}°, 方位角={}°", pos.altitude, pos.azimuth);
    println!("画像生成中...");

    // 画像生成
    let output_path = Path::new("test_reflection.png");
    renderer::render_and_save(&pos, &cfg.image, output_path)?;

    println!("✅ 画像を生成しました: {}", output_path.display());
    println!("水面の反射が自然で白飛びしていないことを確認してください。");

    Ok(())
}
