use anyhow::{Result, bail};
use regex::Regex;
use std::path::Path;
use std::process::Command;

/// Patterns whose VALUES (after separator) are redacted.
const SECRET_KV_PATTERN: &str = r#"(?i)(OPENAI_API_KEY|ANTHROPIC_API_KEY|AWS_SECRET_ACCESS_KEY|[Aa][Pp][Ii][_-]?[Kk][Ee][Yy]|[Pp]assword|[Tt]oken|[Ss]ecret)\s*[:=]\s*["']?([A-Za-z0-9._/+\-]{8,})"#;
const PRIVATE_KEY_PATTERN: &str = r"-----BEGIN [A-Z ]* PRIVATE KEY-----";

/// Redact secret values from text. Returns (redacted_text, redaction_count).
pub fn redact_secrets(input: &str) -> (String, usize) {
    let kv_re = Regex::new(SECRET_KV_PATTERN).expect("valid regex");
    let pk_re = Regex::new(PRIVATE_KEY_PATTERN).expect("valid regex");

    let mut count = 0usize;

    // Redact key=value pairs: keep key, replace value with [REDACTED]
    let after_kv = kv_re.replace_all(input, |caps: &regex::Captures| {
        count += 1;
        format!("{}=[REDACTED]", &caps[1])
    });

    // Redact private key headers
    let after_pk = pk_re.replace_all(&after_kv, |_: &regex::Captures| {
        count += 1;
        "[REDACTED PRIVATE KEY]".to_string()
    });

    (after_pk.into_owned(), count)
}

/// Hard-fail check: unfilled template placeholders in artifact text.
/// Returns Ok(()) if clean, Err with the offending path on failure.
pub fn check_no_unfilled_placeholders(path: &Path, content: &str) -> Result<()> {
    // Strip inline code spans, then strip HTML comment blocks, strip blockquotes,
    // then remove allowed documentation occurrences before checking for bare placeholders.
    let after_inline_code = strip_inline_code(content);
    let after_html_comments = strip_html_comments(&after_inline_code);
    let after_blockquotes = strip_blockquotes(&after_html_comments);

    // Remove allowed patterns: →UPG-####, UPG-####__...
    let after_arrows = after_blockquotes.replace("→UPG-####", "");
    let ugp_prefixed = Regex::new(r"UPG-####__\S+").expect("valid regex");
    let after_prefixed = ugp_prefixed.replace_all(&after_arrows, "UPG-FILLED__");

    if after_prefixed.contains("UPG-####") {
        bail!(
            "precheck failed — literal placeholder 'UPG-####' found in {} (fill in the real UPG id)",
            path.display()
        );
    }

    let after_arrows_chg = after_prefixed.replace("→CHG-YYYYMMDD-NNN", "");
    let chg_prefixed = Regex::new(r"CHG-YYYYMMDD-NNN__\S+").expect("valid regex");
    let after_chg = chg_prefixed.replace_all(&after_arrows_chg, "CHG-FILLED__");

    if after_chg.contains("CHG-YYYYMMDD-NNN") {
        bail!(
            "precheck failed — literal placeholder 'CHG-YYYYMMDD-NNN' found in {} (fill in the real CHG id)",
            path.display()
        );
    }

    Ok(())
}

/// Hard-fail check: forbidden field `latest_review:` (superseded by UPG-0001).
pub fn check_no_forbidden_fields(path: &Path, content: &str) -> Result<()> {
    let re = Regex::new(r"(?m)^\s*latest_review:").expect("valid regex");
    if re.is_match(content) {
        bail!(
            "precheck failed — forbidden field 'latest_review:' found in {} (use review_state instead)",
            path.display()
        );
    }
    Ok(())
}

/// Warning-only check: draft markers.
pub fn check_draft_markers(path: &Path, content: &str) -> bool {
    let re = Regex::new(r"(?i)TODO|FIXME|\bTBD\b|\[to be filled\]").expect("valid regex");
    if re.is_match(content) {
        eprintln!(
            "warning: precheck — unresolved draft marker (TODO/FIXME/TBD/[to be filled]) in {}",
            path.display()
        );
        return true;
    }
    false
}

/// Check that a guard-clean path has no uncommitted changes vs HEAD.
pub fn check_guard_clean(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!(
            "precheck failed — --guard-clean path not found: {}",
            path.display()
        );
    }
    let output = Command::new("git")
        .args(["diff", "--quiet", "HEAD", "--"])
        .arg(path)
        .status()
        .map_err(|e| anyhow::anyhow!("git diff failed: {}", e))?;

    if !output.success() {
        bail!(
            "precheck failed — --guard-clean path '{}' has uncommitted changes (expected clean)",
            path.display()
        );
    }
    Ok(())
}

fn strip_inline_code(s: &str) -> String {
    let re = Regex::new(r"`[^`]*`").expect("valid regex");
    re.replace_all(s, "").into_owned()
}

fn strip_html_comments(s: &str) -> String {
    // Multi-line HTML comment stripping
    let mut result = String::new();
    let mut in_comment = false;
    for line in s.lines() {
        if in_comment {
            if line.contains("-->") {
                in_comment = false;
            }
            result.push('\n');
        } else if line.contains("<!--") {
            in_comment = !line.contains("-->");
            result.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

fn strip_blockquotes(s: &str) -> String {
    s.lines()
        .filter(|l| !l.trim_start().starts_with('>'))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn p(s: &str) -> PathBuf { PathBuf::from(s) }

    #[test]
    fn secret_redaction_detects_api_key() {
        let input = "OPENAI_API_KEY=sk-abc1234567890abcdef\n";
        let (out, count) = redact_secrets(input);
        assert!(out.contains("[REDACTED]"), "should redact: {}", out);
        assert_eq!(count, 1);
    }

    #[test]
    fn secret_redaction_passes_short_value() {
        // Values shorter than 8 chars are not redacted (not a real secret)
        let input = "TOKEN=short\n";
        let (out, count) = redact_secrets(input);
        assert_eq!(count, 0, "short value should not be redacted: {}", out);
    }

    #[test]
    fn secret_redaction_detects_private_key() {
        let input = "-----BEGIN RSA PRIVATE KEY-----\nABCDEF\n-----END RSA PRIVATE KEY-----\n";
        let (out, count) = redact_secrets(input);
        assert!(out.contains("[REDACTED PRIVATE KEY]"), "should redact pk: {}", out);
        assert_eq!(count, 1);
    }

    #[test]
    fn placeholder_ugp_fails() {
        let result = check_no_unfilled_placeholders(&p("test.md"), "feature: UPG-####\n");
        assert!(result.is_err());
    }

    #[test]
    fn placeholder_in_code_span_passes() {
        // Inside inline code span: allowed documentation occurrence
        let result = check_no_unfilled_placeholders(&p("test.md"), "like `UPG-####`\n");
        assert!(result.is_ok(), "code-span placeholder should pass: {:?}", result);
    }

    #[test]
    fn placeholder_with_prefix_passes() {
        // UPG-####__ prefix is an allowed pattern
        let result = check_no_unfilled_placeholders(&p("test.md"), "UPG-####__CHG-20260702-001\n");
        assert!(result.is_ok(), "prefixed placeholder should pass: {:?}", result);
    }

    #[test]
    fn forbidden_field_fails() {
        let result = check_no_forbidden_fields(&p("test.md"), "latest_review: REV__001\n");
        assert!(result.is_err());
    }

    #[test]
    fn forbidden_field_absent_passes() {
        let result = check_no_forbidden_fields(&p("test.md"), "review_state: DRAFT\n");
        assert!(result.is_ok());
    }
}
