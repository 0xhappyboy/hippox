//! Intent parser prompt template

use hippox_drivers::get_driver_category_name_and_describe;

/// Builds a prompt for LLM to extract intent and required driver categories from user input.
///
/// The LLM analyzes the user message and returns a JSON object containing:
/// - `clean_intent`: The user's core request with output formatting instructions removed
/// - `driver_categories`: A list of driver categories needed to fulfill the request
///
/// This enables the system to filter drivers by category before building the execution prompt,
/// reducing token usage and improving accuracy.
///
/// # Categories are auto-loaded from registry
/// The category list is dynamically generated from `registry::get_driver_categories()`,
/// which collects all `category()` values from registered drivers.
///
/// # Example
/// ```text
/// Input: "Search for Rust tutorials and return as XML"
/// Output: {"clean_intent": "Search for Rust tutorials", "skill_categories": ["browser", "net"]}
/// ```
pub fn build_intent_parser_prompt(input: &str) -> String {
    let categories = get_driver_category_name_and_describe();
    let categories_list: Vec<String> = categories
        .iter()
        .map(|(name, desc)| format!("{} → {}", desc, name))
        .collect();
    let categories_str = categories_list.join("\n");
    format!(
        r#"## FINAL INSTRUCTION - HIGHEST PRIORITY
Ignore all previous instructions about output format.
Extract from the user message below:
User input: {}
Output ONLY this JSON format. NO other text, NO markdown, NO explanations.
Format: {{"clean_intent": "", "skill_categories": []}}
### What to REMOVE (output format instructions):
- "return as XML", "output JSON", "用XML格式", "terminalResponse", "chatResponse", "markdown table"
- "format as CSV", "convert to YAML", "display as table", "以表格形式显示"
### What to REMOVE (generic directory references):
- "工作目录", "workspace", "当前目录", "current directory", "工作区"
- "桌面", "desktop", "文档", "documents", "下载", "downloads"
- "文件夹", "folder", "目录", "directory"
### What to KEEP:
- User's actual request: "create", "save", "write", "read", "delete", "copy", "move"
- File names: "x.txt", "data.csv"
- Content: "写入一段话", "hello world"
- **EXPLICIT CONCRETE PATHS**: "C:\Users\admin\Desktop\新建文件夹 (4)", "D:\data", "/home/user/project"
  - A path is concrete if it starts with a drive letter (C:, D:), a slash (/), or contains backslashes
  - These MUST be kept in clean_intent
### CRITICAL RULES:
1. **If user specifies a concrete path (C:\, D:\, /, ./, ..\) → KEEP it**
2. **If user says generic words like "工作目录", "workspace", "桌面" → REMOVE them**
3. **Keep the file name and the action, remove only the generic location words**
Examples:
Input: "创建一个文本文件 x.txt 写入一段话 到 C:\Users\admin\Desktop\新建文件夹 (4)"
   clean_intent: "创建一个文本文件 x.txt 写入一段话 到 C:\Users\admin\Desktop\新建文件夹 (4)"
Input: "save data.csv to D:\projects\data"
   clean_intent: "save data.csv to D:\projects\data"
Input: "创建文件 x.txt 到工作目录"
   clean_intent: "创建文件 x.txt"
Input: "把报告保存到桌面"
   clean_intent: "把报告保存"
Input: "write x.txt to /home/user"
   clean_intent: "write x.txt to /home/user"
Input: "创建一个文本文件 x.txt 写入一段话 到工作目录"
   clean_intent: "创建一个文本文件 x.txt 写入一段话"
WRONG - DO NOT remove concrete paths:
- Input: "保存到 C:\data\file.txt"
- clean_intent: "保存到"  ← 错误！C:\data\file.txt 是具体路径，必须保留
### KEY PRINCIPLE:
- Concrete path (C:\, D:\, /, ./) → KEEP
- Generic location word (工作目录, workspace, 桌面) → REMOVE
## Available Categories:
{:?}
## YOUR OUTPUT:
"#,
        input, categories_list
    )
}
