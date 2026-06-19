//! Application control drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Application;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    #[cfg(any(feature = "application_control", feature = "all"))]
    {
        use crate::drivers::application_control::*;
        map.insert(
            "application_control_launch".to_string(),
            Arc::new(ApplicationControlLaunchDriver),
        );
        map.insert(
            "application_control_launch_with_args".to_string(),
            Arc::new(ApplicationControlLaunchWithArgsDriver),
        );
        map.insert(
            "application_control_launch_as_admin".to_string(),
            Arc::new(ApplicationControlLaunchAsAdminDriver),
        );
        map.insert(
            "application_control_close".to_string(),
            Arc::new(ApplicationControlCloseDriver),
        );
        map.insert(
            "application_control_is_running".to_string(),
            Arc::new(ApplicationControlIsRunningDriver),
        );
        map.insert(
            "application_control_wait_for_exit".to_string(),
            Arc::new(ApplicationControlWaitForExitDriver),
        );
        map.insert(
            "application_control_get_path".to_string(),
            Arc::new(ApplicationControlGetPathDriver),
        );
        map.insert(
            "application_control_wait_for".to_string(),
            Arc::new(ApplicationControlWaitForDriver),
        );
        map.insert(
            "application_control_restart".to_string(),
            Arc::new(ApplicationControlRestartDriver),
        );
        map.insert(
            "application_control_list_running".to_string(),
            Arc::new(ApplicationControlListRunningDriver),
        );
        map.insert(
            "application_control_install".to_string(),
            Arc::new(ApplicationControlInstallDriver),
        );
        map.insert(
            "application_control_uninstall".to_string(),
            Arc::new(ApplicationControlUninstallDriver),
        );
    }
}
