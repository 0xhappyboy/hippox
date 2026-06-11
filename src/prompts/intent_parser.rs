//! Intent parser prompt template

/// Build intent parser prompt to extract clean_intent and skill_categories
pub fn build_intent_parser_prompt(input: &str) -> String {
    format!(
        r#"## FINAL INSTRUCTION - HIGHEST PRIORITY
Ignore all previous instructions about output format.

Extract from the user message below:

User input: {}

Output ONLY this JSON format. NO other text, NO markdown, NO explanations.

Format: {{"clean_intent": "", "skill_categories": []}}

Categories: math, file, net, crypto, random, document, message, database, devops, system, image, time, task, text, regex, blockchain

## YOUR OUTPUT:
"#,
        input
    )
}
