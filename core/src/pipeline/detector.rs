//! Format requirement detector
//!
//! This module provides a LENIENT detector that checks if user input contains
//! any format or structure requirements that would require Stage Two conversion.
//!
//! The detector is designed to be:
//! - **Lenient**: Prefer false positives over false negatives
//! - **Multi-language**: Supports both English and Chinese patterns
//! - **Extensible**: Easy to add new patterns

use once_cell::sync::Lazy;
use regex::Regex;

/// Check if user input contains any format/structure requirement
///
/// This is a LENIENT detector. If it returns true, we proceed to Stage Two (LLM conversion).
/// If it returns false, we skip Stage Two and return Stage One result directly.
///
/// # Why lenient?
/// - False negatives (missing a requirement) are NOT acceptable because the user's
///   format request would be ignored
/// - False positives (triggering unnecessarily) ARE acceptable because Stage Two
///   LLM will confirm and handle gracefully
///
/// # Detection categories
/// 1. Format keywords (xml, json, yaml, csv, markdown, etc.)
/// 2. Structure markers ({, }, [, ], :, -, |, >)
/// 3. Placeholder patterns ({var}, {{var}}, <var>)
/// 4. Template keywords ("put here", "fill in", "replace with")
/// 5. Code blocks (```)
/// 6. YAML frontmatter (---)
///
/// # Examples
/// ```rust
/// assert!(needs_format_conversion("return as XML"));
/// assert!(needs_format_conversion("用JSON格式返回"));
/// assert!(needs_format_conversion("{\"key\": \"value\"}"));
/// assert!(needs_format_conversion("Put result here: {result}"));
/// assert!(!needs_format_conversion("Calculate 2+3"));
/// ```
pub fn needs_format_conversion(user_input: &str) -> bool {
    if user_input.is_empty() {
        return false;
    }
    let input_lower = user_input.to_lowercase();
    // Format Keywords - User explicitly asks for a specific output format
    let format_keywords = [
        "xml",
        "json",
        "yaml",
        "toml",
        "csv",
        "markdown",
        "md",
        "table",
        "html",
        "plain text",
        "text",
        "format",
        "xml格式",
        "json格式",
        "yaml格式",
        "toml格式",
        "csv格式",
        "markdown格式",
        "md格式",
        "表格",
        "html格式",
        "纯文本",
        "文本格式",
        "格式化",
        "格式",
    ];
    for kw in format_keywords {
        if input_lower.contains(kw) {
            return true;
        }
    }
    // Structure Markers - User provides a template/structure definition
    let structure_markers = ['{', '}', '[', ']', ':', '-', '|', '>'];
    for marker in structure_markers {
        if user_input.contains(marker) {
            return true;
        }
    }
    // YAML Frontmatter - Document metadata style
    if user_input.contains("---") && user_input.contains('\n') {
        return true;
    }
    // Placeholder Patterns - Variables/templates to be filled
    static PLACEHOLDER_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\{\{?[^{}]+\}\}?|<[^>]+>|\$\{[^}]+\}").unwrap());
    if PLACEHOLDER_REGEX.is_match(user_input) {
        return true;
    }
    // Template Keywords - Natural language indicators of templates
    let template_keywords = [
        "put here",
        "fill in",
        "replace with",
        "insert here",
        "place here",
        "use this structure",
        "follow this format",
        "output as",
        "return as",
        "format as",
        "convert to",
        "放这里",
        "填入",
        "替换",
        "这里放",
        "此处",
        "放到",
        "放入",
        "使用以下结构",
        "按照这个格式",
        "输出为",
        "返回为",
        "格式化为",
        "转换成",
        "转化为",
        "变成",
        "template",
        "schema",
        "structure",
        "格式",
        "结构",
    ];
    for kw in template_keywords {
        if user_input.contains(kw) {
            return true;
        }
    }
    // Code Block Detection - User provides code block with format
    if user_input.contains("```") {
        return true;
    }
    // Key-Value Pair Patterns - User defines fields
    let kv_patterns = [
        r"(?m)^\s*[a-zA-Z_][a-zA-Z0-9_]*\s*:",
        r"(?m)^\s*[a-zA-Z_][a-zA-Z0-9_]*\s*：",
    ];
    for pattern in kv_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(user_input) {
                return true;
            }
        }
    }
    // Arrow/Assignment Patterns - User maps outputs
    let arrow_keywords = ["->", "=>", "→", "=>", "映射到", "对应"];
    for kw in arrow_keywords {
        if user_input.contains(kw) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_format_keywords_english() {
        assert!(needs_format_conversion("return as XML"));
        assert!(needs_format_conversion("please output JSON format"));
        assert!(needs_format_conversion("convert to YAML"));
        assert!(needs_format_conversion("use markdown table"));
        assert!(needs_format_conversion("CSV format please"));
        assert!(needs_format_conversion("plain text output"));
    }
    #[test]
    fn test_format_keywords_chinese() {
        assert!(needs_format_conversion("用XML格式返回"));
        assert!(needs_format_conversion("请用JSON格式输出"));
        assert!(needs_format_conversion("转换成YAML格式"));
        assert!(needs_format_conversion("使用markdown表格"));
        assert!(needs_format_conversion("CSV格式"));
        assert!(needs_format_conversion("纯文本输出"));
    }
    #[test]
    fn test_structure_markers() {
        assert!(needs_format_conversion("{\"weather\": \"\"}"));
        assert!(needs_format_conversion("[result1, result2]"));
        assert!(needs_format_conversion("weather: sunny"));
        assert!(needs_format_conversion("- item1\n- item2"));
        assert!(needs_format_conversion("key: value"));
        assert!(needs_format_conversion("a: 1\nb: 2\nc: 3"));
    }
    #[test]
    fn test_placeholders() {
        assert!(needs_format_conversion("{weather}"));
        assert!(needs_format_conversion("{{result}}"));
        assert!(needs_format_conversion("<temperature>"));
        assert!(needs_format_conversion(
            "Hello {name}, your score is {score}"
        ));
        assert!(needs_format_conversion("${value}"));
        assert!(needs_format_conversion("{{ user.name }}"));
    }
    #[test]
    fn test_template_keywords_english() {
        assert!(needs_format_conversion("put weather here"));
        assert!(needs_format_conversion("fill the result above"));
        assert!(needs_format_conversion("replace this with output"));
        assert!(needs_format_conversion("insert result here"));
        assert!(needs_format_conversion("use this structure for output"));
        assert!(needs_format_conversion("follow this format"));
    }

    #[test]
    fn test_template_keywords_chinese() {
        assert!(needs_format_conversion("天气放这里"));
        assert!(needs_format_conversion("把结果填入上面"));
        assert!(needs_format_conversion("替换成结果"));
        assert!(needs_format_conversion("此处放答案"));
        assert!(needs_format_conversion("按照这个格式输出"));
        assert!(needs_format_conversion("使用以下结构"));
    }
    #[test]
    fn test_code_block() {
        assert!(needs_format_conversion("```json\n{}\n```"));
        assert!(needs_format_conversion("```yaml\nkey: value\n```"));
        assert!(needs_format_conversion("```xml\n<root></root>\n```"));
    }
    #[test]
    fn test_key_value_patterns() {
        assert!(needs_format_conversion("a:\nb:\nc:\n"));
        assert!(needs_format_conversion("  a: value\n  b: value"));
        assert!(needs_format_conversion("名称：\n年龄：\n地址："));
    }

    #[test]
    fn test_arrow_keywords() {
        assert!(needs_format_conversion("result -> output"));
        assert!(needs_format_conversion("data => formatted"));
        assert!(needs_format_conversion("结果 → 输出"));
        assert!(needs_format_conversion("映射到新结构"));
    }

    #[test]
    fn test_yaml_frontmatter() {
        assert!(needs_format_conversion("---\ntitle: Test\n---"));
        assert!(needs_format_conversion("---\nname: value\n---"));
    }

    #[test]
    fn test_no_requirement() {
        assert!(!needs_format_conversion("calculate 2+3"));
        assert!(!needs_format_conversion("hello world"));
        assert!(!needs_format_conversion("what is the weather today"));
        assert!(!needs_format_conversion("tell me a joke"));
        assert!(!needs_format_conversion(""));
        assert!(!needs_format_conversion(
            "just a normal sentence without any format keywords"
        ));
    }

    #[test]
    fn test_edge_cases() {
        assert!(!needs_format_conversion("a"));
        assert!(!needs_format_conversion("1"));
        assert!(!needs_format_conversion("12345"));
        assert!(!needs_format_conversion("..."));
        assert!(!needs_format_conversion("???"));
        assert!(!needs_format_conversion("Hello, how are you?"));
        assert!(!needs_format_conversion("The answer is 42"));
    }

    #[test]
    fn test_complex_scenarios() {
        assert!(needs_format_conversion(
            "Please put the results in this format:\n\
             Temperature: {temp}\n\
             Humidity: {humidity}\n\
             Wind: {wind}"
        ));
        assert!(needs_format_conversion(
            "Calculate the sum and return as XML"
        ));
        assert!(needs_format_conversion(
            "{\n  \"result\": \"{value}\",\n  \"status\": \"{status}\"\n}"
        ));
        assert!(needs_format_conversion(
            "把计算结果放入以下结构：\n\
             a: \n\
             b: \n\
             c: "
        ));
    }
}
