//! Shared utilities for operating system security

use serde::{Deserialize, Serialize};

/// Weak password detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeakPasswordResult {
    pub username: String,
    pub password: String,
    pub is_weak: bool,
    pub reason: String,
    pub severity: String, // "low", "medium", "high", "critical"
}

/// Security policy assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicyResult {
    pub policy_name: String,
    pub is_compliant: bool,
    pub current_value: String,
    pub expected_value: String,
    pub severity: String,
    pub recommendation: String,
}

/// CVE vulnerability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CveInfo {
    pub id: String, // CVE-2024-1234
    pub description: String,
    pub severity: String, // CRITICAL, HIGH, MEDIUM, LOW
    pub cvss_score: Option<f64>,
    pub published_date: Option<String>,
    pub affected_products: Vec<String>,
    pub references: Vec<String>,
    pub exploit_available: bool,
}

/// Threat intelligence result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIntelResult {
    pub indicator: String,      // IP, domain, hash
    pub indicator_type: String, // "ip", "domain", "hash"
    pub malicious: bool,
    pub confidence: f64,          // 0.0 - 1.0
    pub threat_type: Vec<String>, // "malware", "phishing", "botnet", etc.
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
    pub related_indicators: Vec<String>,
    pub source: String,
}

/// Phishing URL detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhishingDetectionResult {
    pub url: String,
    pub is_phishing: bool,
    pub confidence: f64,
    pub reasons: Vec<String>,
    pub redirects: Vec<String>,
    pub domain_reputation: String,
}

/// Password strength level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PasswordStrength {
    VeryWeak,
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

/// Common weak passwords database (for demonstration)
pub const COMMON_WEAK_PASSWORDS: &[&str] = &[
    "password",
    "123456",
    "12345678",
    "123456789",
    "12345",
    "1234567",
    "1234567890",
    "qwerty",
    "abc123",
    "password1",
    "admin",
    "admin123",
    "welcome",
    "letmein",
    "monkey",
    "dragon",
    "master",
    "sunshine",
    "princess",
    "qwerty123",
    "iloveyou",
    "baseball",
    "football",
    "superman",
    "michael",
    "jordan",
    "killer",
    "hunter",
    "shadow",
    "password123",
    "1234567890",
    "qwertyuiop",
    "passw0rd",
    "p@ssw0rd",
];

/// Common weak username patterns
pub const COMMON_WEAK_USERNAMES: &[&str] = &[
    "admin",
    "root",
    "user",
    "guest",
    "test",
    "demo",
    "administrator",
    "sysadmin",
    "webmaster",
    "postgres",
    "mysql",
    "oracle",
    "sa",
];

/// Security policy definitions
pub const SECURITY_POLICIES: &[(&str, &str, &str)] = &[
    (
        "password_min_length",
        "Minimum password length should be at least 8 characters",
        "8",
    ),
    (
        "password_complexity",
        "Password must contain uppercase, lowercase, number, special character",
        "true",
    ),
    (
        "password_history",
        "Password history should remember at least 5 passwords",
        "5",
    ),
    (
        "account_lockout_threshold",
        "Account should lock after 5 failed attempts",
        "5",
    ),
    (
        "account_lockout_duration",
        "Account lockout duration should be at least 15 minutes",
        "15",
    ),
    (
        "session_timeout",
        "Session timeout should be set to 30 minutes or less",
        "30",
    ),
    (
        "mfa_required",
        "Multi-factor authentication should be enabled for all users",
        "true",
    ),
    (
        "audit_logging",
        "Audit logging should be enabled for security events",
        "true",
    ),
];

/// Common CVE database (simplified for demonstration)
pub const COMMON_CVES: &[(&str, &str, &str, f64)] = &[
    ("CVE-2024-1234", "Buffer overflow in service X", "HIGH", 7.5),
    (
        "CVE-2024-5678",
        "SQL injection vulnerability",
        "CRITICAL",
        9.8,
    ),
    (
        "CVE-2024-9012",
        "Cross-site scripting vulnerability",
        "MEDIUM",
        6.1,
    ),
    (
        "CVE-2024-3456",
        "Remote code execution vulnerability",
        "CRITICAL",
        9.0,
    ),
    (
        "CVE-2024-7890",
        "Privilege escalation vulnerability",
        "HIGH",
        7.8,
    ),
    (
        "CVE-2024-2345",
        "Information disclosure vulnerability",
        "MEDIUM",
        5.3,
    ),
    (
        "CVE-2024-6789",
        "Denial of service vulnerability",
        "HIGH",
        7.0,
    ),
    (
        "CVE-2024-0123",
        "Authentication bypass vulnerability",
        "CRITICAL",
        9.1,
    ),
    (
        "CVE-2024-4567",
        "Insecure deserialization vulnerability",
        "HIGH",
        8.1,
    ),
    (
        "CVE-2024-8901",
        "Server-side request forgery vulnerability",
        "MEDIUM",
        6.5,
    ),
];

