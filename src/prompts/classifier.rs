//! Classifier prompt for stage zero - determines which skill categories are needed

/// Build classifier prompt to determine required skill categories
pub fn build_classifier_prompt(input: &str) -> String {
    format!(
        r#"You are a skill category classifier. Your ONLY job is to analyze the user input and determine which skill categories are needed.

## Available Categories
- **math**: Calculations, expressions, unit conversions, power/root operations, statistics
- **file**: File read/write/delete/list/copy, archive (zip/tar/gzip) operations
- **net**: HTTP requests, DNS lookup, ping, IP info, TCP/UDP/FTP operations
- **crypto**: Hash (MD5/SHA256/SHA512), Base64 encode/decode
- **random**: Random numbers, strings, UUID, password generation
- **document**: JSON, YAML, TOML, CSV, XML, Excel, PDF, Markdown, HTML, PPTX, DOCX
- **message**: Email, Telegram, DingDing, Feishu, WeCom
- **database**: PostgreSQL, MySQL, Redis, SQLite queries and operations
- **devops**: Kubernetes, Docker, GitHub operations
- **system**: OS management, process control, command execution, clipboard, notifications
- **image**: Image resize, convert, rotate, crop, compress, info
- **time**: DateTime operations, system time
- **task**: Schedule, unschedule, list scheduled tasks
- **text**: Text diff, sort, deduplicate, filter
- **regex**: Regex match, find, replace, extract
- **blockchain**: Bitcoin, EVM, Solana wallet operations

## User Input
{}

## Output Format
Output ONLY a JSON object with the categories array. If no skill categories are needed, output empty array.

Examples:
- "calculate 2+3" -> {{"categories": ["math"]}}
- "read file and send email" -> {{"categories": ["file", "message"]}}
- "hello" -> {{"categories": []}}

## Your Output
"#,
        input
    )
}
