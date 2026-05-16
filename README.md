<h1 align="center">
    🦛 HippoX
</h1>
<h4 align="center">
A reliable AI agent engine. 
A skill-driven AI agent engine that automatically loads and executes skills simply by writing a `SKILL.md` file to describe them. It is not bound to any frontend—CLI, TCP, HTTP, and WebSocket can all be used.
</h4>
<p align="center">
  <a href="https://github.com/0xhappyboy/hippo/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-Apache2.0-d1d1f6.svg?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=googledocs&label=license&logoColor=BEC5C9" alt="License"></a>
</p>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

## Basic Usage

```rust
tracing_subscriber::fmt().init();
i18n::init();
let lang = env::var("HIPPO_LANG").unwrap_or_else(|_| "en".to_string());
let provider = match env::var("HIPPO_LLM_PROVIDER_KEY").as_deref() {
    Ok("deepseek") => ModelProvider::DeepSeek,
    Ok("anthropic") => ModelProvider::Anthropic,
    Ok("google") => ModelProvider::Google,
    _ => ModelProvider::OpenAI,
};
let hippox = Hippox::new("skills", provider, &lang).await?;
// Configure which protocols to enable
let config = ServiceConfig {
    enable_cli: env::var("HIPPO_ENABLE_CLI")
                .unwrap_or_else(|_| "true".to_string())
                .parse::<bool>()
                .unwrap_or(true),
    enable_tcp: env::var("HIPPO_ENABLE_TCP")
                .unwrap_or_else(|_| "false".to_string())
                .parse::<bool>()
                .unwrap_or(false),
    enable_http: env::var("HIPPO_ENABLE_HTTP")
                .unwrap_or_else(|_| "false".to_string())
                .parse::<bool>()
                .unwrap_or(false),
    enable_websocket: env::var("HIPPO_ENABLE_WS")
                .unwrap_or_else(|_| "false".to_string())
                .parse::<bool>()
                .unwrap_or(false),
    enable_grpc: false,
    };
hippox.start(config).await?;
```

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
| HIPPOX_DINGTALK_ACCESS_TOKEN | dingding robot web hook token | None    | 钉钉web hook token                  |

## Supported Protocols

| Protocol  | Address               |
| --------- | --------------------- |
| CLI       | Terminal interaction  |
| TCP       | 127.0.0.1:8080        |
| HTTP      | http://127.0.0.1:8081 |
| WebSocket | ws://127.0.0.1:8082   |

## Environment Variables

| Variable               | Default | Description                                     |
| ---------------------- | ------- | ----------------------------------------------- |
| HIPPO_LANG             | en      | Language setting (en/zh)                        |
| HIPPO_LLM_PROVIDER_KEY | None    | LLM provider (openai/deepseek/anthropic/google) |
| HIPPO_ENABLE_CLI       | true    | Enable CLI interface                            |
| HIPPO_ENABLE_TCP       | false   | Enable TCP server on 127.0.0.1:8080             |
| HIPPO_ENABLE_HTTP      | false   | Enable HTTP server on http://127.0.0.1:8081     |
| HIPPO_ENABLE_WS        | false   | Enable WebSocket server on ws://127.0.0.1:8082  |
