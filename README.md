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

## 🔗 Quick Links

| Resource             | Link                                                                       |
| :------------------- | :------------------------------------------------------------------------- |
| 🌐 **Website**       | [https://hippox.vercel.app/](https://hippox.vercel.app/)                   |
| 📖 **Documentation** | [https://hippox-docs-en.vercel.app/](https://hippox-docs-en.vercel.app/)   |
| 📦 **Crates.io**     | [https://crates.io/crates/hippox](https://crates.io/crates/hippox)         |
| 💻 **GitHub**        | [https://github.com/0xhappyboy/hippo](https://github.com/0xhappyboy/hippo) |

## Basic Usage

```rust
use hippox::{Hippox, tasks, TaskStatus};
use langhub::types::ModelProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize Hippox
    let hippox = Hippox::new(ModelProvider::OpenAI, Some("sk-xxx".into()), None, ConfigInitMethod::ParamsJsonStr("{}".into())).await?;
    // Submit task, get task_id immediately
    let task_id = hippox.handle_natural_language("Calculate 15 * 3 and save to result.txt", None);
    // Query task status from global task manager
    loop {
        if let Some(task) = tasks::get_task(&task_id).await {
            println!("Status: {:?}, Progress: {}%", task.status, task.progress());
            if matches!(task.status, TaskStatus::Completed | TaskStatus::Failed) {
                println!("Result: {:?}", task.final_output);
                break;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    Ok(())
}
```

## Hippox Core Working Principle

### Natural Language

```
═══════════════════════════════════════════════════════════════════════════════════
                    HIPPOX NATURAL LANGUAGE ARCHITECTURE (English)
═══════════════════════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────────────────────┐
│  INPUT LAYER                                                                    │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │  User Input: "Calculate 2+3 and save the result to result.txt"          │   │
│   └─────────────────────────────────────────────────────────────────────────┘   │
│                                      │                                          │
│                                      ▼                                          │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │  Hippox::handle_natural_language(input, callback)                       │   │
│   │  → Returns task_id immediately, executes async in background           │   │
│   └─────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
═══════════════════════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────────────────────┐
│  SCHEDULER LAYER: Skill Selection                                              │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │  SkillScheduler::select_skill(user_input)                               │   │
│   │                                                                          │   │
│   │  Method 1: Trigger Matching (keywords)                                  │   │
│   │  ┌─────────────────────────────────────────────────────────────────┐    │   │
│   │  │ "calculate" → math_calculator │ "save" → file_write             │    │   │
│   │  └─────────────────────────────────────────────────────────────────┘    │   │
│   │                           ↓ if no match                                  │   │
│   │  Method 2: LLM Intelligent Selection                                     │   │
│   │  ┌─────────────────────────────────────────────────────────────────┐    │   │
│   │  │ Prompt: Skills Registry + User Input → LLM → {"action": "calc"} │    │   │
│   │  └─────────────────────────────────────────────────────────────────┘    │   │
│   └─────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
═══════════════════════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────────────────────┐
│  EXECUTION LAYER: Workflow Modes                                               │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────────┐          │
│   │   ReAct    │  │   Batch    │  │   Chain    │  │ PlanAndExecute │          │
│   │  (default) │  │  Parallel  │  │  Variable  │  │  DAG + Cond    │          │
│   │  Use for:  │  │  Use for:  │  │  Use for:  │  │  Use for:      │          │
│   │  Open-ended│  │  Independent│  │  Pipeline  │  │  Complex Deps  │          │
│   └────────────┘  └────────────┘  └────────────┘  └────────────────┘          │
│                                                                                  │
│   This example uses ReAct mode:                                                 │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │  Iteration 1: Think → {"action": "math_calculator", "params": {...}}    │   │
│   │               Act → Execute → Returns "5"                               │   │
│   │               Observe → Result: 5, still need to save                   │   │
│   │                                                                          │   │
│   │  Iteration 2: Think → {"action": "file_write", "params": {...}}         │   │
│   │               Act → Execute → Returns "Saved"                           │   │
│   │               Observe → Task complete                                   │   │
│   │                                                                          │   │
│   │  Iteration 3: Think → {"action": "done", "message": "2+3=5, saved"}     │   │
│   └─────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
═══════════════════════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────────────────────┐
│  ATOMIC SKILLS LAYER                                                           │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │                        SKILL REGISTRY                                   │   │
│   │  ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐          │   │
│   │  │math_  │ │file_  │ │  net_ │ │ time_ │ │system_│ │  db_  │          │   │
│   │  │calc   │ │write  │ │http   │ │now    │ │info   │ │query  │          │   │
│   │  │power  │ │read   │ │ping   │ │format │ │exec   │ │redis  │          │   │
│   │  │stats  │ │delete │ │dns    │ │parse  │ │port   │ │k8s    │          │   │
│   │  └───────┘ └───────┘ └───────┘ └───────┘ └───────┘ └───────┘          │   │
│   └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                  │
│   Execute: registry::get_skill("math_calculator") → execute(params) → "5"       │
└─────────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
═══════════════════════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────────────────────┐
│  OUTPUT LAYER                                                                   │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │  Final Output: "2+3=5, saved to result.txt"                             │   │
│   │                                                                          │   │
│   │  Callbacks triggered:                                                    │   │
│   │  • on_step_start()   → Step started                                     │   │
│   │  • on_step_success() → Step succeeded, took XXms                        │   │
│   │  • on_workflow_complete() → Workflow done, total XXms                   │   │
│   └─────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════════════════════
  Features: Async Task Pool │ Priority Queue │ Timeout │ Pause/Resume │ Retry │ I18n
═══════════════════════════════════════════════════════════════════════════════════
```

## Pipe Line

```
User Input
    │
    ▼
┌─────────────┐
│ Stage Zero  │ → Classify intent (math, file, web, etc.)
│ Classify    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Stage One   │ → Execute workflow, output standard JSON
│  Execute    │
└──────┬──────┘
       │
       ▼
┌─────────────┐      ┌─────────────┐
│ Need Format │ ──Yes──▶│ Stage Two   │ → Call LLM to convert
│ Conversion? │      │  Convert    │   (JSON → XML/Table)
└──────┬──────┘      └──────┬──────┘
       │                    │
       └──────────┬─────────┘
                  ▼
            Final Output
```

## Task Pool

### State Machine

```
Pending ──► Running ──► Completed
    │           │
    │           ├──► Paused ──► Running (resume)
    │           │
    │           └──► Cancelled
    │
    └──► Cancelled
              │
              └──► Failed ──► Pending (retry)
```

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Global Static (Auto-Start)                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    GLOBAL_TASK_POOL                        │  │
│  │              (Initialized at program load)                │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Hippox Instance                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    TaskPool (Global)                       │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐                   │  │
│  │  │Task A   │  │Task B   │  │Task C   │                   │  │
│  │  │Pending  │  │Running  │  │Pending  │                   │  │
│  │  └────┬────┘  └────┬────┘  └────┬────┘                   │  │
│  │       └────────────┼────────────┘                         │  │
│  │                    ▼                                      │  │
│  │         ┌─────────────────────┐                          │  │
│  │         │    Priority Queue   │                          │  │
│  │         │  [Task A, Task C]   │                          │  │
│  │         └──────────┬──────────┘                          │  │
│  │                    │                                      │  │
│  │                    ▼                                      │  │
│  │         ┌─────────────────────┐                          │  │
│  │         │  Execution Engine   │  (max: 10 workers)      │  │
│  │         │  ┌────┐ ┌────┐ ┌────┐                         │  │
│  │         │  │ W1 │ │ W2 │ │ W3 │  ...                    │  │
│  │         │  └──┬─┘ └──┬─┘ └──┬─┘                         │  │
│  │         │     └──────┼──────┘                           │  │
│  │         │           ▼                                   │  │
│  │         │  ┌─────────────────┐                          │  │
│  │         │  │ ExecutableTask  │                          │  │
│  │         │  │   .execute()    │                          │  │
│  │         │  └─────────────────┘                          │  │
│  │         └─────────────────────┘                          │  │
│  │                    ▲                                      │  │
│  │         ┌──────────┴──────────┐                          │  │
│  │         │  Notifier (wakeup)  │                          │  │
│  │         └─────────────────────┘                          │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              ExecutableTask Implementations               │  │
│  │  ┌─────────────────────┐    ┌─────────────────────┐       │  │
│  │  │ NaturalLanguageTask │    │    SkillMdTask      │       │  │
│  │  │ • input             │    │ • path              │       │  │
│  │  │ • workflow_executor │    │ • params            │       │  │
│  │  │ • scheduler         │    │ • workflow_executor │       │  │
│  │  │ • skills_registry   │    │ • scheduler         │       │  │
│  │  │ • instances_registry│    │ • skills_registry   │       │  │
│  │  └─────────────────────┘    └─────────────────────┘       │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         External APIs                           │
├─────────────────────────────────────────────────────────────────┤
│  handle_natural_language()  → task_id  (non-blocking)          │
│  handle_skill_md()          → task_id  (non-blocking)          │
│  get_task_status() / cancel() / pause() / resume() / retry()   │
└─────────────────────────────────────────────────────────────────┘
```

## Architectural Layering

```
┌─────────────────────────────────────────────────────────────┐
│                        User Layer                           │
│              handle_natural_language()                      │
│                 handle_skill_md()                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Workflow Layer                          │
│   WorkflowExecutor (ReAct / Batch / Chain / PlanAndExecute) │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│                    Scheduling Layer                          │
│   SkillScheduler (LLM Interaction, Skill Selection, Fallback)│
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Execution Layer                         │
│   Executor (Parse LLM Response, Route to Skills)            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Registry Layer                          │
│   Registry (Skill Registration, Instance Configs)           │
└─────────────────────────────────────────────────────────────┘
```

## Data Stream

```
1. Initialize Hippox
   ├── Load configuration (instance configs)
   ├── Generate skills_registry (cached)
   ├── Generate instances_registry (cached)
   └── Generate welcome_message (cached)

2. First Conversation
   ├── is_first_message = false → true
   ├── Send welcome_message (includes both registries)
   └── LLM knows all skills and instances

3. Subsequent Conversations
   ├── is_first_message = true → no longer sent
   └── Reuse cached registries
```

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
