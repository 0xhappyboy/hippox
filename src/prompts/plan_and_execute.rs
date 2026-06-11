//! Plan-And-Execute mode prompt templates

use crate::prompts::{generate_instances_registry, generate_skills_registry, get_identity_intro};

pub fn build_plan_prompt(input: &str) -> String {
    let skills_registry = generate_skills_registry();
    let instances_registry = generate_instances_registry();
    let identity_intro = get_identity_intro();

    format!(
        r#"{} 

[SKILLS]
{}

[INSTANCES]
{}

[FORMAT]
{{"mode": "plan", "plan": {{"steps": [{{"id": "s1", "action": "name", "parameters": {{}}, "output_as": "r1"}}]}}}}
If no skills: {{"mode": "done", "message": "..."}}

[USER]
{}
"#,
        identity_intro, skills_registry, instances_registry, input
    )
}