/// Threat intelligence data (simplified)
pub const THREAT_INTEL_DATA: &[(&str, &str, bool, f64, &str)] = &[
    ("185.130.5.253", "ip", true, 0.95, "Known malware C2 server"),
    (
        "45.33.22.11",
        "ip",
        true,
        0.92,
        "Botnet command and control",
    ),
    ("8.8.8.8", "ip", false, 0.0, "Google DNS - Legitimate"),
    ("1.1.1.1", "ip", false, 0.0, "Cloudflare DNS - Legitimate"),
    (
        "malware.example.com",
        "domain",
        true,
        0.98,
        "Known malware distribution domain",
    ),
    (
        "phishing.example.org",
        "domain",
        true,
        0.96,
        "Active phishing domain",
    ),
    ("google.com", "domain", false, 0.0, "Legitimate domain"),
    ("github.com", "domain", false, 0.0, "Legitimate domain"),
    (
        "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8",
        "hash",
        true,
        1.0,
        "Known malware hash (SHA-256)",
    ),
    (
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "hash",
        false,
        0.0,
        "Empty file hash",
    ),
];

/// Phishing URL patterns
pub const PHISHING_INDICATORS: &[(&str, &str)] = &[
    ("secure-login", "Common phishing keyword"),
    ("account-verify", "Common phishing keyword"),
    ("update-payment", "Common phishing keyword"),
    ("confirm-identity", "Common phishing keyword"),
    ("password-reset", "Common phishing keyword"),
    ("banking-secure", "Common phishing keyword"),
    ("appleid", "Common phishing keyword"),
    ("microsoft", "Common phishing keyword"),
    ("paypal", "Common phishing keyword"),
    ("amazon", "Common phishing keyword"),
    ("netflix", "Common phishing keyword"),
    ("spotify", "Common phishing keyword"),
];

/// Check if password is weak
pub fn is_password_weak(password: &str) -> (bool, String) {
    let password_lower = password.to_lowercase();
    // Check against common weak passwords
    if COMMON_WEAK_PASSWORDS.contains(&password_lower.as_str()) {
        return (
            true,
            "Password is in the list of common weak passwords".to_string(),
        );
    }
    // Check length
    if password.len() < 8 {
        return (
            true,
            "Password is too short (less than 8 characters)".to_string(),
        );
    }
    // Check complexity
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    if !has_upper || !has_lower || !has_digit || !has_special {
        return (
            true,
            "Password lacks complexity: need uppercase, lowercase, digit, and special character"
                .to_string(),
        );
    }
    // Check for repeated patterns
    if password.len() >= 4 {
        for i in 0..password.len() - 3 {
            let substr = &password[i..i + 4];
            if password.matches(substr).count() > 1 {
                return (true, "Password contains repeated patterns".to_string());
            }
        }
    }
    // Check for sequential characters
    let seq = "abcdefghijklmnopqrstuvwxyz0123456789";
    for i in 0..seq.len().saturating_sub(3) {
        if password_lower.contains(&seq[i..i + 3]) {
            return (true, "Password contains sequential characters".to_string());
        }
    }
    (false, "Password meets security requirements".to_string())
}

/// Get password strength
pub fn get_password_strength(password: &str) -> PasswordStrength {
    let (is_weak, _) = is_password_weak(password);
    if is_weak {
        return PasswordStrength::Weak;
    }
    let len = password.len();
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    let mut score = 0;
    if len >= 12 {
        score += 2;
    } else if len >= 8 {
        score += 1;
    }
    if has_upper {
        score += 1;
    }
    if has_lower {
        score += 1;
    }
    if has_digit {
        score += 1;
    }
    if has_special {
        score += 2;
    }
    match score {
        0..=2 => PasswordStrength::VeryWeak,
        3..=4 => PasswordStrength::Weak,
        5..=6 => PasswordStrength::Medium,
        7..=8 => PasswordStrength::Strong,
        _ => PasswordStrength::VeryStrong,
    }
}

