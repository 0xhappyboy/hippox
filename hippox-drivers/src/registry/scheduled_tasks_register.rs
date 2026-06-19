//! Scheduled tasks drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::ScheduledTasks;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "scheduled_tasks", feature = "all"))]
    {
        use crate::drivers::{ListScheduledTasksDriver, ScheduleTaskDriver, UnscheduleTaskDriver};

        map.insert("schedule_task".to_string(), Arc::new(ScheduleTaskDriver));
        map.insert("unschedule_task".to_string(), Arc::new(UnscheduleTaskDriver));
        map.insert(
            "list_scheduled_tasks".to_string(),
            Arc::new(ListScheduledTasksDriver),
        );
    }
}
