//! Chain mode prompt template with categories filtering

use crate::prompts::get_identity_intro;

/// Build chain mode prompt with filtered skills
pub fn build_chain_prompt_with_categories(filtered_skills: &str, input: &str) -> String {
    let identity_intro = get_identity_intro();

    format!(
        r#"{}

## Available Atomic Skills
{}

## Response Format
{{"mode": "chain", "steps": [
  {{"action": "skill1", "parameters": {{}}, "output_as": "result1"}},
  {{"action": "skill2", "parameters": {{"input": "{{{{result1}}}}"}}}}
]}}

## User Input
{}
"#,
        identity_intro, filtered_skills, input
    )
}
