// prompts/skill_md.rs
use crate::prompts::{generate_instances_registry, generate_skills_registry};

pub fn build_skill_md_prompt(instructions: &str) -> String {
    let skills_registry = generate_skills_registry();
    let instances_registry = generate_instances_registry();

    format!(
        "{}\n\n## Available Atomic Skills\n{}\n\n## Available Instances\n{}\n\n## Task\nExecute the workflow step by step according to the instructions above.",
        instructions, skills_registry, instances_registry
    )
}
