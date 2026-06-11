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

Categories: math, file, net, crypto, random, document, message, database, devops, system, image, time, task, text, regex, blockchain

## YOUR OUTPUT:
"#,
        input
    )
}