/// Generate password dictionary
pub fn generate_password_dict(seed: &str, count: usize) -> Vec<String> {
    let mut dict: Vec<String> = Vec::new();
    let base = seed.to_lowercase();
    // Add common variations
    dict.push(base.clone());
    dict.push(format!("{}{}", base, "123"));
    dict.push(format!("{}{}", base, "1234"));
    dict.push(format!("{}{}", base, "!"));
    dict.push(format!("{}{}", base, "@"));
    dict.push(format!("{}{}", base, "2024"));
    dict.push(format!("{}{}", base, "2025"));
    dict.push(format!("{}{}", base, "!@#"));
    dict.push(format!("{}{}", base, "admin"));
    dict.push(format!("{}{}", base, "password"));
    dict.push(format!("{}{}", base, "123456"));
    // Add some common weak passwords if seed is empty or common
    if seed.is_empty() || seed.len() < 3 {
        // Convert &str to String for each element
        for pwd in COMMON_WEAK_PASSWORDS {
            dict.push(pwd.to_string());
        }
    }
    // Capitalize variations
    if !base.is_empty() {
        let mut capitalized = base.clone();
        if let Some(first) = capitalized.chars().next() {
            capitalized.remove(0);
            let cap = first.to_uppercase().to_string();
            dict.push(format!("{}{}", cap, capitalized));
        }
    }
    // Limit results
    dict.truncate(count);
    dict
}

/// Query CVE by ID
pub fn query_cve(cve_id: &str) -> Option<CveInfo> {
    for (id, desc, severity, score) in COMMON_CVES {
        if id.eq_ignore_ascii_case(cve_id) {
            return Some(CveInfo {
                id: id.to_string(),
                description: desc.to_string(),
                severity: severity.to_string(),
                cvss_score: Some(*score),
                published_date: Some("2024-01-01".to_string()),
                affected_products: vec!["All systems".to_string()],
                references: vec![format!("https://nvd.nist.gov/vuln/detail/{}", id)],
                exploit_available: *score >= 7.0,
            });
        }
    }
    None
}

/// Query CVEs by keyword
pub fn query_cves_by_keyword(keyword: &str) -> Vec<CveInfo> {
    let keyword_lower = keyword.to_lowercase();
    COMMON_CVES
        .iter()
        .filter(|(id, desc, _, _)| {
            id.to_lowercase().contains(&keyword_lower)
                || desc.to_lowercase().contains(&keyword_lower)
        })
        .map(|(id, desc, severity, score)| CveInfo {
            id: id.to_string(),
            description: desc.to_string(),
            severity: severity.to_string(),
            cvss_score: Some(*score),
            published_date: Some("2024-01-01".to_string()),
            affected_products: vec!["All systems".to_string()],
            references: vec![format!("https://nvd.nist.gov/vuln/detail/{}", id)],
            exploit_available: *score >= 7.0,
        })
        .collect()
}

/// Query threat intelligence
pub fn query_threat_intel(indicator: &str) -> ThreatIntelResult {
    let indicator_lower = indicator.to_lowercase();
    for (ind, ind_type, malicious, confidence, desc) in THREAT_INTEL_DATA {
        if ind.eq_ignore_ascii_case(&indicator_lower) {
            let threat_type = if *malicious {
                if desc.contains("malware") {
                    vec!["malware".to_string()]
                } else if desc.contains("phishing") {
                    vec!["phishing".to_string()]
                } else if desc.contains("botnet") {
                    vec!["botnet".to_string()]
                } else {
                    vec!["suspicious".to_string()]
                }
            } else {
                vec!["legitimate".to_string()]
            };
            return ThreatIntelResult {
                indicator: ind.to_string(),
                indicator_type: ind_type.to_string(),
                malicious: *malicious,
                confidence: *confidence,
                threat_type,
                first_seen: Some("2024-01-01".to_string()),
                last_seen: Some("2024-06-01".to_string()),
                related_indicators: vec![],
                source: "Internal Threat Intelligence Database".to_string(),
            };
        }
    }
    // Unknown indicator
    ThreatIntelResult {
        indicator: indicator.to_string(),
        indicator_type: "unknown".to_string(),
        malicious: false,
        confidence: 0.0,
        threat_type: vec!["unknown".to_string()],
        first_seen: None,
        last_seen: None,
        related_indicators: vec![],
        source: "Internal Threat Intelligence Database".to_string(),
    }
}

