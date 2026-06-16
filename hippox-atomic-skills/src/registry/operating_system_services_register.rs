//! Operating system services skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::OperatingSystemServices;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    #[cfg(any(feature = "operating_system_services", feature = "all"))]
    {
        use crate::skills::{
            ServiceAllSkill, ServiceConfigPathSkill, ServiceCopySkill, ServiceDependenciesSkill, ServiceDisableSkill, ServiceEnableSkill, ServiceEnabledListSkill, ServiceEnvSkill, ServiceExportSkill, ServiceFailureActionSkill, ServiceFailureCountSkill, ServiceHistorySkill, ServiceImportSkill, ServiceListSkill, ServiceLockSkill, ServiceLogsSkill, ServiceMaskSkill, ServiceMaskedListSkill, ServicePidSkill, ServiceRecentSkill, ServiceReloadSkill, ServiceRenameSkill, ServiceResetFailureCountSkill, ServiceResourcesSkill, ServiceRestartSkill, ServiceReverseDependenciesSkill, ServiceRunningSkill, ServiceSearchSkill, ServiceSecuritySkill, ServiceSetEnvSkill, ServiceSetTimeoutSkill, ServiceStartSkill, ServiceStartTypeSkill, ServiceStatusSkill, ServiceStdoutSkill, ServiceStopSkill, ServiceUnlockSkill, ServiceUnmaskSkill, ServiceUptimeSkill, ServiceUserSkill
        };
        map.insert("service_list".to_string(), Arc::new(ServiceListSkill));
        map.insert("service_start".to_string(), Arc::new(ServiceStartSkill));
        map.insert("service_stop".to_string(), Arc::new(ServiceStopSkill));
        map.insert("service_restart".to_string(), Arc::new(ServiceRestartSkill));
        map.insert("service_status".to_string(), Arc::new(ServiceStatusSkill));
        map.insert("service_enable".to_string(), Arc::new(ServiceEnableSkill));
        map.insert("service_disable".to_string(), Arc::new(ServiceDisableSkill));
        map.insert(
            "service_dependencies".to_string(),
            Arc::new(ServiceDependenciesSkill),
        );
        map.insert(
            "service_reverse_dependencies".to_string(),
            Arc::new(ServiceReverseDependenciesSkill),
        );
        map.insert("service_logs".to_string(), Arc::new(ServiceLogsSkill));
        map.insert(
            "service_config_path".to_string(),
            Arc::new(ServiceConfigPathSkill),
        );
        map.insert("service_reload".to_string(), Arc::new(ServiceReloadSkill));
        map.insert("service_uptime".to_string(), Arc::new(ServiceUptimeSkill));
        map.insert(
            "service_resources".to_string(),
            Arc::new(ServiceResourcesSkill),
        );
        map.insert("service_pid".to_string(), Arc::new(ServicePidSkill));
        map.insert("service_user".to_string(), Arc::new(ServiceUserSkill));
        map.insert(
            "service_start_type".to_string(),
            Arc::new(ServiceStartTypeSkill),
        );
        map.insert(
            "service_set_timeout".to_string(),
            Arc::new(ServiceSetTimeoutSkill),
        );
        map.insert(
            "service_failure_action".to_string(),
            Arc::new(ServiceFailureActionSkill),
        );
        map.insert(
            "service_failure_count".to_string(),
            Arc::new(ServiceFailureCountSkill),
        );
        map.insert(
            "service_reset_failure_count".to_string(),
            Arc::new(ServiceResetFailureCountSkill),
        );
        map.insert("service_env".to_string(), Arc::new(ServiceEnvSkill));
        map.insert("service_set_env".to_string(), Arc::new(ServiceSetEnvSkill));
        map.insert("service_stdout".to_string(), Arc::new(ServiceStdoutSkill));
        map.insert("service_mask".to_string(), Arc::new(ServiceMaskSkill));
        map.insert("service_unmask".to_string(), Arc::new(ServiceUnmaskSkill));
        map.insert(
            "service_masked_list".to_string(),
            Arc::new(ServiceMaskedListSkill),
        );
        map.insert("service_all".to_string(), Arc::new(ServiceAllSkill));
        map.insert("service_running".to_string(), Arc::new(ServiceRunningSkill));
        map.insert(
            "service_enabled_list".to_string(),
            Arc::new(ServiceEnabledListSkill),
        );
        map.insert("service_recent".to_string(), Arc::new(ServiceRecentSkill));
        map.insert("service_search".to_string(), Arc::new(ServiceSearchSkill));
        map.insert("service_export".to_string(), Arc::new(ServiceExportSkill));
        map.insert("service_import".to_string(), Arc::new(ServiceImportSkill));
        map.insert("service_copy".to_string(), Arc::new(ServiceCopySkill));
        map.insert("service_rename".to_string(), Arc::new(ServiceRenameSkill));
        map.insert("service_history".to_string(), Arc::new(ServiceHistorySkill));
        map.insert("service_lock".to_string(), Arc::new(ServiceLockSkill));
        map.insert("service_unlock".to_string(), Arc::new(ServiceUnlockSkill));
        map.insert(
            "service_security".to_string(),
            Arc::new(ServiceSecuritySkill),
        );
    }
}
