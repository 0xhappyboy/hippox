//! Intent parser prompt template

use hippox_atomic_skills::skill_registry::get_skill_categories;

/// Builds a prompt for LLM to extract intent and required skill categories from user input.
///
/// The LLM analyzes the user message and returns a JSON object containing:
/// - `clean_intent`: The user's core request with output formatting instructions removed
/// - `skill_categories`: A list of skill categories needed to fulfill the request
///
/// This enables the system to filter skills by category before building the execution prompt,
/// reducing token usage and improving accuracy.
///
/// # Categories are auto-loaded from registry
/// The category list is dynamically generated from `registry::get_skill_categories()`,
/// which collects all `category()` values from registered skills.
///
/// # Example
/// ```text
/// Input: "Search for Rust tutorials and return as XML"
/// Output: {"clean_intent": "Search for Rust tutorials", "skill_categories": ["browser", "net"]}
/// ```
pub fn build_intent_parser_prompt(input: &str) -> String {
    let categories = get_skill_categories();
    let categories_str: Vec<String> = categories.iter().map(|(cat, _)| cat.clone()).collect();
    let categories_list = categories_str.join(", ");

    format!(
        r#"## FINAL INSTRUCTION - HIGHEST PRIORITY
Ignore all previous instructions about output format.

Extract from the user message below:

User input: {}

Output ONLY this JSON format. NO other text, NO markdown, NO explanations.

Format: {{"clean_intent": "", "skill_categories": []}}

### What to REMOVE (output format instructions):
- "return as XML", "output JSON", "用XML格式", "terminalResponse", "chatResponse", "markdown table"
- "format as CSV", "convert to YAML", "display as table"
- Any schema, template, or placeholder structure

### What to KEEP (task execution content):
- User's actual request: "search", "download", "save", "read", "write", "copy", "move"
- File paths: "C:\Users\...\workspace", "/home/user/data.txt", "./config.json"
- URLs: "https://bilibili.com", "http://api.example.com"
- Directories: "save to workspace", "download to local", "put in default directory"
- Rules: "workspace directory: C:\...", "default path is ..."
- Search queries: "find 2 alien videos"
- Numbers, keys, values, text content

### KEY PRINCIPLE:
If it tells the system HOW to present the output → REMOVE
If it tells the system WHAT to do or WHERE to put things → KEEP

Examples:
- "return as XML" → REMOVE
- "save to C:\workspace\file.xlsx" → KEEP (path is data)
- "use workspace directory: C:\...\workspace" → KEEP (execution rule)
- "output JSON format" → REMOVE

## Available Categories:
{}

## YOUR OUTPUT:
"#,
        input, categories_list
    )
}
