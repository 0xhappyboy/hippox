//! Operating system services drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::OperatingSystemServices;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    #[cfg(any(feature = "operating_system_services", feature = "all"))]
    {
        use crate::drivers::{
            ServiceAllDriver, ServiceConfigPathDriver, ServiceCopyDriver, ServiceDependenciesDriver, ServiceDisableDriver, ServiceEnableDriver, ServiceEnabledListDriver, ServiceEnvDriver, ServiceExportDriver, ServiceFailureActionDriver, ServiceFailureCountDriver, ServiceHistoryDriver, ServiceImportDriver, ServiceListDriver, ServiceLockDriver, ServiceLogsDriver, ServiceMaskDriver, ServiceMaskedListDriver, ServicePidDriver, ServiceRecentDriver, ServiceReloadDriver, ServiceRenameDriver, ServiceResetFailureCountDriver, ServiceResourcesDriver, ServiceRestartDriver, ServiceReverseDependenciesDriver, ServiceRunningDriver, ServiceSearchDriver, ServiceSecurityDriver, ServiceSetEnvDriver, ServiceSetTimeoutDriver, ServiceStartDriver, ServiceStartTypeDriver, ServiceStatusDriver, ServiceStdoutDriver, ServiceStopDriver, ServiceUnlockDriver, ServiceUnmaskDriver, ServiceUptimeDriver, ServiceUserDriver
        };
        map.insert("service_list".to_string(), Arc::new(ServiceListDriver));
        map.insert("service_start".to_string(), Arc::new(ServiceStartDriver));
        map.insert("service_stop".to_string(), Arc::new(ServiceStopDriver));
        map.insert("service_restart".to_string(), Arc::new(ServiceRestartDriver));
        map.insert("service_status".to_string(), Arc::new(ServiceStatusDriver));
        map.insert("service_enable".to_string(), Arc::new(ServiceEnableDriver));
        map.insert("service_disable".to_string(), Arc::new(ServiceDisableDriver));
        map.insert(
            "service_dependencies".to_string(),
            Arc::new(ServiceDependenciesDriver),
        );
        map.insert(
            "service_reverse_dependencies".to_string(),
            Arc::new(ServiceReverseDependenciesDriver),
        );
        map.insert("service_logs".to_string(), Arc::new(ServiceLogsDriver));
        map.insert(
            "service_config_path".to_string(),
            Arc::new(ServiceConfigPathDriver),
        );
        map.insert("service_reload".to_string(), Arc::new(ServiceReloadDriver));
        map.insert("service_uptime".to_string(), Arc::new(ServiceUptimeDriver));
        map.insert(
            "service_resources".to_string(),
            Arc::new(ServiceResourcesDriver),
        );
        map.insert("service_pid".to_string(), Arc::new(ServicePidDriver));
        map.insert("service_user".to_string(), Arc::new(ServiceUserDriver));
        map.insert(
            "service_start_type".to_string(),
            Arc::new(ServiceStartTypeDriver),
        );
        map.insert(
            "service_set_timeout".to_string(),
            Arc::new(ServiceSetTimeoutDriver),
        );
        map.insert(
            "service_failure_action".to_string(),
            Arc::new(ServiceFailureActionDriver),
        );
        map.insert(
            "service_failure_count".to_string(),
            Arc::new(ServiceFailureCountDriver),
        );
        map.insert(
            "service_reset_failure_count".to_string(),
            Arc::new(ServiceResetFailureCountDriver),
        );
        map.insert("service_env".to_string(), Arc::new(ServiceEnvDriver));
        map.insert("service_set_env".to_string(), Arc::new(ServiceSetEnvDriver));
        map.insert("service_stdout".to_string(), Arc::new(ServiceStdoutDriver));
        map.insert("service_mask".to_string(), Arc::new(ServiceMaskDriver));
        map.insert("service_unmask".to_string(), Arc::new(ServiceUnmaskDriver));
        map.insert(
            "service_masked_list".to_string(),
            Arc::new(ServiceMaskedListDriver),
        );
        map.insert("service_all".to_string(), Arc::new(ServiceAllDriver));
        map.insert("service_running".to_string(), Arc::new(ServiceRunningDriver));
        map.insert(
            "service_enabled_list".to_string(),
            Arc::new(ServiceEnabledListDriver),
        );
        map.insert("service_recent".to_string(), Arc::new(ServiceRecentDriver));
        map.insert("service_search".to_string(), Arc::new(ServiceSearchDriver));
        map.insert("service_export".to_string(), Arc::new(ServiceExportDriver));
        map.insert("service_import".to_string(), Arc::new(ServiceImportDriver));
        map.insert("service_copy".to_string(), Arc::new(ServiceCopyDriver));
        map.insert("service_rename".to_string(), Arc::new(ServiceRenameDriver));
        map.insert("service_history".to_string(), Arc::new(ServiceHistoryDriver));
        map.insert("service_lock".to_string(), Arc::new(ServiceLockDriver));
        map.insert("service_unlock".to_string(), Arc::new(ServiceUnlockDriver));
        map.insert(
            "service_security".to_string(),
            Arc::new(ServiceSecurityDriver),
        );
    }
}
