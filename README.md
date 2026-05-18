<p align="center">
    <img src="./resources/logo/logo-1.png" alt="Portal" width="100" height="100">
</p>
<h1 align="center">
    HippoX
</h1>
<h4 align="center">
A reliable AI agent and skills orchestration runtime engine. <br>
A skill-driven AI agent engine that automatically loads and executes skills simply by writing a `SKILL.md` file to describe them. 
</h4>
<p align="center">
  <a href="https://github.com/0xhappyboy/hippo/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-Apache2.0-d1d1f6.svg?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=googledocs&label=license&logoColor=BEC5C9" alt="License"></a>
  <a href="https://crates.io/crates/hippox">
<img src="https://img.shields.io/badge/crates-hippox-20B2AA.svg?style=flat&labelColor=0F1F2D&color=FFD700&logo=rust&logoColor=FFD700">
</a>
</p>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

## Basic Usage

```rust
use hippox::{Hippox, ModelProvider, core::ConfigInitMethod};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load from environment variables
    let hippox = Hippox::new(
        "./skills",
        ModelProvider::OpenAI,
        Some("api-key".to_string()),
        None,
        ConfigInitMethod::Env,
    ).await?;
    // Load from TOML file
    let hippox = Hippox::new(
        "./skills",
        ModelProvider::OpenAI,
        Some("api-key".to_string()),
        None,
        ConfigInitMethod::TomlFile("config.toml".to_string()),
    ).await?;
    // Load from JSON file
    let hippox = Hippox::new(
        "./skills",
        ModelProvider::OpenAI,
        Some("api-key".to_string()),
        None,
        ConfigInitMethod::JsonFile("config.json".to_string()),
    ).await?;
    // Load from JSON string
    let config_json = r#"{"lang": "en", "provider": "openai"}"#.to_string();
    let hippox = Hippox::new(
        "./skills",
        ModelProvider::OpenAI,
        Some("api-key".to_string()),
        None,
        ConfigInitMethod::ParamsJsonStr(config_json),
    ).await?;
    let response = hippox.handle_natural_language("What is 15 + 27?", Some("session-1")).await;
    println!("{}", response);
    Ok(())
}
```

## Configuration File Formats

### Environment Variables

```bash
export HIPPOX_LANG=en
export HIPPOX_SMTP_HOST=smtp.gmail.com
export HIPPOX_SMTP_PORT=587
```

### TOML Format (`config.toml`)

```toml
lang = "en"
enable_cli = true

[smtp]
host = "smtp.gmail.com"
port = 587
```

### JSON Format (`config.json`)

```json
{
  "lang": "en",
  "smtp_host": "smtp.gmail.com",
  "smtp_port": 587
}
```

## SKill Scheduling Model

<img src="./resources/scheduler_en.png" width="100%">

## Atomic Skill List

