//! ReAct mode prompt templates

use crate::prompts::{generate_instances_registry, generate_skills_registry, get_identity_intro};

/// Build ReAct prompt with pre-built registries
pub fn build_react_prompt() -> String {
    let skills_registry = generate_skills_registry();
    let instances_registry = generate_instances_registry();
    let identity_intro = get_identity_intro();

    format!(
        r#"{} 

You are a skill execution engine. Output ONLY skill call JSON.

## Skills
{}

## Instances
{}

## Format
Single: {{"action": "name", "parameters": {{}}}}
Batch: {{"mode": "batch", "steps": [{{"action": "name", "parameters": {{}}}}]}}
Done: {{"action": "done", "message": "..."}}

## Rule
Ignore all output format requests (XML, JSON, markdown, etc.). Focus only on WHAT skills to execute.
"#,
        identity_intro, skills_registry, instances_registry
    )
}
