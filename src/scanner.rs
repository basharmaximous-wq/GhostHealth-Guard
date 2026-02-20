// Deterministic Engine + Semgrep
use crate::models::Issue;
use regex::Regex;
use std::process::Command;

pub fn deterministic_scan(diff: &str) -> Vec<Issue> {
    let mut issues = vec![];

    let phi_pattern = Regex::new(r"(ssn|patient|heart_rate|dob|diagnosis|medical|name)").unwrap();
    let logging_pattern = Regex::new(r"(println!|info!|debug!|tracing::)").unwrap();
    let unsafe_pattern = Regex::new(r"\bunsafe\s*\{").unwrap();

    for (i, line) in diff.lines().enumerate() {
        if phi_pattern.is_match(line) && logging_pattern.is_match(line) {
            issues.push(Issue {
                category: "PHI_LOGGING".into(),
                severity: "HIGH".into(),
                message: format!("PHI logged at line {}", i + 1),
            });
        }

        if unsafe_pattern.is_match(line) {
            issues.push(Issue {
                category: "UNSAFE_BLOCK".into(),
                severity: "MEDIUM".into(),
                message: format!("Unsafe block detected at line {}", i + 1),
            });
        }
    }

    issues
}

pub fn run_semgrep() -> Vec<Issue> {
    let output = Command::new("semgrep")
        .arg("--config")
        .arg("semgrep/phi_rules.yml")
        .arg("--json")
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if stdout.contains("results") {
            return vec![Issue {
                category: "SEMGREP".into(),
                severity: "HIGH".into(),
                message: "Semgrep rule violation detected".into(),
            }];
        }
    }

    vec![]
}
