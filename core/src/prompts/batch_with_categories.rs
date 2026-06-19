//! Batch mode prompt template with categories filtering

use crate::prompts::get_identity_intro;

/// Build batch mode prompt with filtered drivers
pub fn build_batch_prompt_with_categories(filtered_drivers: &str, input: &str) -> String {
    let identity_intro = get_identity_intro();

    format!(
        r#"{}


## Available Atomic Skills
{}

## Response Format
{{"mode": "batch", "steps": [{{"action": "skill1", "parameters": {{}}}}]}}

If no skills are needed:
{{"action": "done", "message": "Your answer"}}

## User Input
{}
"#,
        identity_intro, filtered_drivers, input
    )
}
