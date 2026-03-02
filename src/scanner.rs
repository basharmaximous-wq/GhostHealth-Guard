use crate::models::Issue;
use regex::Regex;
use std::process::Command;
use std::sync::OnceLock;

static PHI_RE: OnceLock<Option<Regex>> = OnceLock::new();
static LOGGING_RE: OnceLock<Option<Regex>> = OnceLock::new();
static UNSAFE_RE: OnceLock<Option<Regex>> = OnceLock::new();
static HARDCODED_RE: OnceLock<Option<Regex>> = OnceLock::new();

pub fn deterministic_scan(diff: &str) -> Vec<Issue> {
    let mut issues = vec![];

    let phi_pattern = match PHI_RE
        .get_or_init(|| {
            Regex::new(r"(?i)(ssn|patient_id|patient|heart_rate|dob|diagnosis|medical_record|name)")
                .ok()
        })
        .as_ref()
    {
        Some(re) => re,
        None => return issues,
    };
    let logging_pattern = match LOGGING_RE
        .get_or_init(|| Regex::new(r"(println!|info!|debug!|warn!|tracing::)").ok())
        .as_ref()
    {
        Some(re) => re,
        None => return issues,
    };
    let unsafe_pattern = match UNSAFE_RE
        .get_or_init(|| Regex::new(r"\bunsafe\s*\{").ok())
        .as_ref()
    {
        Some(re) => re,
        None => return issues,
    };
    let hardcoded_pattern = match HARDCODED_RE
        .get_or_init(|| Regex::new(r#"(?i)(password|secret|api_key|token)\s*=\s*"[^"]+""#).ok())
        .as_ref()
    {
        Some(re) => re,
        None => return issues,
    };

    for (i, line) in diff.lines().enumerate() {
        // PHI being logged
        if phi_pattern.is_match(line) && logging_pattern.is_match(line) {
            issues.push(Issue {
                category: "PHI_LOGGING".into(),
                severity: "HIGH".into(),
                message: format!("PHI field logged at line {} — HIPAA violation", i + 1),
            });
        }

        // Unsafe blocks
        if unsafe_pattern.is_match(line) {
            issues.push(Issue {
                category: "UNSAFE_BLOCK".into(),
                severity: "MEDIUM".into(),
                message: format!("Unsafe block detected at line {}", i + 1),
            });
        }

        // Hardcoded secrets
        if hardcoded_pattern.is_match(line) {
            issues.push(Issue {
                category: "HARDCODED_SECRET".into(),
                severity: "CRITICAL".into(),
                message: format!("Hardcoded secret detected at line {}", i + 1),
            });
        }
    }

    issues
}

#[allow(dead_code)]
pub fn run_semgrep() -> Vec<Issue> {
    let output = Command::new("semgrep")
        .args(["--config", "semgrep/phi_rules.yml", "--json", "."])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains("\"results\"") && !stdout.contains("\"results\": []") {
                return vec![Issue {
                    category: "SEMGREP".into(),
                    severity: "HIGH".into(),
                    message: "Semgrep PHI rule violation detected".into(),
                }];
            }
            vec![]
        }
        Ok(_) => {
            // semgrep ran but found nothing or failed silently
            vec![]
        }
        Err(_) => {
            // semgrep not installed — skip silently
            vec![]
        }
    }
}
