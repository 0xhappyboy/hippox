//! Shared utilities for operating system security

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use sysinfo::{System, Users};

// ============ Existing types and constants (keep as is) ============

/// Weak password detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeakPasswordResult {
    pub username: String,
    pub password: String,
    pub is_weak: bool,
    pub reason: String,
    pub severity: String,
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
    pub id: String,
    pub description: String,
    pub severity: String,
    pub cvss_score: Option<f64>,
    pub published_date: Option<String>,
    pub affected_products: Vec<String>,
    pub references: Vec<String>,
    pub exploit_available: bool,
}

/// Threat intelligence result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIntelResult {
    pub indicator: String,
    pub indicator_type: String,
    pub malicious: bool,
    pub confidence: f64,
    pub threat_type: Vec<String>,
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

// ============ Permission Check Types ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheckResult {
    pub path: String,
    pub exists: bool,
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
    pub owner: String,
    pub group: String,
    pub permissions: String,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionScanResult {
    pub path: String,
    pub total_files: usize,
    pub issues_found: usize,
    pub results: Vec<PermissionCheckResult>,
}

// ============ Account Security Types ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSecurityResult {
    pub username: String,
    pub uid: u32,
    pub gid: u32,
    pub home_dir: String,
    pub shell: String,
    pub password_expires: Option<String>,
    pub account_locked: bool,
    pub password_empty: bool,
    pub is_root: bool,
    pub is_system: bool,
    pub issues: Vec<String>,
}

// ============ Baseline Check Types ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineCheckResult {
    pub category: String,
    pub check_name: String,
    pub compliant: bool,
    pub current_value: String,
    pub expected_value: String,
    pub severity: String,
    pub recommendation: String,
}

// ============ Share Check Types ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareInfo {
    pub name: String,
    pub path: String,
    pub description: String,
    pub shared: bool,
    pub read_only: bool,
    pub permissions: String,
    pub security_issues: Vec<String>,
}

// ============ System Log Types ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub host: String,
    pub program: String,
    pub pid: Option<u32>,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQueryResult {
    pub total_entries: usize,
    pub entries: Vec<LogEntry>,
    pub query: String,
}

// ============ Persistence Types ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceEntry {
    pub name: String,
    pub path: String,
    pub command: String,
    pub enabled: bool,
    pub source: String,
    pub suspicious: bool,
    pub reason: String,
}

// ============ Privilege Escalation Types ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeEscalationResult {
    pub check_name: String,
    pub vulnerable: bool,
    pub description: String,
    pub details: String,
    pub severity: String,
}

// ============ Patch Detection Types ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchInfo {
    pub name: String,
    pub installed: bool,
    pub version: String,
    pub release_date: Option<String>,
    pub severity: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchScanResult {
    pub total_checked: usize,
    pub installed: usize,
    pub missing: usize,
    pub patches: Vec<PatchInfo>,
}

// ============ Registry Monitor Types (Windows) ============

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryKeyInfo {
    pub path: String,
    pub name: String,
    pub value: String,
    pub value_type: String,
    pub last_modified: String,
    pub security_issues: Vec<String>,
}

// ============ Common Constants ============

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
    "qwertyuiop",
    "passw0rd",
    "p@ssw0rd",
];

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

// ============ Existing Functions (keep as is) ============

pub fn is_password_weak(password: &str) -> (bool, String) {
    let password_lower = password.to_lowercase();
    if COMMON_WEAK_PASSWORDS.contains(&password_lower.as_str()) {
        return (
            true,
            "Password is in the list of common weak passwords".to_string(),
        );
    }
    if password.len() < 8 {
        return (
            true,
            "Password is too short (less than 8 characters)".to_string(),
        );
    }
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
    if password.len() >= 4 {
        for i in 0..password.len() - 3 {
            let substr = &password[i..i + 4];
            if password.matches(substr).count() > 1 {
                return (true, "Password contains repeated patterns".to_string());
            }
        }
    }
    let seq = "abcdefghijklmnopqrstuvwxyz0123456789";
    for i in 0..seq.len().saturating_sub(3) {
        if password_lower.contains(&seq[i..i + 3]) {
            return (true, "Password contains sequential characters".to_string());
        }
    }
    (false, "Password meets security requirements".to_string())
}

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

