// 共通ユーティリティ
use url::Url;

pub fn to_json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string())
}

/// 比較用にURLを正規化する（フラグメント除去。解析不能なら None）
pub fn normalize_url(raw: &str) -> Option<String> {
    let mut u = Url::parse(raw.trim()).ok()?;
    u.set_fragment(None);
    Some(u.to_string())
}

/// スキーム・www・末尾スラッシュの違いを無視した比較キー
pub fn loose_url_key(raw: &str) -> Option<String> {
    let u = Url::parse(raw.trim()).ok()?;
    let host = u.host_str()?.trim_start_matches("www.").to_string();
    let path = u.path().trim_end_matches('/').to_string();
    let query = u.query().map(|q| format!("?{q}")).unwrap_or_default();
    Some(format!("{host}{path}{query}"))
}

/// 出力メッセージの言語（未知の値は日本語）
#[derive(serde::Deserialize, Default, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    En,
    Ko,
    #[serde(rename = "zh-cn")]
    ZhCn,
    #[serde(rename = "zh-tw")]
    ZhTw,
    #[default]
    #[serde(other)]
    Ja,
}

impl Lang {
    pub fn pick<T>(self, ja: T, en: T, ko: T, zh_cn: T, zh_tw: T) -> T {
        match self {
            Lang::Ja => ja,
            Lang::En => en,
            Lang::ZhCn => zh_cn,
            Lang::ZhTw => zh_tw,
            Lang::Ko => ko,
        }
    }
}
