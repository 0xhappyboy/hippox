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

### Instantiate

```rust
// =================== Method 1 ===================
let hippox = Hippox::builder(ModelProvider::OpenAI)
    .api_key("sk-xxx")
    .lang("zh")
    .identity(|id| {
        id.name = Some("agent".to_string());
        id.role = Some("assistant".to_string());
        id.personality = Some("friendly".to_string());
    })
    .add_postgresql(
        PostgreSQLConfig::new(
            "main".to_string(),
            Some("db".to_string()),
            None,
            "localhost".to_string(),
            5432,
            "mydb".to_string(),
            "user".to_string(),
            "password".to_string(),
        )
    )
    .build()
    .await?;

// =================== Method 2 ===================
let mut config = HippoxConfig::default();
config.lang = "zh".to_string();
config.identity_information = IdentityInformation {
    name: Some("agent".to_string()),
    role: Some("assistant".to_string()),
    personality: Some("friendly".to_string()),
    ..Default::default()
};
let pg_config = PostgreSQLConfig::new(
    "main".to_string(),
    Some("db".to_string()),
    None,
    "localhost".to_string(),
    5432,
    "mydb".to_string(),
    "user".to_string(),
    "password".to_string(),
);
config.add_postgresql_instance(pg_config);
let hippox = Hippox::new(
    ModelProvider::OpenAI,
    Some("sk-xxx".to_string()),
    None,
    Some(config),
).await?;

// =================== Simple Method ===================
let hippox = Hippox::new(
    ModelProvider::OpenAI,
    Some("sk-xxx".to_string()),
    None,
    Some(HippoxConfig::default()),
).await?;

// builder
let hippox = Hippox::builder(ModelProvider::OpenAI)
    .api_key("sk-xxx")
    .build()
    .await?;
```

### Task Execution

#### Submit

##### 1. Execution mode

- Asynchronous non-blocking submission. Task goes to background pool, returns task_id immediately. Result must be obtained via polling.

##### 2. How it works

- Call `submit()` method
- `NaturalLanguageTask` is created and pushed to global `TASK_POOL`
- Background execution engine processes tasks automatically
- Method `returns task_id immediately (does NOT wait for completion)`
- Caller repeatedly queries `get_task(task_id)` to check status
- When `task.status == TaskStatus::Completed`, extract result from `task.final_output`

##### 3. Use when

- You don't need immediate results, or want to run multiple tasks concurrently.

```rust
use hippox::{Hippox, TaskStatus};
use langhub::types::ModelProvider;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hippox = Hippox::builder(ModelProvider::OpenAI)
        .api_key("sk-xxx")
        .build()
        .await?;
    // Submit task, returns task_id immediately
    let task_id = hippox.submit("Calculate 15 * 3", None);
    // Poll until task completes
    let result = loop {
        if let Some(task) = hippox.get_task(&task_id) {
            if let Some(output) = task.final_output {
                break output;
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    };
    println!("Result: {}", result);
    Ok(())
}
```

#### Execute - Direct execution

##### 1. Execution mode

- Synchronous blocking call. The function waits until the task completes and returns the result directly.

##### 2. How it works

- Call `execute()` method
- Task starts immediately in the current thread
- Code pauses and waits for completion
- Returns `String` result directly

##### 3. Use when

- You need the result immediately and don't want to manage task state.

```rust
use hippox::Hippox;
use langhub::types::ModelProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hippox = Hippox::builder(ModelProvider::OpenAI)
        .api_key("sk-xxx")
        .build()
        .await?;
    // Execute and wait for result
    let result = hippox.execute("Calculate 15 * 3", None).await;
    println!("Result: {}", result);
    Ok(())
}
```

### Configuration

#### 1. HippoxConfig

```rust
/// Hippox global configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct HippoxConfig {
    /// Language setting: "en" or "zh"
    pub lang: String,
    /// AI identity information (name, role, personality, etc.)
    pub identity_information: IdentityInformation,
    /// PostgreSQL database instances (multiple)
    pub postgresql_instances: HashMap<String, PostgreSQLConfig>,
    /// MySQL database instances (multiple)
    pub mysql_instances: HashMap<String, MySQLConfig>,
    /// Redis instances (multiple)
    pub redis_instances: HashMap<String, RedisConfig>,
    /// SQLite instances (multiple)
    pub sqlite_instances: HashMap<String, SQLiteConfig>,
    /// Docker instances (multiple)
    pub docker_instances: HashMap<String, DockerConfig>,
    /// Kubernetes clusters (multiple)
    pub k8s_instances: HashMap<String, K8sConfig>,
    /// TCP connection instances (multiple)
    pub tcp_instances: HashMap<String, TCPConfig>,
    /// UDP connection instances (multiple)
    pub udp_instances: HashMap<String, UDPConfig>,
    /// FTP server instances (multiple)
    pub ftp_instances: HashMap<String, FTPConfig>,
    /// SMTP email instances (multiple)
    pub smtp_instances: HashMap<String, SMTPConfig>,
    /// Telegram bot instances (multiple)
    pub telegram_instances: HashMap<String, TelegramConfig>,
    /// DingTalk robot instances (multiple)
    pub dingtalk_instances: HashMap<String, DingTalkConfig>,
    /// Feishu webhook instances (multiple)
    pub feishu_instances: HashMap<String, FeishuConfig>,
    /// WeCom webhook instances (multiple)
    pub wecom_instances: HashMap<String, WeComConfig>,
    /// GitHub API instances (multiple)
    pub github_instances: HashMap<String, GitHubConfig>,
}
```

