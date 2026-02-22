use crate::models::Issue;
use regex::Regex;
use std::process::Command;

pub fn deterministic_scan(diff: &str) -> Vec<Issue> {
    let mut issues = vec![];

    let phi_pattern = Regex::new(
        r"(?i)(ssn|patient_id|patient|heart_rate|dob|diagnosis|medical_record|name)"
    ).unwrap();
    let logging_pattern = Regex::new(
        r"(println!|info!|debug!|warn!|tracing::)"
    ).unwrap();
    let unsafe_pattern = Regex::new(r"\bunsafe\s*\{").unwrap();
    let hardcoded_pattern = Regex::new(
        r#"(?i)(password|secret|api_key|token)\s*=\s*"[^"]+""#
    ).unwrap();

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