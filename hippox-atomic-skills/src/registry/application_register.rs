//! Application control skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Application;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    #[cfg(any(feature = "application_control", feature = "all"))]
    {
        use crate::skills::application_control::*;
        map.insert(
            "application_control_launch".to_string(),
            Arc::new(ApplicationControlLaunchSkill),
        );
        map.insert(
            "application_control_launch_with_args".to_string(),
            Arc::new(ApplicationControlLaunchWithArgsSkill),
        );
        map.insert(
            "application_control_launch_as_admin".to_string(),
            Arc::new(ApplicationControlLaunchAsAdminSkill),
        );
        map.insert(
            "application_control_close".to_string(),
            Arc::new(ApplicationControlCloseSkill),
        );
        map.insert(
            "application_control_is_running".to_string(),
            Arc::new(ApplicationControlIsRunningSkill),
        );
        map.insert(
            "application_control_wait_for_exit".to_string(),
            Arc::new(ApplicationControlWaitForExitSkill),
        );
        map.insert(
            "application_control_get_path".to_string(),
            Arc::new(ApplicationControlGetPathSkill),
        );
        map.insert(
            "application_control_wait_for".to_string(),
            Arc::new(ApplicationControlWaitForSkill),
        );
        map.insert(
            "application_control_restart".to_string(),
            Arc::new(ApplicationControlRestartSkill),
        );
        map.insert(
            "application_control_list_running".to_string(),
            Arc::new(ApplicationControlListRunningSkill),
        );
        map.insert(
            "application_control_install".to_string(),
            Arc::new(ApplicationControlInstallSkill),
        );
        map.insert(
            "application_control_uninstall".to_string(),
            Arc::new(ApplicationControlUninstallSkill),
        );
    }
}
