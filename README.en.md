# seo-audit-core

[![CI](https://github.com/taotaro1104-commits/seo-audit-core/actions/workflows/ci.yml/badge.svg)](https://github.com/taotaro1104-commits/seo-audit-core/actions)
![license](https://img.shields.io/badge/license-MIT-blue)

[日本語](README.md) | English

A Rust crate containing the **pure logic** behind an SEO audit tool. No I/O; inputs and outputs are JSON strings, and it compiles to WASM as-is.

- **SERP preview** — estimate the pixel width of a title / description and where it gets truncated (handles mixed full-width/half-width CJK text)
- **robots.txt check** — user-agent group selection, longest-match Allow/Disallow, `*` and `$`, per-HTTP-status handling
- **sitemap.xml check** — urlset / sitemapindex parsing and whether a URL is listed
- **Indexability verdict** — combines status, redirects, robots.txt, meta robots, X-Robots-Tag, canonical and sitemap into one verdict (messages in 5 languages)

Web version: a free tool running the same logic in the browser → <https://seo.matchinfo-rank.com/>

## Usage

```toml
[dependencies]
seo-audit-core = { git = "https://github.com/taotaro1104-commits/seo-audit-core" }
```

```rust
use seo_audit_core::{check_robots, serp_preview, text_width_px};

let json = serp_preview("My page title", "My meta description");
assert_eq!(text_width_px("あいう", 20.0), 60.0); // full-width = 1em

let robots = "User-agent: *\nDisallow: /private/\n";
let json = check_robots(robots, 200, "https://example.com/private/a", "Googlebot");
// => {"state":"ok","allowed":false,"group":"*","matched_rule":{"kind":"disallow","pattern":"/private/","line":2},...}
```

Runnable example: `cargo run --example quickstart`

`check_indexability` takes a JSON object (`requested_url`, `status_code`, `meta_robots`, `x_robots_tag`, `canonical`, `robots_allowed`, `in_sitemap`, `lang`) and returns `indexable`, `noindex_sources`, `canonical_state` and `findings[]`.

## Notes

- Google's real truncation rules are not public; the width constants (title 600px on PC, 340px×2 on mobile, …) are empirical estimates.
- ASCII uses an Arial width table, CJK/full-width is 1em, half-width katakana is 0.5em.
- Pure functions only: fetching HTML/robots.txt is the caller's job.

## Development

```bash
cargo test
cargo build --target wasm32-unknown-unknown
```

## License

MIT
