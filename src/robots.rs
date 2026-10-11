// robots.txt の解析と URL の許可判定（RFC 9309 / Google の解釈に準拠）
use serde::Serialize;
use url::Url;

const MAX_PATTERN_LEN: usize = 1000;

#[derive(Serialize, Clone)]
pub struct Rule {
    /// "allow" | "disallow"
    pub kind: &'static str,
    pub pattern: String,
    pub line: usize,
}

struct Group {
    agents: Vec<String>,
    rules: Vec<Rule>,
}

#[derive(Serialize)]
pub struct RobotsCheck {
    /// "ok" | "not_found" | "server_error"
    pub state: &'static str,
    pub allowed: bool,
    /// 適用されたグループのUA（小文字）。該当なしは null
    pub group: Option<String>,
    pub matched_rule: Option<Rule>,
    pub sitemaps: Vec<String>,
}

pub fn check(robots_txt: &str, status_code: u16, url: &str, user_agent: &str) -> RobotsCheck {
    let (groups, sitemaps) = parse(robots_txt);
    let result = |state, allowed, group, matched_rule| RobotsCheck {
        state, allowed, group, matched_rule, sitemaps: sitemaps.clone(),
    };

    // Google: 4xx は制限なし、5xx・接続失敗は全拒否として扱う
    match status_code {
        200..=299 => {}
        400..=499 => return result("not_found", true, None, None),
        _ => return result("server_error", false, None, None),
    }

    let path = path_and_query(url);
    if path == "/robots.txt" {
        return result("ok", true, None, None);
    }

    let token = user_agent.trim().to_lowercase();
    let Some((group_name, rules)) = select_rules(&groups, &token) else {
        return result("ok", true, None, None);
    };

    let matched = rules.iter()
        .filter(|r| pattern_matches(&r.pattern, &path))
        .max_by(|a, b| {
            let by_len = a.pattern.len().cmp(&b.pattern.len());
            // 同じ長さなら allow を優先
            by_len.then_with(|| (a.kind == "allow").cmp(&(b.kind == "allow")))
        })
        .cloned();
    let allowed = matched.as_ref().map(|r| r.kind == "allow").unwrap_or(true);
    result("ok", allowed, Some(group_name), matched)
}

fn parse(txt: &str) -> (Vec<Group>, Vec<String>) {
    let mut groups: Vec<Group> = Vec::new();
    let mut sitemaps = Vec::new();
    let mut in_agent_block = false;

    for (i, raw) in txt.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        let Some((key, value)) = line.split_once(':') else { continue };
        let key = key.trim().to_lowercase();
        let value = value.trim();

        match key.as_str() {
            "user-agent" => {
                if !in_agent_block {
                    groups.push(Group { agents: Vec::new(), rules: Vec::new() });
                }
                in_agent_block = true;
                if let Some(g) = groups.last_mut() {
                    g.agents.push(value.to_lowercase());
                }
            }
            "allow" | "disallow" => {
                in_agent_block = false;
                // 空の Disallow は「制限なし」なのでルールとして扱わない
                if value.is_empty() || value.len() > MAX_PATTERN_LEN {
                    continue;
                }
                if let Some(g) = groups.last_mut() {
                    let kind = if key == "allow" { "allow" } else { "disallow" };
                    g.rules.push(Rule { kind, pattern: value.to_string(), line: i + 1 });
                }
            }
            "sitemap" => sitemaps.push(value.to_string()),
            _ => {}
        }
    }
    (groups, sitemaps)
}

/// 最も具体的なUAグループのルールを返す（同じUAの複数グループは結合）
fn select_rules(groups: &[Group], token: &str) -> Option<(String, Vec<Rule>)> {
    let best = groups.iter()
        .flat_map(|g| g.agents.iter())
        .filter(|a| !a.is_empty() && a.as_str() != "*" && token.starts_with(a.as_str()))
        .max_by_key(|a| a.len())
        .cloned()
        .or_else(|| (token == "*").then(|| "*".to_string()))
        .or_else(|| groups.iter().any(|g| g.agents.iter().any(|a| a == "*")).then(|| "*".to_string()))?;

    let rules = groups.iter()
        .filter(|g| g.agents.iter().any(|a| *a == best))
        .flat_map(|g| g.rules.iter().cloned())
        .collect();
    Some((best, rules))
}

fn path_and_query(url: &str) -> String {
    match Url::parse(url) {
        Ok(u) => match u.query() {
            Some(q) => format!("{}?{}", u.path(), q),
            None => u.path().to_string(),
        },
        Err(_) => "/".to_string(),
    }
}

/// `*`（任意の文字列）と末尾 `$`（終端）に対応した前方一致。O(パターン長×パス長)
pub fn pattern_matches(pattern: &str, path: &str) -> bool {
    let text = path.as_bytes();
    let (pat, anchored) = match pattern.strip_suffix('$') {
        Some(p) => (p.as_bytes(), true),
        None => (pattern.as_bytes(), false),
    };

    // states[i] = パス先頭 i バイトまで一致済み
    let mut states = vec![false; text.len() + 1];
    states[0] = true;
    for &pc in pat {
        let mut next = vec![false; text.len() + 1];
        if pc == b'*' {
            if let Some(first) = states.iter().position(|&s| s) {
                next[first..].iter_mut().for_each(|s| *s = true);
            }
        } else {
            for i in 0..text.len() {
                if states[i] && text[i] == pc {
                    next[i + 1] = true;
                }
            }
        }
        states = next;
        if !states.iter().any(|&s| s) {
            return false;
        }
    }
    if anchored { states[text.len()] } else { true }
}
