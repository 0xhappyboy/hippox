//! Batch mode prompt templates

use crate::prompts::{generate_drivers_registry, get_identity_intro};

pub fn build_batch_prompt(input: &str) -> String {
    let drivers_registry = generate_drivers_registry();
    let identity_intro = get_identity_intro();

    format!(
        r#"{} 

[SKILLS]
{}

[FORMAT]
{{"mode": "batch", "steps": [{{"action": "skill1", "parameters": {{}}}}]}}
If no skills: {{"action": "done", "message": "..."}}

[USER]
{}
"#,
        identity_intro, drivers_registry, input
    )
}