| Skill Name           | Description                                    | Parameters                                                                          | Category  |
| -------------------- | ---------------------------------------------- | ----------------------------------------------------------------------------------- | --------- |
| mysql_query          | Execute SELECT query on MySQL database         | query (string, required), params (array), limit (integer)                           | database  |
| mysql_execute        | Execute INSERT/UPDATE/DELETE on MySQL          | query (string, required), params (array)                                            | database  |
| mysql_list_tables    | List all tables in MySQL database              | none                                                                                | database  |
| postgres_query       | Execute SELECT query on PostgreSQL database    | query (string, required), params (array), limit (integer)                           | database  |
| postgres_execute     | Execute INSERT/UPDATE/DELETE on PostgreSQL     | query (string, required), params (array)                                            | database  |
| postgres_list_tables | List all tables in PostgreSQL database         | schema (string)                                                                     | database  |
| redis_set            | Set a key-value pair in Redis                  | key (string, required), value (string, required), ttl (integer)                     | database  |
| redis_get            | Get a value from Redis by key                  | key (string, required)                                                              | database  |
| redis_del            | Delete a key from Redis                        | key (string, required)                                                              | database  |
| redis_keys           | Find keys matching a pattern in Redis          | pattern (string)                                                                    | database  |
| redis_hset           | Set a field in a Redis hash                    | key (string, required), field (string, required), value (string, required)          | database  |
| redis_hget           | Get a field from a Redis hash                  | key (string, required), field (string, required)                                    | database  |
| sqlite_query         | Execute SELECT query on SQLite database        | query (string, required), params (array), limit (integer)                           | database  |
| sqlite_execute       | Execute INSERT/UPDATE/DELETE on SQLite         | query (string, required), params (array)                                            | database  |
| sqlite_list_tables   | List all tables in SQLite database             | none                                                                                | database  |
| github_get_repo      | Get information about a GitHub repository      | owner (string, required), repo (string, required)                                   | github    |
| github_create_issue  | Create an issue in a GitHub repository         | owner, repo, title (required), body, labels                                         | github    |
| github_list_issues   | List issues from a GitHub repository           | owner, repo, state, limit                                                           | github    |
| github_star_repo     | Star a GitHub repository                       | owner (required), repo (required)                                                   | github    |
| github_search_repos  | Search GitHub repositories by query            | query (required), limit                                                             | github    |
| github_get_user      | Get GitHub user information                    | username (required)                                                                 | github    |
| github_list_prs      | List pull requests from a GitHub repository    | owner, repo, state, limit                                                           | github    |
| csv_read             | Read and parse CSV file content                | path (required), has_header, delimiter, limit                                       | document  |
| csv_write            | Write structured data to a CSV file            | path (required), headers (required), rows (required), delimiter                     | document  |
| excel_read           | Read data from Excel (.xlsx) files             | path (required), sheet, has_header, limit                                           | document  |
| excel_write          | Write data to Excel (.xlsx) files              | path (required), headers (required), rows (required), sheet_name                    | document  |
| markdown_read        | Read and parse Markdown file content           | path (required), extract_frontmatter                                                | document  |
| markdown_write       | Write or generate Markdown content to a file   | path (required), content (required), append                                         | document  |
| xml_parse            | Parse XML content from a file or string        | source (required), is_path, xpath                                                   | document  |
| xml_to_json          | Convert XML content to JSON format             | source (required), is_path, pretty                                                  | document  |
| file_copy            | Copy or move a file                            | source (required), destination (required), move                                     | file      |
| file_delete          | Delete a file or empty directory               | path (required), recursive                                                          | file      |
| file_list            | List contents of a directory                   | path (required), show_hidden, detail                                                | file      |
| file_read            | Read content from a file                       | path (required), max_size                                                           | file      |
| file_write           | Write content to a file                        | path (required), content (required), append                                         | file      |
| calculator           | Evaluate mathematical expressions              | expression (required), precision                                                    | math      |
| unit_converter       | Convert between different units of measurement | value (required), from (required), to (required), precision                         | math      |
| math_power           | Calculate power, square root, or cube root     | base, exponent, sqrt, cbrt, precision                                               | math      |
| math_statistics      | Calculate statistical values from a number set | numbers (required), operation (required), precision                                 | math      |
| send_dingding        | Send a message via DingDing robot              | text (required), at_mobiles, at_all, msg_type, title                                | messaging |
| send_email           | Send an email via SMTP server                  | to (required), subject (required), body (required), from, cc, bcc, is_html          | messaging |
| send_feishu          | Send a message via Feishu (Lark) bot           | text, msg_type, title, content, image_key, at_mobiles, at_all                       | messaging |
| send_telegram        | Send a message via Telegram Bot                | chat_id (required), text (required), parse_mode, disable_notification               | messaging |
| send_wecom           | Send a message via WeCom (Enterprise WeChat)   | text (required), msg_type, mentioned_list, mentioned_mobile_list                    | messaging |
| ftp_upload           | Upload a file to FTP server                    | host, port, username, password, local_path (required), remote_path, mode            | net       |
| ftp_download         | Download a file from FTP server                | host, port, username, password, remote_path (required), local_path (required), mode | net       |
| ftp_list             | List directory contents on FTP server          | host, port, username, password, directory                                           | net       |
| ftp_delete           | Delete a file from FTP server                  | host, port, username, password, remote_path (required)                              | net       |
| http_request         | Send HTTP requests to web APIs                 | url (required), method, headers, body, timeout                                      | net       |
| read_url             | Fetch and read content from a URL              | url (required), method, headers, timeout, max_size, raw                             | net       |
| tcp_send             | Send data over TCP connection                  | host, port, data (required), encoding, timeout, delimiter, wait_response            | net       |
| tcp_receive          | Receive data from TCP connection (server mode) | port (required), bind_address, buffer_size, timeout, encoding, send_response        | net       |
| udp_send             | Send data over UDP                             | host, port, data (required), encoding, timeout                                      | net       |
| udp_receive          | Receive UDP datagram                           | port (required), bind_address, buffer_size, timeout, encoding, send_response        | net       |
| udp_broadcast        | Send UDP broadcast message                     | port (required), data (required), encoding, timeout                                 | net       |
| exec_command         | Execute a system command                       | command, args, timeout, working_dir, env                                            | system    |
| system_info          | Get system information (OS, CPU, memory, disk) | info_type                                                                           | system    |
| datetime             | Get current date/time or convert timezone      | operation, timezone, format                                                         | time      |

## Envs

| Environment Variable         | Description                   | Default | Options                             |
| ---------------------------- | ----------------------------- | ------- | ----------------------------------- |
| HIPPOX_LANG                  | Language setting              | en      | zh, en                              |
| HIPPOX_PROVIDER              | LLM provider                  | openai  | openai, deepseek, anthropic, google |
| HIPPOX_ENABLE_CLI            | Enable CLI interface          | true    | true, false                         |
| HIPPOX_ENABLE_TCP            | Enable TCP server             | false   | true, false                         |
| HIPPOX_ENABLE_HTTP           | Enable HTTP server            | false   | true, false                         |
| HIPPOX_ENABLE_WS             | Enable WebSocket server       | false   | true, false                         |
| HIPPOX_SMTP_HOST             | SMTP server hostname          | None    | smtp.gmail.com                      |
| HIPPOX_SMTP_PORT             | SMTP server port              | 587     | 465, 587                            |
| HIPPOX_SMTP_USERNAME         | SMTP authentication username  | None    | your@gmail.com                      |
| HIPPOX_SMTP_PASSWORD         | SMTP authentication password  | None    | your eamil password                 |
| HIPPOX_SMTP_FROM             | Default sender email address  | None    | bot@example.com                     |
| HIPPOX_TELEGRAM_BOT_TOKEN    | Telegram Bot Token            | None    | 1234567890:xxxxxxxxxxxxxxxx         |
| HIPPOX_DINGDING_ACCESS_TOKEN | dingding robot web hook token | None    | 钉钉web hook token                  |
