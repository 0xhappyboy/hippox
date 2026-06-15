//! Plan-And-Execute mode prompt templates

use crate::prompts::{generate_skills_registry, get_identity_intro};

pub fn build_plan_prompt(input: &str) -> String {
    let skills_registry = generate_skills_registry();
    let identity_intro = get_identity_intro();

    format!(
        r#"{} 

[SKILLS]
{}

[FORMAT]
{{"mode": "plan", "plan": {{"steps": [{{"id": "s1", "action": "name", "parameters": {{}}, "output_as": "r1"}}]}}}}
If no skills: {{"mode": "done", "message": "..."}}

[USER]
{}
"#,
        identity_intro, skills_registry, input
    )
}
