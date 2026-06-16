use hippox_atomic_skills::{Skill, get_skill_by_name, list_skills_names};
use std::sync::Arc;

pub fn get_skill(name: &str) -> Option<Arc<dyn Skill>> {
    get_skill_by_name(name)
}

pub fn list_skills() -> Vec<String> {
    list_skills_names()
}
