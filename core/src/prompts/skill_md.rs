use crate::prompts::generate_skills_registry;

pub fn build_skill_md_prompt(instructions: &str) -> String {
    let skills_registry = generate_skills_registry();

    format!(
        "{}\n\n## Available Atomic Skills\n{}\n\n## Task\nExecute the workflow step by step according to the instructions above.",
        instructions, skills_registry
    )
}
