//! Pattern-based secret and misconfiguration scanner.

use regex::Regex;

use crate::security::domain::{DetectedFinding, FindingType, Severity};

struct ScanRule {
    name: &'static str,
    pattern: Regex,
    finding_type: FindingType,
    severity: Severity,
    title: &'static str,
}

/// Scans text content for secrets and weak configurations.
pub struct SecurityScanner;

impl SecurityScanner {
    /// Runs all detection rules against the given content.
    #[must_use]
    pub fn scan(content: &str, resource: Option<&str>) -> Vec<DetectedFinding> {
        let rules = rules();
        let mut findings = Vec::new();

        for rule in &rules {
            if rule.pattern.is_match(content) {
                findings.push(DetectedFinding {
                    finding_type: rule.finding_type,
                    severity: rule.severity,
                    title: rule.title.to_string(),
                    description: format!(
                        "Detected {} pattern in scanned content. Sensitive values are redacted in storage.",
                        rule.name
                    ),
                    resource: resource.map(str::to_string),
                });
            }
        }

        findings.extend(scan_weak_configs(content, resource));
        findings
    }

    /// Redacts known secret patterns for safe display.
    #[must_use]
    pub fn redact(content: &str) -> String {
        let mut redacted = content.to_string();

        for rule in rules() {
            redacted = rule
                .pattern
                .replace_all(&redacted, "[REDACTED]")
                .into_owned();
        }

        redacted
    }
}

fn scan_weak_configs(content: &str, resource: Option<&str>) -> Vec<DetectedFinding> {
    let mut findings = Vec::new();
    let lower = content.to_lowercase();

    if lower.contains("debug=true") || lower.contains("debug: true") {
        findings.push(DetectedFinding {
            finding_type: FindingType::WeakConfig,
            severity: Severity::Medium,
            title: "Debug mode enabled".into(),
            description: "Debug mode should be disabled in production environments.".into(),
            resource: resource.map(str::to_string),
        });
    }

    if lower.contains("sslmode=disable") || lower.contains("verify=false") {
        findings.push(DetectedFinding {
            finding_type: FindingType::WeakConfig,
            severity: Severity::High,
            title: "TLS verification disabled".into(),
            description: "Transport security verification is disabled, enabling MITM risk.".into(),
            resource: resource.map(str::to_string),
        });
    }

    if lower.contains("password=password") || lower.contains("password: \"123456\"") {
        findings.push(DetectedFinding {
            finding_type: FindingType::WeakConfig,
            severity: Severity::Critical,
            title: "Weak default password".into(),
            description: "A weak or default password was detected in configuration.".into(),
            resource: resource.map(str::to_string),
        });
    }

    findings
}

fn rules() -> Vec<ScanRule> {
    vec![
        ScanRule {
            name: "AWS access key",
            pattern: Regex::new(r"AKIA[0-9A-Z]{16}").expect("regex"),
            finding_type: FindingType::Secret,
            severity: Severity::Critical,
            title: "AWS access key detected",
        },
        ScanRule {
            name: "GitHub token",
            pattern: Regex::new(r"ghp_[A-Za-z0-9]{20,}").expect("regex"),
            finding_type: FindingType::ApiKey,
            severity: Severity::Critical,
            title: "GitHub personal access token",
        },
        ScanRule {
            name: "Slack token",
            pattern: Regex::new(r"xox[baprs]-[A-Za-z0-9-]{10,}").expect("regex"),
            finding_type: FindingType::ApiKey,
            severity: Severity::High,
            title: "Slack API token detected",
        },
        ScanRule {
            name: "Private key",
            pattern: Regex::new(r"-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----").expect("regex"),
            finding_type: FindingType::Secret,
            severity: Severity::Critical,
            title: "Private key material detected",
        },
        ScanRule {
            name: "Generic API key",
            pattern: Regex::new(r"(?i)(api[_-]?key|secret[_-]?key)\s*[:=]\s*['\x22]?[A-Za-z0-9_\-]{16,}")
                .expect("regex"),
            finding_type: FindingType::ApiKey,
            severity: Severity::High,
            title: "Hardcoded API key",
        },
        ScanRule {
            name: "Exposed database port",
            pattern: Regex::new(r"0\.0\.0\.0:(5432|3306|27017)").expect("regex"),
            finding_type: FindingType::ExposedPort,
            severity: Severity::High,
            title: "Database port exposed on all interfaces",
        },
        ScanRule {
            name: "Docker privileged",
            pattern: Regex::new(r"(?i)privileged:\s*true\b").expect("regex"),
            finding_type: FindingType::DockerIssue,
            severity: Severity::High,
            title: "Privileged Docker container",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_github_token() {
        let token = "ghp_abcdefghijklmnopqrstuvwxyz123456";
        let findings = SecurityScanner::scan(&format!("token = {token}"), None);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn redacts_secrets() {
        let token = "ghp_abcdefghijklmnopqrstuvwxyz123456";
        let redacted = SecurityScanner::redact(&format!("key={token}"));
        assert!(!redacted.contains("ghp_"));
    }
}
