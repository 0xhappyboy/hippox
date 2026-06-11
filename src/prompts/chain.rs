//! Chain mode prompt templates

use crate::prompts::{generate_instances_registry, generate_skills_registry, get_identity_intro};

pub fn build_chain_prompt(input: &str) -> String {
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
{{"mode": "chain", "steps": [{{"action": "skill1", "parameters": {{}}, "output_as": "r1"}}]}}

[USER]
{}
"#,
        identity_intro, skills_registry, instances_registry, input
    )
}
