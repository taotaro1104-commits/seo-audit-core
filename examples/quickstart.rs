use seo_audit_core::{check_robots, serp_preview, text_width_px};

fn main() {
    println!("{}", serp_preview("SEOレポート! — URLを入れるだけの無料SEO分析ツールで今すぐ診断", "説明文"));
    println!("{}", text_width_px("あいう", 20.0));
    let robots = "User-agent: *\nDisallow: /private/\n";
    println!("{}", check_robots(robots, 200, "https://example.com/private/a", "Googlebot"));
}
