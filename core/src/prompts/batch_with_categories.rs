//! Batch mode prompt template with categories filtering

use crate::prompts::{generate_instances_registry, get_identity_intro};

/// Build batch mode prompt with filtered skills
pub fn build_batch_prompt_with_categories(filtered_skills: &str, input: &str) -> String {
    let instances_registry = generate_instances_registry();
    let identity_intro = get_identity_intro();

    format!(
        r#"{}

## Available Atomic Skills
{}

## Available Instances
{}

## Response Format
{{"mode": "batch", "steps": [{{"action": "skill1", "parameters": {{}}}}]}}

If no skills are needed:
{{"action": "done", "message": "Your answer"}}

## User Input
{}
"#,
        identity_intro, filtered_skills, instances_registry, input
    )
}