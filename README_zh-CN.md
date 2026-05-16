<h1 align="center">
   🦛 河马X
</h1>
<h4 align="center">
一个可靠的AI代理引擎.
一个Skill驱动的AI代理引擎, 你只需要编写 `SKILL.md` 文件来描述技能就能自动加载并执行,它不绑定任何前端——CLI、TCP、HTTP、WebSocket都可以接入.
</h4>
<p align="center">
  <a href="https://github.com/0xhappyboy/hippo/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-Apache2.0-d1d1f6.svg?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=googledocs&label=license&logoColor=BEC5C9" alt="License"></a>
</p>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

## 基础使用

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

## 支持协议

| 协议      | 地址                  |
| --------- | --------------------- |
| CLI       | 终端直接交互          |
| TCP       | 127.0.0.1:8080        |
| HTTP      | http://127.0.0.1:8081 |
| WebSocket | ws://127.0.0.1:8082   |

## 环境变量

| 变量                   | 默认值 | 说明                                            |
| ---------------------- | ------ | ----------------------------------------------- |
| HIPPO_LANG             | en     | 语言设置 (en/zh)                                |
| HIPPO_LLM_PROVIDER_KEY | 无     | LLM 提供商 (openai/deepseek/anthropic/google)   |
| HIPPO_ENABLE_CLI       | true   | 启用 CLI 命令行交互                             |
| HIPPO_ENABLE_TCP       | false  | 启用 TCP 服务器，地址 127.0.0.1:8080            |
| HIPPO_ENABLE_HTTP      | false  | 启用 HTTP 服务器，地址 http://127.0.0.1:8081    |
| HIPPO_ENABLE_WS        | false  | 启用 WebSocket 服务器，地址 ws://127.0.0.1:8082 |
