use hippox_atomic_skills::Skill;
use hippox_atomic_skills::skill_registry::get_registry;
use std::sync::Arc;

pub fn get_skill(name: &str) -> Option<Arc<dyn Skill>> {
    get_registry().get(name).cloned()
}

pub fn list_skills() -> Vec<String> {
    get_registry().keys().cloned().collect()
}