pub fn generate_password_dict(seed: &str, count: usize) -> Vec<String> {
    let mut dict: Vec<String> = Vec::new();
    let base = seed.to_lowercase();
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
    if seed.is_empty() || seed.len() < 3 {
        for pwd in COMMON_WEAK_PASSWORDS {
            dict.push(pwd.to_string());
        }
    }
    if !base.is_empty() {
        let mut capitalized = base.clone();
        if let Some(first) = capitalized.chars().next() {
            capitalized.remove(0);
            let cap = first.to_uppercase().to_string();
            dict.push(format!("{}{}", cap, capitalized));
        }
    }
    dict.truncate(count);
    dict
}

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

pub fn detect_phishing(url: &str) -> PhishingDetectionResult {
    let url_lower = url.to_lowercase();
    let mut reasons = Vec::new();
    let mut is_phishing = false;
    let mut confidence: f64 = 0.0;
    for (pattern, reason) in PHISHING_INDICATORS {
        if url_lower.contains(pattern) {
            reasons.push(format!("Contains suspicious keyword: {}", reason));
            confidence += 0.1;
        }
    }
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
    let shorteners = [
        "bit.ly", "tinyurl", "goo.gl", "shorturl", "rebrand", "is.gd",
    ];
    for shortener in &shorteners {
        if url_lower.contains(shortener) {
            reasons.push("URL uses a URL shortener service".to_string());
            confidence += 0.1;
        }
    }
    if !url_lower.starts_with("https://") && !url_lower.starts_with("http://") {
    } else if url_lower.starts_with("http://") && !url_lower.contains("localhost") {
        let domain_part = url_lower.split("://").nth(1).unwrap_or("");
        if !domain_part.starts_with("localhost") && !domain_part.starts_with("127.0.0.1") {
            reasons.push("URL uses insecure HTTP protocol".to_string());
            confidence += 0.05;
        }
    }
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

fn get_policy_current_value(policy_name: &str) -> String {
    match policy_name {
        "password_min_length" => "8".to_string(),
        "password_complexity" => "true".to_string(),
        "password_history" => "5".to_string(),
        "account_lockout_threshold" => "5".to_string(),
        "account_lockout_duration" => "15".to_string(),
        "session_timeout" => "30".to_string(),
        "mfa_required" => "false".to_string(),
        "audit_logging" => "true".to_string(),
        _ => "unknown".to_string(),
    }
}

// ============ New Functions ============

pub fn check_file_permissions(path: &str) -> PermissionCheckResult {
    let path_obj = Path::new(path);
    let mut issues = Vec::new();

    let exists = path_obj.exists();
    let readable = exists
        && path_obj
            .metadata()
            .map(|m| m.permissions().readonly())
            .unwrap_or(true);
    let writable = exists
        && fs::metadata(path)
            .map(|m| !m.permissions().readonly())
            .unwrap_or(false);
    let executable = exists
        && path_obj
            .metadata()
            .map(|m| m.permissions().readonly())
            .unwrap_or(true);

    let owner = if exists {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if let Ok(meta) = fs::metadata(path) {
                let uid = meta.uid();
                if let Some(user) = Users::new_with_refreshed_list()
                    .iter()
                    .find(|u| u.id() == uid)
                {
                    user.name().to_string()
                } else {
                    uid.to_string()
                }
            } else {
                "unknown".to_string()
            }
        }
        #[cfg(not(unix))]
        {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    };

    let group = if exists {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if let Ok(meta) = fs::metadata(path) {
                let gid = meta.gid();
                if let Some(user) = Users::new_with_refreshed_list()
                    .iter()
                    .find(|u| u.primary_group_id() == gid)
                {
                    user.name().to_string()
                } else {
                    gid.to_string()
                }
            } else {
                "unknown".to_string()
            }
        }
        #[cfg(not(unix))]
        {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    };

    let permissions = if exists {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = path_obj
                .metadata()
                .map(|m| m.permissions().mode())
                .unwrap_or(0);
            format!("{:o}", mode & 0o777)
        }
        #[cfg(not(unix))]
        {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    };

    if !exists {
        issues.push("Path does not exist".to_string());
    }
    if exists && !readable {
        issues.push("Not readable".to_string());
    }
    if exists && !writable {
        issues.push("Not writable".to_string());
    }

    PermissionCheckResult {
        path: path.to_string(),
        exists,
        readable,
        writable,
        executable,
        owner,
        group,
        permissions,
        issues,
    }
}

pub fn scan_permissions(path: &str, recursive: bool) -> PermissionScanResult {
    let mut results = Vec::new();
    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return PermissionScanResult {
            path: path.to_string(),
            total_files: 0,
            issues_found: 0,
            results: vec![],
        };
    }

    if path_obj.is_file() {
        results.push(check_file_permissions(path));
    } else if path_obj.is_dir() && recursive {
        for entry in walkdir::WalkDir::new(path_obj)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path().to_string_lossy().to_string();
            results.push(check_file_permissions(&file_path));
        }
    } else {
        results.push(check_file_permissions(path));
    }

    let issues_found = results.iter().filter(|r| !r.issues.is_empty()).count();

    PermissionScanResult {
        path: path.to_string(),
        total_files: results.len(),
        issues_found,
        results,
    }
}

