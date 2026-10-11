//! seo-audit-core — SEO診断の純粋ロジック（SERP表示幅・robots.txt・sitemap・インデックス可否）
//!
//! I/Oを持たない関数だけで構成されており、入力・出力はJSON文字列。
//! Web版（https://seo.matchinfo-rank.com/）ではこれと同じロジックをWASMとしてブラウザ内で実行している。

mod indexability;
mod robots;
mod serp;
mod sitemap;
mod util;

use util::to_json;

pub use serp::text_width_px;

/// タイトル・ディスクリプションの検索結果での表示幅と省略位置を推定する（JSON文字列）
pub fn serp_preview(title: &str, description: &str) -> String {
    to_json(&serp::preview(title, description))
}

/// robots.txt で URL がクロール可能か判定する（JSON文字列）
/// status_code は robots.txt 取得時のHTTPステータス（接続失敗は 0）
pub fn check_robots(robots_txt: &str, status_code: u16, url: &str, user_agent: &str) -> String {
    to_json(&robots::check(robots_txt, status_code, url, user_agent))
}

/// sitemap.xml を解析し、対象URLの掲載有無を返す（JSON文字列）
pub fn parse_sitemap(xml: &str, target_url: &str) -> String {
    to_json(&sitemap::parse(xml, target_url))
}

/// インデックス可否を総合判定する。入力・出力ともJSON文字列
pub fn check_indexability(input_json: &str) -> String {
    match serde_json::from_str::<indexability::Input>(input_json) {
        Ok(input) => to_json(&indexability::evaluate(&input)),
        Err(e) => to_json(&serde_json::json!({ "error": format!("入力JSONが不正です: {e}") })),
    }
}