#### 2. IdentityInformation

```rust
/// AI identity configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct IdentityInformation {
    /// AI name, e.g., "Assistant", "Hippox"
    pub name: Option<String>,
    /// Gender, e.g., "male", "female", "neutral"
    pub sex: Option<String>,
    /// Age, e.g., "25", "young"
    pub age: Option<String>,
    /// Species, e.g., "AI", "human", "robot"
    pub species: Option<String>,
    /// Role, e.g., "assistant", "teacher", "life coach"
    pub role: Option<String>,
    /// Personality, e.g., "friendly", "humorous", "professional"
    pub personality: Option<String>,
    /// Tone style, e.g., "casual", "formal", "poetic"
    pub tone_style: Option<String>,
    /// Knowledge scope, e.g., "general", "medical", "programming"
    pub knowledge_scope: Option<String>,
    /// Catchphrase, e.g., "Haha", "I see", "Let's go"
    pub catchphrase: Option<String>,
    /// Prohibited topics, e.g., "no politics", "no medical advice"
    pub taboos: Option<String>,
}
```

## Hippox Core Working Principle

````
 ┌─────────────────────────────────────────────────────────────────────────────────────┐
│                           Hippox Engine - Complete Execution Flow                    │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                     │
│  handle_natural_language(input, callback) → returns task_id                        │
│                                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │                         Task Submission                                      │   │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │   │
│  │  │  NaturalLanguageTask::new(input, workflow_executor, scheduler)       │  │   │
│  │  │  tasks::create_task_with_executable() → push to TASK_POOL            │  │   │
│  │  │  TASK_NOTIFIER.notify_one() → wake up execution engine               │  │   │
│  │  └──────────────────────────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────────────┘   │
│                                      │                                              │
│                                      ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │                      NaturalLanguageTask Async Execution                     │   │
│  ├─────────────────────────────────────────────────────────────────────────────┤   │
│  │                                                                             │   │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │   │
│  │  │ Stage Zero: Intent Classification (SystemPipeline::stage_zero)       │  │   │
│  │  │  ┌────────────────────────────────────────────────────────────────┐  │  │   │
│  │  │  │ build_classifier_prompt() → LLM.chat() → parse → categories    │  │  │   │
│  │  │  └────────────────────────────────────────────────────────────────┘  │  │   │
│  │  │  Output: categories = ["math", "file", "net", ...] or []             │  │   │
│  │  └──────────────────────────────────────────────────────────────────────┘  │   │
│  │                                    │                                        │   │
│  │                                    ▼                                        │   │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │   │
│  │  │ Stage One: Workflow Execution (WorkflowExecutor)                      │  │   │
│  │  │                                                                      │  │   │
│  │  │  ┌────────────────────────────────────────────────────────────────┐ │  │   │
│  │  │  │                   WorkflowMode Dispatch                        │ │  │   │
│  │  │  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌─────────────────┐  │ │  │   │
│  │  │  │  │  ReAct   │ │  Batch   │ │  Chain   │ │ PlanAndExecute  │  │ │  │   │
│  │  │  │  │ loop     │ │ parallel │ │ seq pass │ │ plan→conditional│  │ │  │   │
│  │  │  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └───────┬─────────┘  │ │  │   │
│  │  │  │       │            │            │              │            │ │  │   │
│  │  │  │       └────────────┴────────────┴──────────────┘            │ │  │   │
│  │  │  │                            │                                 │ │  │   │
│  │  │  │                            ▼                                 │ │  │   │
│  │  │  │  ┌─────────────────────────────────────────────────────────┐ │ │  │   │
│  │  │  │  │  Skill Filter: non-empty categories → generate_by_cat    │ │ │  │   │
│  │  │  │  │               empty categories → use all registered      │ │ │  │   │
│  │  │  │  └─────────────────────────────────────────────────────────┘ │ │  │   │
│  │  │  │                            │                                 │ │  │   │
│  │  │  │                            ▼                                 │ │  │   │
│  │  │  │  ┌─────────────────────────────────────────────────────────┐ │ │  │   │
│  │  │  │  │  Build Skill Registry JSON → Inject into LLM Prompt     │ │ │  │   │
│  │  │  │  │  generate_skill_registry_table_json_str()               │ │ │  │   │
│  │  │  │  └─────────────────────────────────────────────────────────┘ │ │  │   │
│  │  │  │                            │                                 │ │  │   │
│  │  │  │                            ▼                                 │ │  │   │
│  │  │  │  ┌─────────────────────────────────────────────────────────┐ │ │  │   │
│  │  │  │  │  LLM generates SkillCall → Executor.execute()           │ │ │  │   │
│  │  │  │  │  ┌────────────────────────────────────────────────────┐ │ │ │  │   │
│  │  │  │  │  │ parse_skill_call() → get_skill() → skill.execute() │ │ │ │  │   │
│  │  │  │  │  └────────────────────────────────────────────────────┘ │ │ │  │   │
│  │  │  │  └─────────────────────────────────────────────────────────┘ │ │  │   │
│  │  │  └────────────────────────────────────────────────────────────────┘ │  │   │
│  │  │                                                                      │  │   │
│  │  │  Output: raw_json (e.g., {"mode":"batch","results":[...]})          │  │   │
│  │  └──────────────────────────────────────────────────────────────────────┘  │   │
│  │                                    │                                        │   │
│  │                                    ▼                                        │   │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │   │
│  │  │ Stage Two: Format Conversion (needs_format_conversion detection)     │  │   │
│  │  │                                                                      │  │   │
│  │  │  Detects: format_keywords | structure {[:|> | YAML front | {{}} | ```│  │   │
│  │  │  ┌─────────────────────────┐      ┌─────────────────────────────┐   │  │   │
│  │  │  │  false: return raw JSON │      │  true: build_format_conv_    │   │  │   │
│  │  │  └───────────┬─────────────┘      │  prompt() → LLM.generate()  │   │  │   │
│  │  │              │                    └───────────────┬─────────────┘   │  │   │
│  │  │              └────────────────────┬───────────────┘                 │  │   │
│  │  │                                   ▼                                 │  │   │
│  │  │                           final_output                              │  │   │
│  │  └──────────────────────────────────────────────────────────────────────┘  │   │
│  │                                    │                                        │   │
│  │                                    ▼                                        │   │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │   │
│  │  │ State Update: state_updater.update_workflow_complete(&final_output)  │  │   │
│  │  │ Callback: callback.on_workflow_complete(task_id, output, duration)   │  │   │
│  │  └──────────────────────────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                     │
├─────────────────────────────────────────────────────────────────────────────────────┤
│  Atomic Skill Registry (registry.rs)                                                │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │ SKILL_REGISTRY: Lazy<RwLock<HashMap<String, Arc<dyn Skill>>>>              │   │
│  │ ├── helloworld    ├── file_read/write/delete/list/copy                      │   │
│  │ ├── math_calc/power/statistics/unit                                         │   │
│  │ ├── hash_md5/sha256/sha512, base64, random_*                                │   │
│  │ ├── net_http/ping/tcp/udp/ftp/dns/ip                                        │   │
│  │ ├── os_*/process_*/exec/port_*/clipboard                                    │   │
│  │ ├── postgres/mysql/redis/sqlite                                             │   │
│  │ ├── k8s_*/docker_*/github_*                                                 │   │
│  │ ├── json/yaml/toml/csv/xml/excel/pdf/markdown/html/pptx/docx                │   │
│  │ └── image_resize/convert/crop/compress                                      │   │
│  └─────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                     │
├─────────────────────────────────────────────────────────────────────────────────────┤
│  Task Pool Management (tasks/mod.rs)                                                │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │ TASK_POOL: Arc<RwLock<TaskPool>> (global singleton)                        │   │
│  │ ├── pending_queue: VecDeque<String>   │ background engine polls           │   │
│  │ ├── running_tasks: Vec<String>        │ → next_task() → execute()         │   │
│  │ └── max_concurrent: 10 (default)      │ → complete_task()                 │   │
│  └─────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘
````

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
│  │              ExecutableTask Implementation                │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │                  NaturalLanguageTask                │  │  │
│  │  │  • input: String                                    │  │  │
│  │  │  • workflow_executor: WorkflowExecutor              │  │  │
│  │  │  • scheduler: SkillScheduler                        │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         External APIs                           │
├─────────────────────────────────────────────────────────────────┤
│  handle_natural_language()  → task_id  (non-blocking)          │
│  get_task_status() / cancel() / pause() / resume() / retry()   │
└─────────────────────────────────────────────────────────────────┘
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