pub fn check_account_security(username: &str) -> AccountSecurityResult {
    let mut issues: Vec<String> = Vec::new();
    #[cfg(unix)]
    {
        use std::fs;
        use std::io::BufRead;

        if let Ok(file) = fs::File::open("/etc/passwd") {
            let reader = std::io::BufReader::new(file);
            for line in reader.lines().filter_map(|l| l.ok()) {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 7 && parts[0] == username {
                    let uid = parts[2].parse::<u32>().unwrap_or(0);
                    let gid = parts[3].parse::<u32>().unwrap_or(0);
                    let home_dir = parts[5].to_string();
                    let shell = parts[6].to_string();
                    let is_root = uid == 0;
                    let is_system = uid < 1000;

                    let mut user_issues = Vec::new();
                    if is_root {
                        user_issues.push(
                            "Root account detected - consider using sudo instead".to_string(),
                        );
                    }
                    if is_system {
                        user_issues
                            .push("System account - ensure no login access is enabled".to_string());
                    }
                    return AccountSecurityResult {
                        username: username.to_string(),
                        uid,
                        gid,
                        home_dir,
                        shell,
                        password_expires: None,
                        account_locked: false,
                        password_empty: false,
                        is_root,
                        is_system,
                        issues: user_issues,
                    };
                }
            }
        }
    }
    #[cfg(windows)]
    {
        let cmd = std::process::Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "Get-LocalUser -Name '{}' | Select-Object Name, SID, Enabled, PasswordRequired",
                    username
                ),
            ])
            .output();
        if let Ok(output) = cmd {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if !output_str.contains("Cannot find") {
                let is_locked = !output_str.contains("True");
                return AccountSecurityResult {
                    username: username.to_string(),
                    uid: 0,
                    gid: 0,
                    home_dir: "".to_string(),
                    shell: "".to_string(),
                    password_expires: None,
                    account_locked: is_locked,
                    password_empty: output_str.contains("False"),
                    is_root: username.eq_ignore_ascii_case("Administrator"),
                    is_system: false,
                    issues: vec![],
                };
            }
        }
    }
    AccountSecurityResult {
        username: username.to_string(),
        uid: 0,
        gid: 0,
        home_dir: "".to_string(),
        shell: "".to_string(),
        password_expires: None,
        account_locked: true,
        password_empty: false,
        is_root: false,
        is_system: false,
        issues: vec!["User not found".to_string()],
    }
}

pub fn run_baseline_check() -> Vec<BaselineCheckResult> {
    let mut results = Vec::new();

    let policies = [
        ("Password Policy", "Minimum password length", "8", "8"),
        ("Password Policy", "Password complexity", "true", "true"),
        ("Account Policy", "Account lockout threshold", "5", "5"),
        ("Account Policy", "Account lockout duration", "15", "15"),
        ("Session Policy", "Session timeout", "30", "30"),
        ("Security Policy", "MFA enabled", "false", "true"),
        ("Security Policy", "Audit logging", "true", "true"),
        ("Security Policy", "Root login disabled", "true", "true"),
    ];

    for (category, name, current, expected) in policies {
        let compliant = current == expected;
        let severity = if !compliant && (name.contains("MFA") || name.contains("root")) {
            "high"
        } else if !compliant {
            "medium"
        } else {
            "low"
        };

        results.push(BaselineCheckResult {
            category: category.to_string(),
            check_name: name.to_string(),
            compliant,
            current_value: current.to_string(),
            expected_value: expected.to_string(),
            severity: severity.to_string(),
            recommendation: if compliant {
                "No action needed".to_string()
            } else {
                format!("Configure system to meet {} requirement", name)
            },
        });
    }

    results
}

