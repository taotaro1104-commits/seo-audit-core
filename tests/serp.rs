// ②SERPプレビュー（ピクセル幅推定・省略位置）のテスト
use seo_audit_core::{serp_preview, text_width_px};
use serde_json::Value;

fn preview(title: &str, desc: &str) -> Value {
    serde_json::from_str(&serp_preview(title, desc)).unwrap()
}

#[test]
fn fullwidth_chars_are_one_em() {
    assert_eq!(text_width_px("あいう", 20.0), 60.0);
    assert_eq!(text_width_px("ＡＢ", 10.0), 20.0);
}

#[test]
fn ascii_uses_proportional_widths() {
    let narrow = text_width_px("iiii", 20.0);
    let wide = text_width_px("WWWW", 20.0);
    assert!(narrow < 20.0, "i は細い: {narrow}");
    assert!(wide > 70.0, "W は太い: {wide}");
    // Arial の 'a' = 0.556em
    assert!((text_width_px("a", 1000.0) - 556.0).abs() < 0.01);
}

#[test]
fn halfwidth_katakana_is_half_em() {
    assert_eq!(text_width_px("ｱｲ", 20.0), 20.0);
}

#[test]
fn title_of_30_fullwidth_chars_fits_pc() {
    let t = "あ".repeat(30);
    let v = preview(&t, "");
    assert_eq!(v["title"]["pc"]["truncated"], false);
    assert_eq!(v["title"]["pc"]["px"], 600.0);
    assert_eq!(v["title"]["pc"]["display"], t.as_str());
    assert_eq!(v["title"]["chars"], 30);
}

#[test]
fn long_title_is_truncated_with_ellipsis() {
    let t = "あ".repeat(40);
    let v = preview(&t, "");
    let pc = &v["title"]["pc"];
    assert_eq!(pc["truncated"], true);
    let display = pc["display"].as_str().unwrap();
    assert!(display.ends_with("..."));
    let fit = pc["chars_fit"].as_u64().unwrap();
    assert!(fit < 30 && fit >= 27, "fit={fit}");
    assert!(pc["remaining_px"].as_f64().unwrap() < 0.0);
}

#[test]
fn mobile_allows_longer_title_than_pc() {
    let t = "あ".repeat(34);
    let v = preview(&t, "");
    assert_eq!(v["title"]["pc"]["truncated"], true);
    assert_eq!(v["title"]["mobile"]["truncated"], false);
}

#[test]
fn description_limits_and_empty_input() {
    let v = preview("", &"い".repeat(200));
    assert_eq!(v["description"]["pc"]["truncated"], true);
    assert_eq!(v["description"]["mobile"]["truncated"], true);
    assert_eq!(v["title"]["pc"]["px"], 0.0);
    assert_eq!(v["title"]["pc"]["truncated"], false);
    assert_eq!(v["title"]["pc"]["display"], "");
    let short = preview("", "短い説明");
    assert_eq!(short["description"]["pc"]["truncated"], false);
}

#[test]
fn whitespace_is_collapsed() {
    let v = preview("  A \n  B  ", "");
    assert_eq!(v["title"]["pc"]["display"], "A B");
}
