<p align="center">
    <img src="./assets/logo/logo-1.png" alt="Portal" width="100" height="100">
</p>
<h1 align="center">
    HippoX
</h1>
<h4 align="center">
An reliable AI runtime and skills orchestration engine with autonomous decision-making.<br>
A skill-driven AI runtime with autonomous decision-making that automatically loads and executes skills simply by writing a SKILL.md file.
</h4>
<p align="center">
  <a href="https://github.com/0xhappyboy/hippo/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-Apache2.0-d1d1f6.svg?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=googledocs&label=license&logoColor=BEC5C9" alt="License"></a>
  <a href="https://crates.io/crates/hippox">
  <img src="https://img.shields.io/badge/crates-hippox-20B2AA.svg?style=flat&labelColor=0F1F2D&color=FFD700&logo=rust&logoColor=FFD700">
  </a><a href="https://crates.io/crates/hippox">
  <img src="https://img.shields.io/crates/d/hippox?style=flat&labelColor=0F1F2D&color=20B2AA&logo=rust&logoColor=white&label=downloads" alt="Crates.io Downloads">
  </a>
</p>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

### 🔗 Quick Links

| Resource             | Link                                                                       |
| :------------------- | :------------------------------------------------------------------------- |
| 🌐 **Website**       | [https://hippox.vercel.app/](https://hippox.vercel.app/)                   |
| 📖 **Documentation** | [https://hippox-docs-en.vercel.app/](https://hippox-docs-en.vercel.app/)   |
| 📦 **Crates.io**     | [https://crates.io/crates/hippox](https://crates.io/crates/hippox)         |
| 💻 **GitHub**        | [https://github.com/0xhappyboy/hippo](https://github.com/0xhappyboy/hippo) |

## Hippox Core Working Principle

<img src="./assets/architecture/hippox_core_process_en.png" width="100%">

## How skill loaders and schedulers work

<img src="./assets/architecture/skill_load_and_schedul_en.png" width="100%">

## Workflow Model

| Mode           | Enum Value                   | Core Features                                                                                                        | LLM Calls                           | Use Cases                                                 |
| -------------- | ---------------------------- | -------------------------------------------------------------------------------------------------------------------- | ----------------------------------- | --------------------------------------------------------- |
| ReAct          | WorkflowMode::ReAct          | Think → Act → Observe loop, LLM decides next step after each execution                                               | 1 per skill + 1 final response      | Open-ended tasks, dynamic decision making, error recovery |
| Batch          | WorkflowMode::Batch          | Execute multiple independent skills in parallel                                                                      | 1 (generates batch plan)            | Independent operations, bulk processing                   |
| Chain          | WorkflowMode::Chain          | Sequential execution with variable passing ({{variable}} syntax)                                                     | 1 (generates chain)                 | Linear pipelines, data transformation chains              |
| PlanAndExecute | WorkflowMode::PlanAndExecute | One-time planning with conditional branching, variable references ({"$ref":"var"}), error handling (retry/skip/fail) | 1 plan + optional dynamic decisions | Complex workflows, conditional logic, deterministic tasks |

<table>
  <tr>
    <td align="left">
    <h4>Chain</h4>
    </td>
    <td align="left">
    <h4>Batch</h4>
    </td>
  </tr>
  <tr>
    <td align="center"><img src="./assets/architecture/chain_en.png" width="100%" ></td>
    <td align="center"><img src="./assets/architecture/batch_en.png" width="100%"></td>
  </tr>
   <tr>
    <td align="left">
    <h4>ReAct</h4>
    </td>
    <td align="left">
    <h4>PlanAndExecute</h4>
    </td>
  </tr>
  <tr>
    <td align="center"><img src="./assets/architecture/re-act_en.png" width="100%"></td>
    <td align="center"><img src="./assets/architecture/plan-and-execute_en.png" width="100%"></td>
  </tr>
</table>

## Atomic Skill Unit List

| Category             | Skills    | Description                                                                                                                                                                 |
| -------------------- | --------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **File System**      | 5 skills  | Read, write, delete, list, copy files                                                                                                                                       |
| **Archive**          | 5 skills  | Create/extract ZIP/TAR archives, compress files                                                                                                                             |
| **Math**             | 4 skills  | Expression calculator, power/root, statistics, unit conversion                                                                                                              |
| **Crypto/Random**    | 10 skills | MD5, SHA256, SHA512, file hash, Base64 encode/decode, random number/string/uuid/password                                                                                    |
| **Time**             | 1 skill   | Get current date/time                                                                                                                                                       |
| **Network**          | 20 skills | HTTP requests, URL fetch, ICMP/TCP/batch ping, DNS lookup/reverse/batch/test, IP info/validate/range/local, TCP/UDP send/receive/broadcast, FTP upload/download/list/delete |
| **OS Management**    | 18 skills | Reboot, shutdown, sleep, lock, logout, hibernate, uptime, load average, hostname, time, user info, disk/memory/CPU/network/battery info, desktop notification               |
| **Process**          | 6 skills  | List, kill (by PID/name), check running, get PID, detailed process info                                                                                                     |
| **System**           | 7 skills  | System info, exec command, port scan/lookup/test, clipboard get/set/clear                                                                                                   |
| **Document**         | 11 skills | Markdown, CSV, XML, Excel, PDF read/write/parse                                                                                                                             |
| **Messaging**        | 5 skills  | Email, Telegram, DingTalk, Feishu, WeCom                                                                                                                                    |
| **Database**         | 12 skills | PostgreSQL, MySQL, Redis, SQLite query/execute/list                                                                                                                         |
| **Text Processing**  | 4 skills  | Diff, sort, deduplicate, filter                                                                                                                                             |
| **Regex**            | 4 skills  | Match, find, replace, extract                                                                                                                                               |
| **K8s**              | 18 skills | Pod/deployment/service/node/namespace/event/configmap/secret/ingress/statefulset management, logs, exec, scale, restart, port-forward, apply YAML, delete                   |
| **Docker**           | 5 skills  | List, start/stop, logs, inspect, exec                                                                                                                                       |
| **GitHub**           | 7 skills  | Get repo, create/list issues, star, search, get user, list PRs                                                                                                              |
| **Task Scheduler**   | 3 skills  | Schedule, unschedule, list tasks                                                                                                                                            |
| **Image Processing** | 6 skills  | Resize, convert, info, rotate, crop, compress                                                                                                                               |