pub fn check_network_shares() -> Vec<ShareInfo> {
    let mut shares = Vec::new();

    #[cfg(target_os = "windows")]
    {
        let cmd = std::process::Command::new("net").args(&["share"]).output();

        if let Ok(output) = cmd {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines().skip(4) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let name = parts[0].to_string();
                    let path = parts[1].to_string();
                    let desc = parts.get(2).unwrap_or(&"").to_string();
                    let mut issues = Vec::new();
                    if name.starts_with("ADMIN$")
                        || name.starts_with("IPC$")
                        || name.starts_with("C$")
                    {
                        issues.push("Administrative share exposed".to_string());
                    }
                    shares.push(ShareInfo {
                        name,
                        path,
                        description: desc,
                        shared: true,
                        read_only: false,
                        permissions: "Everyone".to_string(),
                        security_issues: issues,
                    });
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let cmd = std::process::Command::new("sh")
            .args(&["-c", "test -f /etc/exports && cat /etc/exports"])
            .output();

        if let Ok(output) = cmd {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if !line.is_empty() && !line.starts_with('#') {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(path) = parts.first() {
                        let mut issues = Vec::new();
                        if line.contains("*") || line.contains("rw,sync") {
                            issues.push("World-readable NFS export".to_string());
                        }
                        shares.push(ShareInfo {
                            name: "NFS Export".to_string(),
                            path: path.to_string(),
                            description: line.to_string(),
                            shared: true,
                            read_only: line.contains("ro"),
                            permissions: "Unknown".to_string(),
                            security_issues: issues,
                        });
                    }
                }
            }
        }

        let cmd = std::process::Command::new("sh")
            .args(&[
                "-c",
                "test -f /etc/samba/smb.conf && grep -E '^\\[.*\\]$' /etc/samba/smb.conf",
            ])
            .output();

        if let Ok(output) = cmd {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.starts_with('[') && !line.contains("global") {
                    let name = line.trim_matches('[').trim_matches(']').to_string();
                    shares.push(ShareInfo {
                        name,
                        path: "Unknown".to_string(),
                        description: "Samba share".to_string(),
                        shared: true,
                        read_only: false,
                        permissions: "Unknown".to_string(),
                        security_issues: vec![],
                    });
                }
            }
        }
    }

    shares
}

pub fn query_system_logs(filter: &str, max_entries: usize) -> LogQueryResult {
    let mut entries = Vec::new();

    #[cfg(not(target_os = "windows"))]
    {
        let args = if filter.is_empty() {
            vec!["-n", &max_entries.to_string()]
        } else {
            vec!["-n", &max_entries.to_string(), "|", "grep", filter]
        };
        let cmd = std::process::Command::new("journalctl")
            .args(&["-n", &max_entries.to_string()])
            .output();

        if let Ok(output) = cmd {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    entries.push(LogEntry {
                        timestamp: parts.get(0).unwrap_or(&"").to_string(),
                        host: parts.get(1).unwrap_or(&"").to_string(),
                        program: parts.get(2).unwrap_or(&"").to_string(),
                        pid: parts
                            .get(3)
                            .and_then(|s| s.trim_matches(':').parse::<u32>().ok()),
                        message: parts.get(4..).unwrap_or(&[]).join(" "),
                        severity: "info".to_string(),
                    });
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let cmd = std::process::Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "Get-WinEvent -MaxEvents {} -FilterXPath '*/System/EventID=4624' | ForEach-Object {{ $_.TimeCreated.ToString('yyyy-MM-dd HH:mm:ss') + ' ' + $_.Message }}",
                    max_entries
                ),
            ])
            .output();

        if let Ok(output) = cmd {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                entries.push(LogEntry {
                    timestamp: "".to_string(),
                    host: "localhost".to_string(),
                    program: "Security".to_string(),
                    pid: None,
                    message: line.to_string(),
                    severity: "info".to_string(),
                });
            }
        }
    }

    LogQueryResult {
        total_entries: entries.len(),
        entries,
        query: filter.to_string(),
    }
}

