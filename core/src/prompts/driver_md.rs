use crate::prompts::generate_drivers_registry;

pub fn build_driver_md_prompt(instructions: &str) -> String {
    let drivers_registry = generate_drivers_registry();

    format!(
        "{}\n\n## Available Atomic Skills\n{}\n\n## Task\nExecute the workflow step by step according to the instructions above.",
        instructions, drivers_registry
    )
}
