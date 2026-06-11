//! Plan-And-Execute mode prompt template with categories filtering

use crate::prompts::{generate_instances_registry, get_identity_intro};

/// Build plan-and-execute mode prompt with filtered skills
pub fn build_plan_prompt_with_categories(filtered_skills: &str, input: &str) -> String {
    let instances_registry = generate_instances_registry();
    let identity_intro = get_identity_intro();

    format!(
        r#"{}

## Available Atomic Skills
{}

## Available Instances
{}

## Response Format
{{"mode": "plan", "plan": {{"steps": [
  {{"id": "step1", "action": "skill_name", "parameters": {{}}, "output_as": "result1"}}
]}}}}

If no skills are needed:
{{"mode": "done", "message": "Your answer"}}

## User Input
{}
"#,
        identity_intro, filtered_skills, instances_registry, input
    )
}