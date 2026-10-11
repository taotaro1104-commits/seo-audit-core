// インデックス可否の総合判定（ステータス・robots.txt・noindex・canonical）
use crate::util::{loose_url_key, normalize_url, Lang};
use serde::{Deserialize, Serialize};
use url::Url;

/// X-Robots-Tag で「bot名:」ではなくディレクティブとして扱う、値にコロンを含むもの
const COLON_DIRECTIVES: &[&str] = &[
    "unavailable_after", "max-snippet", "max-image-preview", "max-video-preview",
];
/// 判定対象のクローラー
const TARGET_BOT: &str = "googlebot";

#[derive(Deserialize)]
pub struct Input {
    pub requested_url: String,
    pub final_url: Option<String>,
    pub status_code: u16,
    pub meta_robots: Option<String>,
    pub x_robots_tag: Option<String>,
    pub canonical: Option<String>,
    /// robots.txt 未取得なら null
    pub robots_allowed: Option<bool>,
    /// sitemap 未取得・判定不能なら null
    pub in_sitemap: Option<bool>,
    #[serde(default)]
    pub lang: Lang,
}

#[derive(Serialize)]
pub struct Finding {
    /// "err" | "warn" | "info" | "ok"
    pub level: &'static str,
    pub message: String,
}

#[derive(Serialize)]
pub struct Indexability {
    pub indexable: bool,
    pub noindex: bool,
    pub nofollow: bool,
    pub noindex_sources: Vec<&'static str>,
    /// "self" | "variant" | "other" | "missing" | "invalid"
    pub canonical_state: &'static str,
    pub canonical_resolved: Option<String>,
    pub redirected: bool,
    pub findings: Vec<Finding>,
}

#[derive(Default)]
struct Directives {
    noindex: bool,
    nofollow: bool,
}

pub fn evaluate(input: &Input) -> Indexability {
    let page_url = input.final_url.clone().unwrap_or_else(|| input.requested_url.clone());
    let meta = parse_meta_robots(input.meta_robots.as_deref().unwrap_or(""));
    let header = parse_x_robots_tag(input.x_robots_tag.as_deref().unwrap_or(""));

    let mut noindex_sources = Vec::new();
    if meta.noindex { noindex_sources.push("meta"); }
    if header.noindex { noindex_sources.push("x-robots-tag"); }
    let noindex = meta.noindex || header.noindex;
    let nofollow = meta.nofollow || header.nofollow;

    let redirected = normalize_url(&page_url) != normalize_url(&input.requested_url);
    let (canonical_state, canonical_resolved) = canonical_state(input.canonical.as_deref(), &page_url);
    let status_ok = (200..300).contains(&input.status_code);
    let robots_blocked = input.robots_allowed == Some(false);

    let lang = input.lang;
    let mut f = Vec::new();
    let mut push = |level, ja: String, en: String, ko: String, zh_cn: String, zh_tw: String| {
        f.push(Finding { level, message: lang.pick(ja, en, ko, zh_cn, zh_tw) });
    };

    if !status_ok {
        let s = input.status_code;
        push("err", format!("HTTPステータスが {s} のためインデックスされません"),
            format!("The page returns HTTP {s}, so it will not be indexed"),
            format!("페이지가 HTTP {s}를 반환하여 색인되지 않습니다"),
            format!("页面返回 HTTP {s}，因此无法编入索引"),
            format!("頁面回傳 HTTP {s}，因此無法建立索引"));
    }
    if robots_blocked {
        push("err", "robots.txt でクロールがブロックされています（Googlebot）".into(),
            "Crawling is blocked by robots.txt (Googlebot)".into(),
            "robots.txt에서 Googlebot 크롤링을 차단하고 있습니다".into(),
            "robots.txt 阻止了 Googlebot 抓取".into(),
            "robots.txt 封鎖了 Googlebot 檢索".into());
    }
    if noindex {
        let src = noindex_sources.join(" / ");
        push("err", format!("noindex が指定されています（{src}）"), format!("noindex is set ({src})"),
            format!("noindex가 설정되어 있습니다 ({src})"), format!("已设置 noindex（{src}）"), format!("已設定 noindex（{src}）"));
    }
    if nofollow {
        push("warn", "nofollow が指定されており、ページ内リンクがたどられません".into(),
            "nofollow is set, so links on the page will not be followed".into(),
            "nofollow가 설정되어 페이지 내 링크를 따라가지 않습니다".into(),
            "已设置 nofollow，页面中的链接不会被跟踪".into(),
            "已設定 nofollow，頁面中的連結不會被追蹤".into());
    }
    if redirected {
        push("info", format!("リダイレクトされています → {page_url}"), format!("The URL redirects to {page_url}"),
            format!("URL이 다음 주소로 리디렉션됩니다 → {page_url}"), format!("URL 重定向到 {page_url}"), format!("URL 重新導向至 {page_url}"));
    }
    match canonical_state {
        "missing" => push("warn", "canonical が未設定です".into(), "No canonical tag is set".into(), "canonical 태그가 설정되지 않았습니다".into(), "未设置 canonical 标签".into(), "未設定 canonical 標籤".into()),
        "invalid" => push("warn", "canonical のURLが不正です".into(), "The canonical URL is invalid".into(), "canonical URL이 올바르지 않습니다".into(), "canonical URL 无效".into(), "canonical URL 無效".into()),
        "variant" => push("warn", "canonical がこのURLの表記ゆれ（http/www/末尾スラッシュ）を指しています".into(),
            "Canonical points to a variant of this URL (http/www/trailing slash)".into(),
            "canonical이 http/www/끝 슬래시 표기만 다른 URL을 가리킵니다".into(),
            "canonical 指向此 URL 的另一种写法（http/www/末尾斜杠）".into(),
            "canonical 指向此 URL 的另一種寫法（http/www/結尾斜線）".into()),
        "other" => push("warn", "canonical が別URLを指しているため、このURLは検索結果に出ない可能性があります".into(),
            "Canonical points to a different URL, so this URL may not appear in search results".into(),
            "canonical이 다른 URL을 가리켜 이 URL이 검색 결과에 표시되지 않을 수 있습니다".into(),
            "canonical 指向其他 URL，因此此 URL 可能不会出现在搜索结果中".into(),
            "canonical 指向其他 URL，因此此 URL 可能不會出現在搜尋結果中".into()),
        _ => {}
    }
    match input.in_sitemap {
        Some(false) => push("info", "sitemap.xml にこのURLが含まれていません".into(), "This URL is not listed in sitemap.xml".into(), "sitemap.xml에 이 URL이 없습니다".into(), "sitemap.xml 中未列出此 URL".into(), "sitemap.xml 中未列出此 URL".into()),
        Some(true) => push("ok", "sitemap.xml に掲載されています".into(), "Listed in sitemap.xml".into(), "sitemap.xml에 포함되어 있습니다".into(), "已列在 sitemap.xml 中".into(), "已列在 sitemap.xml 中".into()),
        None => {}
    }

    let indexable = status_ok && !robots_blocked && !noindex;
    if indexable {
        push("ok", "インデックス可能な状態です".into(), "The page can be indexed".into(), "색인할 수 있는 상태입니다".into(), "页面可以编入索引".into(), "頁面可以建立索引".into());
    }

    Indexability {
        indexable,
        noindex,
        nofollow,
        noindex_sources,
        canonical_state,
        canonical_resolved,
        redirected,
        findings: f,
    }
}

