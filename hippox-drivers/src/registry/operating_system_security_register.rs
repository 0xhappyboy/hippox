//! Operating system security drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::OperatingSystemSecurity;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "operating_system_security", feature = "all"))]
    {
        use crate::drivers::{
            AccountSecurityDriver, BaselineCheckDriver, CveQueryDriver, PatchDetectDriver,
            PermissionCheckDriver, PersistenceDetectDriver, PhishingDetectDriver,
            PrivilegeEscalationDetectDriver, RegistryMonitorDriver, SecurityLogAnalyzeDriver,
            SecurityPolicyCheckDriver, ShareCheckDriver, SyslogQueryDriver, ThreatIntelDriver,
            WeakPasswordCheckDriver,
        };

        // Existing
        map.insert(
            "security_weak_password_check".to_string(),
            Arc::new(WeakPasswordCheckDriver),
        );
        map.insert(
            "security_policy_check".to_string(),
            Arc::new(SecurityPolicyCheckDriver),
        );
        map.insert("security_cve_query".to_string(), Arc::new(CveQueryDriver));
        map.insert(
            "security_threat_intel".to_string(),
            Arc::new(ThreatIntelDriver),
        );
        map.insert(
            "security_phishing_detect".to_string(),
            Arc::new(PhishingDetectDriver),
        );

        // New drivers
        map.insert(
            "security_permission_check".to_string(),
            Arc::new(PermissionCheckDriver),
        );
        map.insert(
            "security_account_check".to_string(),
            Arc::new(AccountSecurityDriver),
        );
        map.insert(
            "security_baseline_check".to_string(),
            Arc::new(BaselineCheckDriver),
        );
        map.insert(
            "security_share_check".to_string(),
            Arc::new(ShareCheckDriver),
        );
        map.insert(
            "security_syslog_query".to_string(),
            Arc::new(SyslogQueryDriver),
        );
        map.insert(
            "security_log_analyze".to_string(),
            Arc::new(SecurityLogAnalyzeDriver),
        );
        map.insert(
            "security_persistence_detect".to_string(),
            Arc::new(PersistenceDetectDriver),
        );
        map.insert(
            "security_privilege_escalation_detect".to_string(),
            Arc::new(PrivilegeEscalationDetectDriver),
        );
        map.insert(
            "security_patch_detect".to_string(),
            Arc::new(PatchDetectDriver),
        );
        #[cfg(target_os = "windows")]
        map.insert(
            "security_registry_monitor".to_string(),
            Arc::new(RegistryMonitorDriver),
        );
        #[cfg(not(target_os = "windows"))]
        map.insert(
            "security_registry_monitor".to_string(),
            Arc::new(RegistryMonitorDriver),
        );
    }
}