pub fn analyze_security_logs(time_range_hours: u64) -> Vec<String> {
    let mut findings = Vec::new();

    #[cfg(not(target_os = "windows"))]
    {
        let cmd = std::process::Command::new("journalctl")
            .args(&["--since", &format!("{} hours ago", time_range_hours)])
            .output();

        if let Ok(output) = cmd {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();

            let failed_logins = lines
                .iter()
                .filter(|l| l.contains("Failed password") || l.contains("authentication failure"))
                .count();

            if failed_logins > 0 {
                findings.push(format!(
                    "Found {} failed login attempts in the last {} hours",
                    failed_logins, time_range_hours
                ));
            }

            let sudo_events = lines
                .iter()
                .filter(|l| l.contains("sudo") && l.contains("COMMAND="))
                .count();

            if sudo_events > 0 {
                findings.push(format!("Found {} sudo commands executed", sudo_events));
            }

            let suspicious = lines
                .iter()
                .filter(|l| {
                    l.contains("Connection refused")
                        || l.contains("Failed password")
                        || l.contains("Invalid user")
                        || l.contains("authentication failure")
                        || l.contains("Permission denied")
                })
                .count();

            if suspicious > 0 {
                findings.push(format!("Found {} suspicious log entries", suspicious));
            }
        }
    }

    if findings.is_empty() {
        findings.push("No security issues found in the log analysis".to_string());
    }

    findings
}

