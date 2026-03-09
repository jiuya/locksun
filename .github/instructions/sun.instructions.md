---
applyTo: "src-tauri/src/sun/**"
---

# 太陽位置計算モジュール (sun/)

## 役割

緯度・経度・日時から太陽の高度角・方位角・日の出/日の入り時刻を計算する。

## アルゴリズム

**NOAA 簡易式（Spencer 1971 / Greve 1978）** を使用。

```
B = 2π × (dayOfYear - 1) / 365
均時差 (分): EOT = 229.18 × (0.000075 + ...)
赤緯 (rad): δ = 0.006918 - 0.399912cos(B) + ...
時角 (度):  HA = (太陽時刻 / 4) - 180
高度角:     sin(alt) = sin(lat)sin(δ) + cos(lat)cos(δ)cos(HA)
```

日の出・日の入りは**二分探索（32回反復）** で求める。精度: ±0.2秒。

## 型定義の約束

```rust
// SunPosition: 常に (altitude: f64, azimuth: f64) のペア
// altitude: -90.0 〜 +90.0 度（正=地平線より上）
// azimuth:  0.0=北, 90.0=東, 180.0=南, 270.0=西

// SunTimes: 各フィールドは Option<DateTime<Local>>
// 白夜・極夜では sunrise/sunset が None になる

// SunPhase: altitude から from_altitude() で生成する
```

## 禁止事項

- `chrono::Utc::now()` の使用禁止 → `Local::now()` を使う
- `f64::NAN` や `f64::INFINITY` を返さない → `clamp()` で範囲内に収める
- 三角関数の引数は必ずラジアンで渡す（変換忘れに注意）

## テスト要件

新しい計算関数を追加したら必ず以下を確認するテストを書くこと：

- 東京(35.68°N, 139.69°E) の日の出が 6時台、日の入りが 17-18時台
- 正午の高度角が正値
- 夜中0時の `SunPhase` が `AstronomicalNight`
