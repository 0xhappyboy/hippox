//! Plan-And-Execute mode prompt template with categories filtering

use crate::prompts::get_identity_intro;

/// Build plan-and-execute mode prompt with filtered drivers
pub fn build_plan_prompt_with_categories(filtered_drivers: &str, input: &str) -> String {
    let identity_intro = get_identity_intro();

    format!(
        r#"{}

## Available Atomic Skills
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
        identity_intro, filtered_drivers, input
    )
}
