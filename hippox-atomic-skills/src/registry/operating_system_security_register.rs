//! Operating system security skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::OperatingSystemSecurity;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "operating_system_security", feature = "all"))]
    {
        use crate::skills::{
            AccountSecuritySkill, BaselineCheckSkill, CveQuerySkill, PatchDetectSkill,
            PermissionCheckSkill, PersistenceDetectSkill, PhishingDetectSkill,
            PrivilegeEscalationDetectSkill, RegistryMonitorSkill, SecurityLogAnalyzeSkill,
            SecurityPolicyCheckSkill, ShareCheckSkill, SyslogQuerySkill, ThreatIntelSkill,
            WeakPasswordCheckSkill,
        };

        // Existing
        map.insert(
            "security_weak_password_check".to_string(),
            Arc::new(WeakPasswordCheckSkill),
        );
        map.insert(
            "security_policy_check".to_string(),
            Arc::new(SecurityPolicyCheckSkill),
        );
        map.insert("security_cve_query".to_string(), Arc::new(CveQuerySkill));
        map.insert(
            "security_threat_intel".to_string(),
            Arc::new(ThreatIntelSkill),
        );
        map.insert(
            "security_phishing_detect".to_string(),
            Arc::new(PhishingDetectSkill),
        );

        // New skills
        map.insert(
            "security_permission_check".to_string(),
            Arc::new(PermissionCheckSkill),
        );
        map.insert(
            "security_account_check".to_string(),
            Arc::new(AccountSecuritySkill),
        );
        map.insert(
            "security_baseline_check".to_string(),
            Arc::new(BaselineCheckSkill),
        );
        map.insert(
            "security_share_check".to_string(),
            Arc::new(ShareCheckSkill),
        );
        map.insert(
            "security_syslog_query".to_string(),
            Arc::new(SyslogQuerySkill),
        );
        map.insert(
            "security_log_analyze".to_string(),
            Arc::new(SecurityLogAnalyzeSkill),
        );
        map.insert(
            "security_persistence_detect".to_string(),
            Arc::new(PersistenceDetectSkill),
        );
        map.insert(
            "security_privilege_escalation_detect".to_string(),
            Arc::new(PrivilegeEscalationDetectSkill),
        );
        map.insert(
            "security_patch_detect".to_string(),
            Arc::new(PatchDetectSkill),
        );
        #[cfg(target_os = "windows")]
        map.insert(
            "security_registry_monitor".to_string(),
            Arc::new(RegistryMonitorSkill),
        );
        #[cfg(not(target_os = "windows"))]
        map.insert(
            "security_registry_monitor".to_string(),
            Arc::new(RegistryMonitorSkill),
        );
    }
}
