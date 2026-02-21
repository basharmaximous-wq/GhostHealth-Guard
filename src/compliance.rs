use crate::models::Issue;

pub fn hipaa_score(issues: &[Issue]) -> u8 {

    let mut score = 0;

    for issue in issues {
        match issue.category.as_str() {

            "PHI_LOGGING" => score += 40,
            "SENSITIVE_FUNCTION" => score += 35,
            "UNSAFE_FUNCTION" => score += 20,
            "SEMGREP_POLICY" => score += 30,
            _ => score += 10,
        }
    }

    score.min(100)
}
