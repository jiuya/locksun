---
applyTo: "tests/**,src-tauri/src/**/tests,src-tauri/src/**/#[cfg(test)]"
---

# テスト方針 (tests/)

## テストの種類

| 種類           | 場所                                        | 用途                       |
| -------------- | ------------------------------------------- | -------------------------- |
| ユニットテスト | 各モジュール末尾の `#[cfg(test)] mod tests` | 関数単位の動作確認         |
| 統合テスト     | `tests/*.rs`                                | モジュールをまたぐ動作確認 |

## テストデータの基準都市

| 都市 | 緯度    | 経度     | 用途               |
| ---- | ------- | -------- | ------------------ |
| 東京 | 35.68°N | 139.69°E | 主要テストケース   |
| 札幌 | 43.06°N | 141.35°E | 高緯度・冬季テスト |
| 那覇 | 26.21°N | 127.68°E | 低緯度テスト       |

## 太陽計算テストの許容誤差

- 日の出・日の入り: **±10分** 以内
- 高度角: **±2度** 以内
- `SunPhase` の正確な値: 基準日時で厳密に一致すること

## テストの書き方

```rust
// ✅ 良い例: 実際の日時・都市で検証
#[test]
fn test_tokyo_sunrise_2026_03_09() {
    let dt = jst(2026, 3, 9, 12, 0);  // 東京・春分前後
    let times = SunCalculator::times(&dt, 35.68, 139.69);
    let sr = times.sunrise.unwrap();
    // ±10分の許容誤差
    assert!(sr.hour() == 6, "日の出が6時台であること: {sr}");
}

// ❌ 悪い例: 魔法数字・検証なし
#[test]
fn test_calc() {
    let r = SunCalculator::position(&Local::now(), 0.0, 0.0);
    assert!(r.altitude != 999.0); // 意味のないアサーション
}
```

## Windows API テストの扱い

- `lockscreen` モジュールのテストは `#[cfg(target_os = "windows")]` でラップ
- CI（非 Windows 環境）でのスキップを意図的に許容する
- 実機テストの結果は PR のコメントに記録する

## ヘルパー関数

統合テストで共通利用する JST 変換関数:

```rust
fn jst(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Local> {
    chrono::FixedOffset::east_opt(9 * 3600)
        .unwrap()
        .with_ymd_and_hms(y, mo, d, h, mi, 0)
        .unwrap()
        .with_timezone(&Local)
}
```
