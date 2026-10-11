// sitemap.xml の簡易解析（urlset / sitemapindex）
// XMLパーサーを入れるとWASMが肥大化するため、<loc>/<lastmod> の文字列走査で処理する
use crate::util::normalize_url;
use serde::Serialize;

const MAX_CHILD_SITEMAPS: usize = 20;

#[derive(Serialize)]
pub struct SitemapResult {
    /// "urlset" | "sitemapindex" | "invalid"
    pub kind: &'static str,
    pub url_count: usize,
    pub latest_lastmod: Option<String>,
    /// urlset の場合のみ判定。sitemapindex/invalid は null
    pub contains_target: Option<bool>,
    pub child_sitemaps: Vec<String>,
}

pub fn parse(xml: &str, target_url: &str) -> SitemapResult {
    let kind = if xml.contains("<sitemapindex") {
        "sitemapindex"
    } else if xml.contains("<urlset") {
        "urlset"
    } else {
        "invalid"
    };

    let locs = tag_values(xml, "loc");
    let latest_lastmod = tag_values(xml, "lastmod").into_iter().max();

    let contains_target = (kind == "urlset").then(|| {
        let target = normalize_url(target_url);
        target.is_some() && locs.iter().any(|l| normalize_url(l) == target)
    });
    let child_sitemaps = if kind == "sitemapindex" {
        locs.iter().take(MAX_CHILD_SITEMAPS).cloned().collect()
    } else {
        Vec::new()
    };

    SitemapResult {
        kind,
        url_count: if kind == "invalid" { 0 } else { locs.len() },
        latest_lastmod,
        contains_target,
        child_sitemaps,
    }
}

/// <tag>値</tag> の値を順に取り出す（CDATA・前後空白・基本エンティティに対応）
fn tag_values(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut rest = xml;

    while let Some(start) = rest.find(&open) {
        let after = &rest[start + open.len()..];
        let Some(end) = after.find(&close) else { break };
        let value = unwrap_cdata(after[..end].trim());
        if !value.is_empty() {
            out.push(decode_entities(value));
        }
        rest = &after[end + close.len()..];
    }
    out
}

fn unwrap_cdata(s: &str) -> &str {
    s.strip_prefix("<![CDATA[")
        .and_then(|x| x.strip_suffix("]]>"))
        .map(str::trim)
        .unwrap_or(s)
}

fn decode_entities(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