pub fn check_persistence_mechanisms() -> Vec<PersistenceEntry> {
    let mut entries = Vec::new();

    #[cfg(not(target_os = "windows"))]
    {
        let paths = [
            ("~/.bashrc", "Shell startup file"),
            ("~/.profile", "Shell profile"),
            ("~/.ssh/authorized_keys", "SSH authorized keys"),
            ("/etc/rc.local", "System startup script"),
            ("/etc/cron.d", "Cron directory"),
            ("/etc/systemd/system", "Systemd services"),
            ("~/.config/autostart", "Desktop autostart"),
        ];

        for (path, desc) in paths {
            let expanded_path = shellexpand::tilde(path).to_string();
            if Path::new(&expanded_path).exists() {
                let suspicious = path.contains(".ssh") || path.contains("rc.local");
                entries.push(PersistenceEntry {
                    name: desc.to_string(),
                    path: expanded_path.clone(),
                    command: "".to_string(),
                    enabled: true,
                    source: "File system".to_string(),
                    suspicious,
                    reason: if suspicious {
                        "Potential persistence mechanism".to_string()
                    } else {
                        "Legitimate persistence".to_string()
                    },
                });
            }
        }

        let cmd = std::process::Command::new("sh")
            .args(&["-c", "crontab -l 2>/dev/null"])
            .output();

        if let Ok(output) = cmd {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if !line.is_empty() && !line.starts_with('#') {
                    entries.push(PersistenceEntry {
                        name: "Cron job".to_string(),
                        path: "Crontab".to_string(),
                        command: line.to_string(),
                        enabled: true,
                        source: "User crontab".to_string(),
                        suspicious: !line.contains("backup") && !line.contains("cleanup"),
                        reason: "Potential suspicious cron job".to_string(),
                    });
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let cmd = std::process::Command::new("reg")
            .args(&[
                "query",
                "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            ])
            .output();

        if let Ok(output) = cmd {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("REG_SZ") {
                    entries.push(PersistenceEntry {
                        name: "Registry Run".to_string(),
                        path: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string(),
                        command: line.to_string(),
                        enabled: true,
                        source: "Windows Registry".to_string(),
                        suspicious: false,
                        reason: "Startup entry".to_string(),
                    });
                }
            }
        }
    }

    entries
}

pub fn check_privilege_escalation() -> Vec<PrivilegeEscalationResult> {
    let mut results = Vec::new();

    #[cfg(not(target_os = "windows"))]
    {
        let checks = [
            ("SUID Binaries", "find / -perm -4000 -type f 2>/dev/null"),
            ("SGID Binaries", "find / -perm -2000 -type f 2>/dev/null"),
            ("Sudo rights", "sudo -l 2>/dev/null"),
            ("Writeable system files", "find /etc -writable 2>/dev/null"),
            ("Docker socket", "test -S /var/run/docker.sock"),
        ];

        for (name, cmd) in checks {
            let output = std::process::Command::new("sh").args(&["-c", cmd]).output();

            if let Ok(output) = output {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let vulnerable = !output_str.is_empty() && !output_str.contains("not allowed");

                results.push(PrivilegeEscalationResult {
                    check_name: name.to_string(),
                    vulnerable,
                    description: format!("Checking for {}", name),
                    details: if output_str.is_empty() {
                        "No findings".to_string()
                    } else {
                        output_str.lines().take(5).collect::<Vec<_>>().join("\n")
                    },
                    severity: if vulnerable { "high" } else { "low" }.to_string(),
                });
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let checks = [
            ("Admin Group", "net localgroup administrators"),
            (
                "UAC Status",
                "reg query HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\System",
            ),
        ];

        for (name, cmd) in checks {
            let output = std::process::Command::new("cmd")
                .args(&["/C", cmd])
                .output();

            if let Ok(output) = output {
                let output_str = String::from_utf8_lossy(&output.stdout);
                results.push(PrivilegeEscalationResult {
                    check_name: name.to_string(),
                    vulnerable: name == "Admin Group" && output_str.contains("Administrator"),
                    description: format!("Checking for {}", name),
                    details: output_str.lines().take(5).collect::<Vec<_>>().join("\n"),
                    severity: if name == "Admin Group" {
                        "high"
                    } else {
                        "medium"
                    }
                    .to_string(),
                });
            }
        }
    }

    results
}

pub fn check_patch_status() -> PatchScanResult {
    let mut patches = Vec::new();

    #[cfg(not(target_os = "windows"))]
    {
        let cmd = std::process::Command::new("sh")
            .args(&["-c", "apt list --upgradable 2>/dev/null || yum check-update 2>/dev/null || dnf check-update 2>/dev/null"])
            .output();

        if let Ok(output) = cmd {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if !line.is_empty()
                    && !line.starts_with("Loading")
                    && !line.starts_with("Available")
                {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(name) = parts.first() {
                        patches.push(PatchInfo {
                            name: name.to_string(),
                            installed: false,
                            version: parts.get(1).unwrap_or(&"").to_string(),
                            release_date: None,
                            severity: "medium".to_string(),
                            description: "Security update available".to_string(),
                        });
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let cmd = std::process::Command::new("powershell")
            .args(&[
                "-Command",
                "Get-WindowsUpdate -IsInstalled | Select-Object -First 20",
            ])
            .output();

        if let Ok(output) = cmd {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if !line.is_empty() && !line.contains("KB") {
                    patches.push(PatchInfo {
                        name: line.to_string(),
                        installed: true,
                        version: "".to_string(),
                        release_date: None,
                        severity: "low".to_string(),
                        description: "Windows update installed".to_string(),
                    });
                }
            }
        }
    }

    let total_checked = patches.len();
    let installed = patches.iter().filter(|p| p.installed).count();
    let missing = patches.iter().filter(|p| !p.installed).count();

    PatchScanResult {
        total_checked,
        installed,
        missing,
        patches,
    }
}

#[cfg(target_os = "windows")]
pub fn monitor_registry_key(path: &str) -> RegistryKeyInfo {
    let cmd = std::process::Command::new("reg")
        .args(&["query", path])
        .output();

    let mut issues = Vec::new();
    let name = path.split('\\').last().unwrap_or(path);

    if let Ok(output) = cmd {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let value = output_str.lines().next().unwrap_or("No data").to_string();

        if path.contains("Run") {
            issues.push("Startup registry key - potential persistence".to_string());
        }
        if path.contains("Services") {
            issues.push("Service registry key - requires admin privileges".to_string());
        }

        RegistryKeyInfo {
            path: path.to_string(),
            name: name.to_string(),
            value,
            value_type: "REG_SZ".to_string(),
            last_modified: "".to_string(),
            security_issues: issues,
        }
    } else {
        RegistryKeyInfo {
            path: path.to_string(),
            name: name.to_string(),
            value: "Key not found".to_string(),
            value_type: "unknown".to_string(),
            last_modified: "".to_string(),
            security_issues: vec!["Registry key not accessible".to_string()],
        }
    }
}
