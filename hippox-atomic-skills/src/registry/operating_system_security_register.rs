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
            CveQuerySkill, PhishingDetectSkill, SecurityPolicyCheckSkill, ThreatIntelSkill,
            WeakPasswordCheckSkill,
        };
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
    }
}
