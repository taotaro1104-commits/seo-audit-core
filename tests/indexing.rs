// ③robots.txt / sitemap / インデックス可否のテスト
use seo_audit_core::{check_indexability, check_robots, parse_sitemap};
use serde_json::{json, Value};

fn robots(txt: &str, status: u16, url: &str, ua: &str) -> Value {
    serde_json::from_str(&check_robots(txt, status, url, ua)).unwrap()
}

// ===== robots.txt =====

const ROBOTS: &str = "
# comment
User-agent: *
Disallow: /private/
Allow: /private/public/
Disallow: /*.pdf$
Disallow: /search?q=*

User-agent: Googlebot
User-agent: Bingbot
Disallow: /no-google/

User-agent: Googlebot-Image
Disallow: /

Sitemap: https://example.com/sitemap.xml
sitemap: https://example.com/news.xml
";

#[test]
fn robots_star_group_rules() {
    let r = |p: &str| robots(ROBOTS, 200, &format!("https://example.com{p}"), "SomeBot")["allowed"].clone();
    assert_eq!(r("/"), true);
    assert_eq!(r("/private/x"), false);
    assert_eq!(r("/private/public/x"), true); // より長い Allow が優先
    assert_eq!(r("/files/a.pdf"), false);
    assert_eq!(r("/files/a.pdf?x=1"), true);  // $ は末尾一致
    assert_eq!(r("/search?q=seo"), false);
}

#[test]
fn robots_specific_group_replaces_star() {
    let g = |p: &str| robots(ROBOTS, 200, &format!("https://example.com{p}"), "Googlebot");
    let v = g("/private/x");
    assert_eq!(v["allowed"], true); // Googlebot グループには /private/ が無い
    assert_eq!(v["group"], "googlebot");
    assert_eq!(g("/no-google/a")["allowed"], false);
    assert_eq!(robots(ROBOTS, 200, "https://example.com/no-google/", "bingbot")["allowed"], false);
}

#[test]
fn robots_most_specific_user_agent_wins() {
    let v = robots(ROBOTS, 200, "https://example.com/a.png", "Googlebot-Image");
    assert_eq!(v["group"], "googlebot-image");
    assert_eq!(v["allowed"], false);
    // 未定義のサブ種別はプレフィックス一致で googlebot グループ
    let v = robots(ROBOTS, 200, "https://example.com/private/x", "Googlebot-News");
    assert_eq!(v["group"], "googlebot");
}

#[test]
fn robots_matched_rule_and_sitemaps() {
    let v = robots(ROBOTS, 200, "https://example.com/private/x", "*");
    assert_eq!(v["matched_rule"]["kind"], "disallow");
    assert_eq!(v["matched_rule"]["pattern"], "/private/");
    assert_eq!(v["sitemaps"], json!(["https://example.com/sitemap.xml", "https://example.com/news.xml"]));
}

#[test]
fn robots_tie_prefers_allow_and_empty_disallow_allows() {
    let txt = "User-agent: *\nDisallow: /page\nAllow: /page\n";
    assert_eq!(robots(txt, 200, "https://example.com/page", "x")["allowed"], true);
    let txt = "User-agent: *\nDisallow:\n";
    assert_eq!(robots(txt, 200, "https://example.com/anything", "x")["allowed"], true);
}

#[test]
fn robots_status_handling() {
    let v = robots("", 404, "https://example.com/x", "Googlebot");
    assert_eq!(v["allowed"], true);
    assert_eq!(v["state"], "not_found");
    let v = robots("", 503, "https://example.com/x", "Googlebot");
    assert_eq!(v["allowed"], false);
    assert_eq!(v["state"], "server_error");
    let v = robots("User-agent: *\nDisallow: /\n", 200, "https://example.com/robots.txt", "x");
    assert_eq!(v["allowed"], true); // robots.txt 自体は常に取得可
}

// ===== sitemap =====

#[test]
fn sitemap_urlset() {
    let xml = r#"<?xml version="1.0"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
      <url><loc>https://example.com/</loc><lastmod>2026-01-01</lastmod></url>
      <url><loc> https://example.com/a?x=1&amp;y=2 </loc><lastmod>2026-03-05T10:00:00+09:00</lastmod></url>
      <url><loc><![CDATA[https://example.com/b/]]></loc></url>
    </urlset>"#;
    let v: Value = serde_json::from_str(&parse_sitemap(xml, "https://example.com/a?x=1&y=2")).unwrap();
    assert_eq!(v["kind"], "urlset");
    assert_eq!(v["url_count"], 3);
    assert_eq!(v["latest_lastmod"], "2026-03-05T10:00:00+09:00");
    assert_eq!(v["contains_target"], true);
    let v: Value = serde_json::from_str(&parse_sitemap(xml, "https://example.com/b")).unwrap();
    assert_eq!(v["contains_target"], false);
}

#[test]
fn sitemap_index_and_invalid() {
    let xml = "<sitemapindex><sitemap><loc>https://example.com/s1.xml</loc></sitemap></sitemapindex>";
    let v: Value = serde_json::from_str(&parse_sitemap(xml, "https://example.com/")).unwrap();
    assert_eq!(v["kind"], "sitemapindex");
    assert_eq!(v["child_sitemaps"], json!(["https://example.com/s1.xml"]));
    assert_eq!(v["contains_target"], Value::Null);
    let v: Value = serde_json::from_str(&parse_sitemap("<html>404</html>", "https://example.com/")).unwrap();
    assert_eq!(v["kind"], "invalid");
}

// ===== インデックス可否 =====

fn idx(input: Value) -> Value {
    serde_json::from_str(&check_indexability(&input.to_string())).unwrap()
}

fn base() -> Value {
    json!({
        "requested_url": "https://example.com/page",
        "final_url": "https://example.com/page",
        "status_code": 200,
        "meta_robots": "index,follow",
        "x_robots_tag": null,
        "canonical": "https://example.com/page",
        "robots_allowed": true,
        "in_sitemap": true
    })
}

fn with(mut v: Value, k: &str, x: Value) -> Value {
    v[k] = x;
    v
}

#[test]
fn indexable_page() {
    let v = idx(base());
    assert_eq!(v["indexable"], true);
    assert_eq!(v["canonical_state"], "self");
    assert_eq!(v["noindex"], false);
}

#[test]
fn noindex_from_meta_or_header() {
    let v = idx(with(base(), "meta_robots", json!("NOINDEX, follow")));
    assert_eq!(v["indexable"], false);
    assert_eq!(v["noindex_sources"], json!(["meta"]));
    let v = idx(with(base(), "x_robots_tag", json!("googlebot: noindex, otherbot: nofollow")));
    assert_eq!(v["noindex"], true);
    assert_eq!(v["nofollow"], false);
    assert_eq!(v["noindex_sources"], json!(["x-robots-tag"]));
    let v = idx(with(base(), "x_robots_tag", json!("otherbot: noindex")));
    assert_eq!(v["noindex"], false);
    let v = idx(with(base(), "x_robots_tag", json!("unavailable_after: 2030-01-01, none")));
    assert_eq!(v["noindex"], true);
    assert_eq!(v["nofollow"], true);
}

#[test]
fn canonical_states() {
    let s = |c: Value| idx(with(base(), "canonical", c))["canonical_state"].clone();
    assert_eq!(s(json!("/page")), "self");
    assert_eq!(s(json!("https://example.com/page#top")), "self");
    assert_eq!(s(json!("https://example.com/page/")), "variant");
    assert_eq!(s(json!("http://www.example.com/page")), "variant");
    assert_eq!(s(json!("https://example.com/other")), "other");
    assert_eq!(s(json!(null)), "missing");
    assert_eq!(s(json!("")), "missing");
    let v = idx(with(base(), "canonical", json!("https://example.com/other")));
    assert_eq!(v["indexable"], true); // canonical 違いは警告のみ
    assert_eq!(v["canonical_resolved"], "https://example.com/other");
}

#[test]
fn status_redirect_and_robots_block() {
    let v = idx(with(base(), "status_code", json!(404)));
    assert_eq!(v["indexable"], false);
    let v = idx(with(base(), "final_url", json!("https://example.com/moved")));
    assert_eq!(v["redirected"], true);
    let v = idx(with(base(), "robots_allowed", json!(false)));
    assert_eq!(v["indexable"], false);
    let levels: Vec<String> = v["findings"].as_array().unwrap().iter()
        .map(|f| f["level"].as_str().unwrap().to_string()).collect();
    assert!(levels.contains(&"err".to_string()));
}

#[test]
fn invalid_input_returns_error_json() {
    let v: Value = serde_json::from_str(&check_indexability("not json")).unwrap();
    assert!(v["error"].is_string());
}
