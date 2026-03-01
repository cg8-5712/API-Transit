use regex::Regex;

use crate::db::entities::route_rules;

/// Result of a successful route match.
pub struct RewriteResult {
    /// The new path to forward to the upstream.
    pub path: String,
    /// Upstream pinned by the rule (None = use global load balancer).
    pub upstream_id: Option<i64>,
}

/// Match `request_path` against the sorted list of enabled rules and produce
/// a rewritten path.  Rules are expected to be ordered by priority descending
/// (highest priority first) — the first match wins.
pub fn rewrite(rules: &[route_rules::Model], request_path: &str) -> Option<RewriteResult> {
    for rule in rules {
        if let Some(path) = apply_rule(rule, request_path) {
            return Some(RewriteResult {
                path,
                upstream_id: rule.upstream_id,
            });
        }
    }
    None
}

fn apply_rule(rule: &route_rules::Model, path: &str) -> Option<String> {
    match rule.match_type.as_str() {
        "exact" => {
            if path == rule.inbound_path {
                Some(rule.outbound_path.clone())
            } else {
                None
            }
        }
        "prefix" => {
            if path.starts_with(&rule.inbound_path) {
                let suffix = &path[rule.inbound_path.len()..];
                let mut new_path = rule.outbound_path.clone();
                // Avoid double slashes when prefix ends with '/' and suffix starts with '/'.
                if new_path.ends_with('/') && suffix.starts_with('/') {
                    new_path.push_str(&suffix[1..]);
                } else {
                    new_path.push_str(suffix);
                }
                Some(new_path)
            } else {
                None
            }
        }
        "regex" => apply_regex_rule(&rule.inbound_path, &rule.outbound_path, path),
        unknown => {
            tracing::warn!(match_type = %unknown, "Unknown match_type, skipping rule");
            None
        }
    }
}

fn apply_regex_rule(pattern: &str, replacement: &str, path: &str) -> Option<String> {
    let re = match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(pattern = %pattern, error = %e, "Invalid regex in route rule");
            return None;
        }
    };
    if re.is_match(path) {
        Some(re.replace(path, replacement).into_owned())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::entities::route_rules;
    use chrono::Utc;
    use sea_orm::prelude::DateTimeUtc;

    fn make_rule(
        inbound: &str,
        outbound: &str,
        match_type: &str,
        priority: i32,
    ) -> route_rules::Model {
        route_rules::Model {
            id: 1,
            name: "test".to_string(),
            inbound_path: inbound.to_string(),
            outbound_path: outbound.to_string(),
            match_type: match_type.to_string(),
            upstream_id: None,
            priority,
            extra_headers: None,
            extra_query: None,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_exact_match() {
        let rules = vec![make_rule("/api/v1/infer", "/v1/inference", "exact", 10)];
        let result = rewrite(&rules, "/api/v1/infer").unwrap();
        assert_eq!(result.path, "/v1/inference");
    }

    #[test]
    fn test_exact_no_match() {
        let rules = vec![make_rule("/api/v1/infer", "/v1/inference", "exact", 10)];
        assert!(rewrite(&rules, "/api/v1/other").is_none());
    }

    #[test]
    fn test_prefix_match() {
        let rules = vec![make_rule("/api/", "/v1/", "prefix", 10)];
        let result = rewrite(&rules, "/api/chat/completions").unwrap();
        assert_eq!(result.path, "/v1/chat/completions");
    }

    #[test]
    fn test_regex_match() {
        let rules = vec![make_rule(
            r"/api/models/(\w+)/chat",
            "/v1/chat/completions",
            "regex",
            10,
        )];
        let result = rewrite(&rules, "/api/models/gpt-4/chat").unwrap();
        assert_eq!(result.path, "/v1/chat/completions");
    }
}
