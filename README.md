# seo-audit-core

[![CI](https://github.com/taotaro1104-commits/seo-audit-core/actions/workflows/ci.yml/badge.svg)](https://github.com/taotaro1104-commits/seo-audit-core/actions)
![license](https://img.shields.io/badge/license-MIT-blue)

日本語 | [English](README.en.md)

SEO診断の**純粋ロジック**だけを切り出した Rust クレートです。I/Oを持たず、入力・出力はJSON文字列。WASMにそのままコンパイルできます。

- **SERPプレビュー** — title / description が検索結果で何pxになり、どこで省略されるか（日本語の全角/半角混在に対応）
- **robots.txt 判定** — User-agent グループ選択、Allow/Disallow の最長一致、`*` / `$`、HTTPステータス別の扱い
- **sitemap.xml 判定** — urlset / sitemapindex の解析と対象URLの掲載有無
- **インデックス可否の総合判定** — ステータス・リダイレクト・robots.txt・meta robots・X-Robots-Tag・canonical・sitemap の矛盾を1つの判定にまとめる（5言語メッセージ）

Web版: 同じロジックをブラウザ内で動かしている無料ツール → <https://seo.matchinfo-rank.com/>

## 使い方

```toml
[dependencies]
seo-audit-core = { git = "https://github.com/taotaro1104-commits/seo-audit-core" }
```

```rust
use seo_audit_core::{check_robots, serp_preview, text_width_px};

// 検索結果での表示幅と省略位置（PC: 600px / モバイル: 680px）
let json = serp_preview("SEOレポート! — URLを入れるだけの無料SEO分析ツールで今すぐ診断", "説明文");
// => {"title":{"pc":{"px":633.2,"max_px":600.0,"truncated":true,"chars_fit":34,"display":"…今す..."}, ...}}

// 全角は1em、ASCIIはArial幅テーブルで計算
assert_eq!(text_width_px("あいう", 20.0), 60.0);

// robots.txt でクロール可能か
let robots = "User-agent: *\nDisallow: /private/\n";
let json = check_robots(robots, 200, "https://example.com/private/a", "Googlebot");
// => {"state":"ok","allowed":false,"group":"*","matched_rule":{"kind":"disallow","pattern":"/private/","line":2},...}
```

動くサンプル: `cargo run --example quickstart`

### インデックス可否

```rust
let input = r#"{
  "requested_url": "https://example.com/a",
  "status_code": 200,
  "meta_robots": "noindex, follow",
  "canonical": "https://example.com/b",
  "robots_allowed": true,
  "lang": "en"
}"#;
let verdict = seo_audit_core::check_indexability(input);
// indexable / noindex / noindex_sources / canonical_state / findings[] を含むJSON
```

## 設計メモ

- **表示幅はヒューリスティック**: Googleの実際の省略条件は非公開のため、定数（title PC 600px、モバイル 340px×2 など）は実測ベースの参考値です。
- **ASCII は Arial の文字幅テーブル**（`i` は細く `W` は太い）、CJK・全角は 1em、半角カナは 0.5em。
- **純粋関数のみ**: HTML取得やHTTPはライブラリの外。呼び出し側が取得結果（ステータス、ヘッダ、robots.txt本文）を渡します。

## 開発

```bash
cargo test
cargo build --target wasm32-unknown-unknown
```

## Roadmap

- 100点満点スコアリング（現在は5言語メッセージと密結合のため未収録。分離後に追加予定）
- CLI / GitHub Action 版

## License

MIT
