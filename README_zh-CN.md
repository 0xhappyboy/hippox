<p align="center">
    <img src="./assets/logo/logo-1.png" alt="Portal" width="100" height="100">
</p>
<h1 align="center">
    HippoX
</h1>
<h4 align="center">
一个可靠的具有自主决策能力的LLM运行时和skill编排引擎.<br>
能够处理自然语言并自动执行OS原生原子skill,从根本上使LLM可以真正意义上接管计算机.
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

| 资源             | 链接                                                                       |
| :--------------- | :------------------------------------------------------------------------- |
| 🌐 **官网**      | [https://hippox.vercel.app/](https://hippox.vercel.app/)                   |
| 📖 **文档**      | [https://hippox-docs-en.vercel.app/](https://hippox-docs-en.vercel.app/)   |
| 📦 **Crates.io** | [https://crates.io/crates/hippox](https://crates.io/crates/hippox)         |
| 💻 **GitHub**    | [https://github.com/0xhappyboy/hippo](https://github.com/0xhappyboy/hippo) |

## 基础使用

### 实例化

```rust
// =================== Method 1 ===================
let hippox = Hippox::builder(ModelProvider::OpenAI)
    .api_key("sk-xxx")
    .lang("zh")
    .identity(|id| {
        id.name = Some("智能助手".to_string());
        id.role = Some("assistant".to_string());
        id.personality = Some("friendly".to_string());
    })
    .build()
    .await?;

// =================== Method 2 ===================
let mut config = HippoxConfig::default();
config.lang = "zh".to_string();
config.identity_information = IdentityInformation {
    name: Some("智能助手".to_string()),
    role: Some("assistant".to_string()),
    personality: Some("friendly".to_string()),
    ..Default::default()
};
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

### 任务执行

#### 提交模式

##### 1. 执行方式

- 异步提交,任务放入后台任务池执行,立即返回任务ID,需要通过轮询获取结果.

##### 2. 工作流程

- 调用 `submit()` 方法.
- 创建 `NaturalLanguageTask` 并放入全局 `TASK_POOL`.
- 后台执行引擎自动轮询并执行任务.
- 方法立即返回 `task_id` (不等待任务完成).
- 调用方需要通过 ` get_task(task_id)` 反复查询任务状态.
- 当 `task.status == TaskStatus::Completed` 时, 从 `task.final_output` 取出结果.

##### 3. 适用场景

- 不需要立即等待结果,或需要同时提交多个任务并发执行.

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

#### Execute - 直接执行

##### 1. 执行方式

- 同步等待,函数会阻塞直到任务完成并直接返回结果.

##### 2. 工作流程

- 调用 `execute()` 方法
- 任务在当前线程中立即开始执行
- 代码暂停等待，直到任务完成
- 直接返回 `String` 类型的结果

##### 3. 适用场景

- 需要立即得到结果,且不希望处理异步状态管理.

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
/// Hippox 全局配置
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct HippoxConfig {
    /// 语言设置: "en" 或 "zh"
    pub lang: String,
    /// AI 身份信息（名称、角色、性格等）
    pub identity_information: IdentityInformation,
    /// PostgreSQL 数据库实例（支持多个）
    pub postgresql_instances: HashMap<String, PostgreSQLConfig>,
    /// MySQL 数据库实例（支持多个）
    pub mysql_instances: HashMap<String, MySQLConfig>,
    /// Redis 实例（支持多个）
    pub redis_instances: HashMap<String, RedisConfig>,
    /// SQLite 实例（支持多个）
    pub sqlite_instances: HashMap<String, SQLiteConfig>,
    /// Docker 实例（支持多个）
    pub docker_instances: HashMap<String, DockerConfig>,
    /// Kubernetes 集群（支持多个）
    pub k8s_instances: HashMap<String, K8sConfig>,
    /// TCP 连接实例（支持多个）
    pub tcp_instances: HashMap<String, TCPConfig>,
    /// UDP 连接实例（支持多个）
    pub udp_instances: HashMap<String, UDPConfig>,
    /// FTP 服务器实例（支持多个）
    pub ftp_instances: HashMap<String, FTPConfig>,
    /// SMTP 邮件实例（支持多个）
    pub smtp_instances: HashMap<String, SMTPConfig>,
    /// Telegram 机器人实例（支持多个）
    pub telegram_instances: HashMap<String, TelegramConfig>,
    /// 钉钉机器人实例（支持多个）
    pub dingtalk_instances: HashMap<String, DingTalkConfig>,
    /// 飞书 webhook 实例（支持多个）
    pub feishu_instances: HashMap<String, FeishuConfig>,
    /// 企业微信 webhook 实例（支持多个）
    pub wecom_instances: HashMap<String, WeComConfig>,
    /// GitHub API 实例（支持多个）
    pub github_instances: HashMap<String, GitHubConfig>,
}
```

#### 2. IdentityInformation

```rust
/// AI 身份配置
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct IdentityInformation {
    /// AI 名称，如 "助手", "Hippox"
    pub name: Option<String>,
    /// 性别，如 "男", "女", "中性"
    pub sex: Option<String>,
    /// 年龄，如 "25", "年轻"
    pub age: Option<String>,
    /// 物种，如 "AI", "人类", "机器人"
    pub species: Option<String>,
    /// 角色，如 "助手", "老师", "人生导师"
    pub role: Option<String>,
    /// 性格，如 "友好", "幽默", "专业"
    pub personality: Option<String>,
    /// 语气风格，如 "随意", "正式", "诗意"
    pub tone_style: Option<String>,
    /// 知识范围，如 "通用", "医疗", "编程"
    pub knowledge_scope: Option<String>,
    /// 口头禅，如 "哈哈", "原来如此", "走起"
    pub catchphrase: Option<String>,
    /// 禁忌话题，如 "不谈政治", "不给医疗建议"
    pub taboos: Option<String>,
}
```

## Hippox核心工作原理

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Hippox 核心工作原理                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ 1. 任务提交 (非阻塞)                                                   │ │
│  │    hippox.submit(input) → NaturalLanguageTask → TASK_POOL            │ │
│  │    → TASK_NOTIFIER.notify_one() → 返回 task_id                       │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                      │                                      │
│                                      ▼                                      │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ 2. 意图分析 (步骤1)                                                    │ │
│  │    build_intent_parser_prompt() → LLM.generate() → 解析              │ │
│  │    输出: clean_intent（纯净意图）, skill_categories（技能分类）       │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                      │                                      │
│                                      ▼                                      │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ 3. 工作流执行 (步骤2) 使用 clean_intent                                │ │
│  │    ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌─────────────────┐        │ │
│  │    │  ReAct   │ │  Batch   │ │  Chain   │ │ PlanAndExecute  │        │ │
│  │    │ 循环执行 │ │ 并行执行 │ │ 顺序传递 │ │ 计划→条件执行   │        │ │
│  │    └────┬─────┘ └────┬─────┘ └────┬─────┘ └───────┬─────────┘        │ │
│  │         └────────────┴────────────┴──────────────┘                   │ │
│  │    LLM生成SkillCall → Executor.execute() → raw_json                  │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                      │                                      │
│                                      ▼                                      │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ 4. 返回值整形 (步骤3)                                                   │ │
│  │    needs_format_conversion(原始输入)?                                  │ │
│  │    ┌─────────┐      ┌─────────────────────────────────────────────┐  │ │
│  │    │  否     │ ──▶ │  直接返回 raw_json                            │  │ │
│  │    └─────────┘      └─────────────────────────────────────────────┘  │ │
│  │    ┌─────────┐      ┌─────────────────────────────────────────────┐  │ │
│  │    │  是     │ ──▶ │  build_format_conversion_prompt()            │  │ │
│  │    └─────────┘      │  → LLM.generate() → 格式化输出               │  │ │
│  │                     └─────────────────────────────────────────────┘  │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                      │                                      │
│                                      ▼                                      │
│                              最终输出 (final_output)                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 流水线

```
用户输入
    │
    ▼
┌─────────────┐
│   Step 1    │ → 意图分析 (Intent Analysis)
│  意图分析    │   build_intent_parser_prompt() → LLM
└──────┬──────┘   输出: clean_intent, skill_categories
       │
       ▼
┌─────────────┐
│   Step 2    │ → 工作流执行 (Workflow Execution)
│  工作流执行  │   使用 clean_intent 执行技能
└──────┬──────┘   输出: raw_json
       │
       ▼
┌─────────────┐      ┌─────────────┐
│ 需要转换格式？│ ──Yes──▶│   Step 3    │ → 返回值整形 (Response Formatting)
│ (detector)  │      │  返回值整形  │   build_format_conversion_prompt() → LLM
└──────┬──────┘      └──────┬──────┘
       │                    │
       └──────────┬─────────┘
                  ▼
             最终输出
```

## 任务池

### 状态机

```
等待中 ──► 运行中 ──► 已完成
    │         │
    │         ├──► 已暂停 ──► 运行中 (恢复)
    │         │
    │         └──► 已取消
    │
    └──► 已取消
              │
              └──► 失败 ──► 等待中 (重试)
```

### 架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                      全局静态变量 (自动启动)                      │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                   GLOBAL_TASK_POOL                         │  │
│  │                 (程序加载时自动初始化)                       │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         Hippox 实例                             │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    任务池 (全局)                           │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐                   │  │
│  │  │ 任务 A  │  │ 任务 B  │  │ 任务 C  │                   │  │
│  │  │ (等待)  │  │ (运行)  │  │ (等待)  │                   │  │
│  │  └────┬────┘  └────┬────┘  └────┬────┘                   │  │
│  │       └────────────┼────────────┘                         │  │
│  │                    ▼                                      │  │
│  │         ┌─────────────────────┐                          │  │
│  │         │     优先级队列       │                          │  │
│  │         │  [任务 A, 任务 C]   │                          │  │
│  │         └──────────┬──────────┘                          │  │
│  │                    │                                      │  │
│  │                    ▼                                      │  │
│  │         ┌─────────────────────┐                          │  │
│  │         │      执行引擎        │  (最多: 10 线程)        │  │
│  │         │  ┌────┐ ┌────┐ ┌────┐                         │  │
│  │         │  │ W1 │ │ W2 │ │ W3 │  ...                    │  │
│  │         │  └──┬─┘ └──┬─┘ └──┬─┘                         │  │
│  │         │     └──────┼──────┘                           │  │
│  │         │           ▼                                   │  │
│  │         │  ┌─────────────────┐                          │  │
│  │         │  │  可执行任务接口  │                          │  │
│  │         │  │   .execute()    │                          │  │
│  │         │  └─────────────────┘                          │  │
│  │         └─────────────────────┘                          │  │
│  │                    ▲                                      │  │
│  │         ┌──────────┴──────────┐                          │  │
│  │         │   通知器 (唤醒)      │                          │  │
│  │         └─────────────────────┘                          │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                可执行任务接口 实现                         │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │                  NaturalLanguageTask                │  │  │
│  │  │  • input: String (用户输入)                         │  │  │
│  │  │  • workflow_executor: WorkflowExecutor (工作流执行器)│  │  │
│  │  │  • scheduler: SkillScheduler (技能调度器)           │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                          对外 API                               │
├─────────────────────────────────────────────────────────────────┤
│  handle_natural_language()  → 任务 ID  (非阻塞)                │
│  get_task_status() / cancel() / pause() / resume() / retry()   │
└─────────────────────────────────────────────────────────────────┘
```

## 工作流模式

| 模式           | 枚举值                       | 核心特点                                                                                   | LLM调用次数            | 适用场景                         |
| -------------- | ---------------------------- | ------------------------------------------------------------------------------------------ | ---------------------- | -------------------------------- |
| ReAct          | WorkflowMode::ReAct          | 思考→行动→观察循环，每步执行后由LLM决定下一步                                              | 1次/技能 + 1次最终响应 | 开放性任务、动态决策、错误恢复   |
| Batch          | WorkflowMode::Batch          | 并行执行多个无依赖关系的独立技能                                                           | 1次（生成批量计划）    | 独立操作、批量处理               |
| Chain          | WorkflowMode::Chain          | 顺序执行，支持变量传递（{{variable}}语法）                                                 | 1次（生成链）          | 线性管道、数据转换链             |
| PlanAndExecute | WorkflowMode::PlanAndExecute | 一次性规划完整工作流，支持条件分支、变量引用（{"$ref":"var"}）、错误处理（重试/跳过/失败） | 1次规划 + 可选动态决策 | 复杂工作流、条件逻辑、确定性任务 |

<table>
  <tr>
    <td align="left">
    <h4>链式模式</h4>
    </td>
    <td align="left">
    <h4>批处理模式</h4>
    </td>
  </tr>
  <tr>
    <td align="center"><img src="./assets/architecture/chain_cn.png" width="100%"></td>
    <td align="center"><img src="./assets/architecture/batch_cn.png" width="100%"></td>
  </tr>
   <tr>
    <td align="left">
    <h4>推理—行动模式</h4>
    </td>
    <td align="left">
    <h4>规划—执行模式</h4>
    </td>
  </tr>
  <tr>
    <td align="center"><img src="./assets/architecture/re-act_cn.png" width="100%"></td>
    <td align="center"><img src="./assets/architecture/plan-and-execute_cn.png" width="100%"></td>
  </tr>
</table>

## 原子Skill单元清单

| 分类           | 数量 | 说明                                                                                                                                  |
| -------------- | ---- | ------------------------------------------------------------------------------------------------------------------------------------- |
| **文件系统**   | 5    | 读取、写入、删除、列表、复制文件                                                                                                      |
| **压缩归档**   | 5    | 创建/解压 ZIP/TAR 归档、文件压缩                                                                                                      |
| **数学计算**   | 4    | 表达式计算、幂/开方、统计、单位换算                                                                                                   |
| **加密/随机**  | 10   | MD5、SHA256、SHA512、文件哈希、Base64编解码、随机数/字符串/UUID/密码                                                                  |
| **时间**       | 1    | 获取当前日期时间                                                                                                                      |
| **网络**       | 20   | HTTP请求、URL获取、ICMP/TCP/批量Ping、DNS查询/反向/批量/测试、IP信息/验证/范围/本地、TCP/UDP收发/广播、FTP上传/下载/列表/删除         |
| **操作系统**   | 18   | 重启、关机、睡眠、锁屏、注销、休眠、运行时间、负载、主机名、时间、用户信息、磁盘/内存/CPU/网络/电池信息、桌面通知                     |
| **进程管理**   | 6    | 列表、终止(按PID/名称)、检查运行状态、获取PID、进程详细信息                                                                           |
| **系统工具**   | 7    | 系统信息、执行命令、端口扫描/查询/测试、剪贴板获取/设置/清除                                                                          |
| **文档处理**   | 11   | Markdown、CSV、XML、Excel、PDF 读写/解析                                                                                              |
| **消息通知**   | 5    | 邮件、Telegram、钉钉、飞书、企业微信                                                                                                  |
| **数据库**     | 12   | PostgreSQL、MySQL、Redis、SQLite 查询/执行/列表                                                                                       |
| **文本处理**   | 4    | 差异对比、排序、去重、过滤                                                                                                            |
| **正则表达式** | 4    | 匹配、查找、替换、提取                                                                                                                |
| **K8s**        | 18   | Pod/部署/服务/节点/命名空间/事件/ConfigMap/Secret/Ingress/StatefulSet管理、日志、执行命令、扩缩容、重启、端口转发、应用YAML、删除资源 |
| **Docker**     | 5    | 列表、启停、日志、详情、执行命令                                                                                                      |
| **GitHub**     | 7    | 获取仓库、创建/列表Issue、Star、搜索、获取用户、列表PR                                                                                |
| **定时任务**   | 3    | 创建、取消、列表任务                                                                                                                  |
| **图片处理**   | 6    | 缩放、格式转换、信息、旋转、裁剪、压缩                                                                                                |
