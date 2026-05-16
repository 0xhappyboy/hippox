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
