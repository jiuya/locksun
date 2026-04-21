// src-tauri/examples/generate_gemini_test.rs
// Gemini API 画像強化のシンプルなテストプログラム
//
// 使用方法:
//   $env:GEMINI_API_KEY = "AIza..."
//   $env:RUST_LOG = "debug"          # Gemini レスポンスの詳細ログを表示したい場合
//   cargo run --example generate_gemini_test

use chrono::{Local, TimeZone};
use locksun_lib::{config, gemini, sun::SunCalculator};
use std::{fs, path::Path};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // 環境変数から API キーを読み取る
    let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_else(|_| {
        eprintln!("警告: GEMINI_API_KEY が未設定です。Gemini API 呼び出しはスキップされます。");
        String::new()
    });

    // ベース画像を生成
    let cfg = config::AppConfig {
        location: config::LocationConfig {
            name: "東京".to_string(),
            latitude: 35.6762,
            longitude: 139.6503,
        },
        update: config::UpdateConfig { interval_secs: 300 },
        image: config::ImageConfig {
            width: 1920,
            height: 1080,
            show_stars: false,
            show_clouds: false,
            water_depth: 0.7,
        },
        behavior: config::BehaviorConfig { autostart: false },
        gemini: config::GeminiConfig {
            api_key: api_key.clone(),
            model_name: "gemini-2.5-flash-image".to_string(),
            enhance_prompt: "Enhance this procedurally generated sky image to look photorealistic. Preserve the overall sky color and sun position. Add natural cloud textures, subtle atmospheric haze, and photographic quality grain.".to_string(),
            enabled: !api_key.is_empty(),
        },
    };

    // テスト再現性のため時間を固定 (東京 JST 2026-03-09 12:00 正午)
    let now = chrono::FixedOffset::east_opt(9 * 3600)
        .unwrap()
        .with_ymd_and_hms(2026, 3, 9, 12, 0, 0)
        .unwrap()
        .with_timezone(&Local);
    let pos = SunCalculator::position(&now, cfg.location.latitude, cfg.location.longitude);
    println!("固定時刻: {}", now.format("%Y-%m-%d %H:%M:%S JST"));
    println!(
        "太陽位置: 高度={:.1}°, 方位角={:.1}°",
        pos.altitude, pos.azimuth
    );

    // ベース画像（PNG バイト列）を生成
    println!("\n[1/3] ベース画像を生成中...");
    let img = locksun_lib::renderer::composer::compose(&pos, &cfg.image)?;
    let base_path = Path::new("test_gemini_base.png");
    img.save(base_path)?;
    println!("✅ ベース画像: {}", base_path.display());

    // PNG バイト列に変換
    let mut png_bytes: Vec<u8> = Vec::new();
    {
        use image::ImageEncoder;
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        encoder.write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            image::ExtendedColorType::Rgb8,
        )?;
    }
    println!("   PNG サイズ: {:.1} KB", png_bytes.len() as f64 / 1024.0);

    if cfg.gemini.api_key.is_empty() {
        println!("\n⚠️  GEMINI_API_KEY が未設定のため Gemini API 呼び出しをスキップします。");
        println!("   $env:GEMINI_API_KEY = \"AIza...\" を設定して再実行してください。");
        return Ok(());
    }

    // Gemini API で強化
    println!(
        "\n[2/3] Gemini API で画像を強化中... (モデル: {})",
        cfg.gemini.model_name
    );
    println!("   プロンプト: \"{}\"", cfg.gemini.enhance_prompt);
    println!(
        "   太陽フェーズ: {:?} (altitude: {:.1}°)",
        locksun_lib::sun::SunPhase::from_altitude(pos.altitude),
        pos.altitude
    );

    match gemini::enhance_image(&cfg.gemini, &pos, &png_bytes).await {
        Ok(enhanced_bytes) => {
            println!(
                "✅ Gemini API 成功: {:.1} KB",
                enhanced_bytes.len() as f64 / 1024.0
            );

            // 強化後の画像を保存
            println!("\n[3/3] 強化画像を保存中...");
            let out_path = Path::new("test_gemini_enhanced.png");
            fs::write(out_path, &enhanced_bytes)?;
            println!("✅ 強化画像: {}", out_path.display());

            println!("\n--- 比較 ---");
            println!("  ベース  : {}", base_path.display());
            println!("  強化後  : {}", out_path.display());
        }
        Err(e) => {
            eprintln!("\n❌ Gemini API エラー: {}", e);
            eprintln!("   よくある原因:");
            eprintln!("     - API キーが無効");
            eprintln!("     - 指定モデルが画像生成非対応 (IMAGE モダリティを確認)");
            eprintln!("     - レート制限");
            return Err(e.into());
        }
    }

    Ok(())
}
