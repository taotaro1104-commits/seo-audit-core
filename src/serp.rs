// 検索結果スニペットのピクセル幅推定と省略シミュレーション
// ※Googleの表示幅は非公開のため、下記定数は実測ベースの参考値
use serde::Serialize;

struct Layout {
    font_px: f64,
    max_px: f64,
}

const TITLE_PC: Layout = Layout { font_px: 20.0, max_px: 600.0 };
const TITLE_MOBILE: Layout = Layout { font_px: 18.0, max_px: 340.0 * 2.0 };
const DESC_PC: Layout = Layout { font_px: 14.0, max_px: 652.0 * 2.0 };
const DESC_MOBILE: Layout = Layout { font_px: 14.0, max_px: 340.0 * 3.0 };

const ELLIPSIS: &str = "...";
/// 表に無い文字の幅（em）
const DEFAULT_EM: f64 = 0.6;

/// Arial の ASCII(0x20〜0x7E) 文字幅（1/1000 em）
const ARIAL_ASCII: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, // ' '〜'/'
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556,                               // 0〜9
    278, 278, 584, 584, 584, 556, 1015,                                             // :〜@
    667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833,                // A〜M
    722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611,                // N〜Z
    278, 278, 278, 469, 556, 333,                                                   // [〜`
    556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833,                // a〜m
    556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500,                // n〜z
    334, 260, 334, 584,                                                             // {〜~
];

#[derive(Serialize)]
pub struct Fit {
    pub px: f64,
    pub max_px: f64,
    pub remaining_px: f64,
    pub truncated: bool,
    pub chars_fit: usize,
    pub display: String,
}

#[derive(Serialize)]
pub struct Field {
    pub text: String,
    pub chars: usize,
    pub pc: Fit,
    pub mobile: Fit,
}

#[derive(Serialize)]
pub struct SerpPreview {
    pub title: Field,
    pub description: Field,
}

pub fn preview(title: &str, description: &str) -> SerpPreview {
    SerpPreview {
        title: field(title, &TITLE_PC, &TITLE_MOBILE),
        description: field(description, &DESC_PC, &DESC_MOBILE),
    }
}

fn field(raw: &str, pc: &Layout, mobile: &Layout) -> Field {
    let text = collapse_whitespace(raw);
    Field {
        chars: text.chars().count(),
        pc: fit(&text, pc),
        mobile: fit(&text, mobile),
        text,
    }
}

fn collapse_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn fit(text: &str, layout: &Layout) -> Fit {
    let px = round1(text_width_px(text, layout.font_px));
    let total_chars = text.chars().count();
    if px <= layout.max_px {
        return Fit {
            px,
            max_px: layout.max_px,
            remaining_px: round1(layout.max_px - px),
            truncated: false,
            chars_fit: total_chars,
            display: text.to_string(),
        };
    }

    let budget = layout.max_px - text_width_px(ELLIPSIS, layout.font_px);
    let mut used = 0.0;
    let mut kept = String::new();
    for c in text.chars() {
        let w = char_width_px(c, layout.font_px);
        if used + w > budget {
            break;
        }
        used += w;
        kept.push(c);
    }
    let chars_fit = kept.chars().count();
    Fit {
        px,
        max_px: layout.max_px,
        remaining_px: round1(layout.max_px - px),
        truncated: true,
        chars_fit,
        display: format!("{}{}", kept.trim_end(), ELLIPSIS),
    }
}

/// 文字列の推定表示幅（px）
pub fn text_width_px(text: &str, font_px: f64) -> f64 {
    text.chars().map(|c| char_width_px(c, font_px)).sum()
}

fn char_width_px(c: char, font_px: f64) -> f64 {
    char_em(c) * font_px
}

fn char_em(c: char) -> f64 {
    let code = c as u32;
    match code {
        0x20..=0x7E => ARIAL_ASCII[(code - 0x20) as usize] as f64 / 1000.0,
        0xFF61..=0xFF9F => 0.5,                    // 半角カナ
        0x1100..=0x11FF                             // ハングル字母
        | 0x2E80..=0x9FFF                           // CJK記号・かな・漢字
        | 0xAC00..=0xD7AF                           // ハングル
        | 0xF900..=0xFAFF                           // CJK互換漢字
        | 0xFE30..=0xFE4F
        | 0xFF00..=0xFF60                           // 全角英数・記号
        | 0xFFE0..=0xFFE6
        | 0x1F300..=0x1FAFF                         // 絵文字
        | 0x20000..=0x3FFFF => 1.0,
        _ => DEFAULT_EM,
    }
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}