/// Detect phishing URL
pub fn detect_phishing(url: &str) -> PhishingDetectionResult {
    let url_lower = url.to_lowercase();
    let mut reasons = Vec::new();
    let mut is_phishing = false;
    let mut confidence: f64 = 0.0;
    // Check for common phishing patterns
    for (pattern, reason) in PHISHING_INDICATORS {
        if url_lower.contains(pattern) {
            reasons.push(format!("Contains suspicious keyword: {}", reason));
            confidence += 0.1;
        }
    }
    // Check for domain spoofing
    let domains = [
        "paypal",
        "amazon",
        "microsoft",
        "apple",
        "google",
        "netflix",
        "spotify",
    ];
    let suspicious_domains = ["login", "verify", "secure", "account", "update", "confirm"];
    let mut count_spoofed = 0;
    let mut count_suspicious = 0;
    for domain in &domains {
        if url_lower.contains(domain) {
            count_spoofed += 1;
        }
    }
    for domain in &suspicious_domains {
        if url_lower.contains(domain) {
            count_suspicious += 1;
        }
    }
    if count_spoofed > 0 && count_suspicious > 0 {
        reasons.push("Potential domain spoofing with suspicious keywords".to_string());
        confidence += 0.3;
    }
    // Check for IP address in URL
    if url_lower.contains("://") {
        let domain_part = url_lower.split("://").nth(1).unwrap_or("");
        let ip_pattern = r"^(\d{1,3}\.){3}\d{1,3}";
        if regex::Regex::new(ip_pattern)
            .unwrap_or_else(|_| regex::Regex::new(r"^$").unwrap())
            .is_match(&domain_part.split('/').next().unwrap_or(""))
        {
            reasons.push("URL uses IP address instead of domain name".to_string());
            confidence += 0.2;
        }
    }
    // Check for URL shorteners
    let shorteners = [
        "bit.ly", "tinyurl", "goo.gl", "shorturl", "rebrand", "is.gd",
    ];
    for shortener in &shorteners {
        if url_lower.contains(shortener) {
            reasons.push("URL uses a URL shortener service".to_string());
            confidence += 0.1;
        }
    }
    // Check for HTTPS
    if !url_lower.starts_with("https://") && !url_lower.starts_with("http://") {
        // no protocol, less suspicious
    } else if url_lower.starts_with("http://") && !url_lower.contains("localhost") {
        // HTTP without HTTPS
        let domain_part = url_lower.split("://").nth(1).unwrap_or("");
        if !domain_part.starts_with("localhost") && !domain_part.starts_with("127.0.0.1") {
            reasons.push("URL uses insecure HTTP protocol".to_string());
            confidence += 0.05;
        }
    }
    // Check domain reputation from threat intel
    let domain = url_lower.split('/').nth(2).unwrap_or("");
    if !domain.is_empty() {
        let domain_intel = query_threat_intel(domain);
        if domain_intel.malicious {
            reasons.push(format!(
                "Domain is flagged as malicious by threat intelligence: {}",
                domain
            ));
            confidence += 0.5;
        }
    }
    // Determine phishing status
    is_phishing = confidence >= 0.5;
    confidence = confidence.min(1.0);
    let domain_reputation = if confidence >= 0.8 {
        "Very Suspicious".to_string()
    } else if confidence >= 0.5 {
        "Suspicious".to_string()
    } else if confidence >= 0.2 {
        "Moderate".to_string()
    } else {
        "Legitimate".to_string()
    };
    PhishingDetectionResult {
        url: url.to_string(),
        is_phishing,
        confidence,
        reasons,
        redirects: vec![],
        domain_reputation,
    }
}

/// Check security policy compliance
pub fn check_security_policies() -> Vec<SecurityPolicyResult> {
    let mut results = Vec::new();
    for (name, desc, expected_val) in SECURITY_POLICIES {
        let current_val = get_policy_current_value(name);
        let is_compliant = current_val == *expected_val;
        let severity = if !is_compliant && name.contains("password") {
            "high".to_string()
        } else if !is_compliant {
            "medium".to_string()
        } else {
            "low".to_string()
        };
        results.push(SecurityPolicyResult {
            policy_name: name.to_string(),
            is_compliant,
            current_value: current_val,
            expected_value: expected_val.to_string(),
            severity,
            recommendation: if is_compliant {
                "No action needed".to_string()
            } else {
                format!("Update policy to meet security requirements: {}", desc)
            },
        });
    }
    results
}

/// Get policy current value (simulated)
fn get_policy_current_value(policy_name: &str) -> String {
    // Simulate values - in a real system, these would be read from configuration
    match policy_name {
        "password_min_length" => "8".to_string(),
        "password_complexity" => "true".to_string(),
        "password_history" => "5".to_string(),
        "account_lockout_threshold" => "5".to_string(),
        "account_lockout_duration" => "15".to_string(),
        "session_timeout" => "30".to_string(),
        "mfa_required" => "false".to_string(), // Non-compliant
        "audit_logging" => "true".to_string(),
        _ => "unknown".to_string(),
    }
}
