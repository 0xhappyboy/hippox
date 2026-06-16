//! Scheduled tasks skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::ScheduledTasks;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "scheduled_tasks", feature = "all"))]
    {
        use crate::skills::{ListScheduledTasksSkill, ScheduleTaskSkill, UnscheduleTaskSkill};

        map.insert("schedule_task".to_string(), Arc::new(ScheduleTaskSkill));
        map.insert("unschedule_task".to_string(), Arc::new(UnscheduleTaskSkill));
        map.insert(
            "list_scheduled_tasks".to_string(),
            Arc::new(ListScheduledTasksSkill),
        );
    }
}