fn apply_directive(d: &mut Directives, token: &str) {
    match token.trim().to_lowercase().as_str() {
        "noindex" => d.noindex = true,
        "nofollow" => d.nofollow = true,
        "none" => { d.noindex = true; d.nofollow = true; }
        _ => {}
    }
}

fn parse_meta_robots(content: &str) -> Directives {
    let mut d = Directives::default();
    content.split(',').for_each(|t| apply_directive(&mut d, t));
    d
}

/// "googlebot: noindex, otherbot: nofollow" のような bot 指定付きの値に対応
fn parse_x_robots_tag(value: &str) -> Directives {
    let mut d = Directives::default();
    let mut current_bot: Option<String> = None;

    for part in value.split(',') {
        let mut token = part.trim();
        if let Some((left, right)) = token.split_once(':') {
            let left_lc = left.trim().to_lowercase();
            if !COLON_DIRECTIVES.contains(&left_lc.as_str()) {
                current_bot = Some(left_lc);
                token = right.trim();
            }
        }
        let applies = current_bot.as_deref().map(|b| b == TARGET_BOT).unwrap_or(true);
        if applies {
            apply_directive(&mut d, token);
        }
    }
    d
}

fn canonical_state(canonical: Option<&str>, page_url: &str) -> (&'static str, Option<String>) {
    let Some(raw) = canonical.map(str::trim).filter(|s| !s.is_empty()) else {
        return ("missing", None);
    };
    let resolved = Url::parse(page_url).ok()
        .and_then(|base| base.join(raw).ok())
        .map(|u| u.to_string());
    let Some(resolved) = resolved else { return ("invalid", None) };

    let state = if normalize_url(&resolved) == normalize_url(page_url) {
        "self"
    } else if loose_url_key(&resolved).is_some() && loose_url_key(&resolved) == loose_url_key(page_url) {
        "variant"
    } else {
        "other"
    };
    (state, Some(resolved))
}
