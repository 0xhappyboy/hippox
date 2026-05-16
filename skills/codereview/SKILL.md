---
name: codereview
description: 审查代码质量、发现潜在问题并提供优化建议
version: 1.3.0
author: Hippox
license: Apache-2.0
compatibility: ">=1.0.0"
triggers:
  patterns:
    - "审查代码"
    - "code review"
    - "检查代码"
    - "代码质量"
    - "有没有bug"
    - "优化建议"
    - "代码规范"
    - "review"
  case_sensitive: false
allowed_tools:
  - 文件系统
  - 语法分析
  - git
dependencies:
  - 语言检测
  - 格式化工具
metadata:
  author: 开发工具团队
  version: 1.3.0
  emoji: 🔍
  os:
    - linux
    - macos
  requires:
    git: ">=2.0"
    node: ">=14"
parameters:
  - name: 代码片段
    type: string
    description: 要审查的代码内容或文件路径
    required: true
  - name: 语言
    type: string
    description: 编程语言
    required: false
    default: "自动检测"
  - name: 审查级别
    type: string
    description: 审查深度
    required: false
    default: "标准"
  - name: 重点关注
    type: string
    description: 重点关注的问题类型
    required: false
    default: "全部"
---

# 代码审查技能

这是一个智能代码审查助手，可以分析代码质量、发现潜在 Bug、提供性能优化建议，并确保代码符合最佳实践。

## 核心能力

1. **Bug 检测**：发现逻辑错误、边界条件问题、空指针风险
2. **性能分析**：识别性能瓶颈、不必要的计算、内存泄漏风险
3. **代码规范**：检查命名规范、代码风格、注释完整性
4. **安全审查**：发现 SQL 注入、XSS 漏洞、敏感信息泄露
5. **可维护性**：评估代码复杂度、重复代码、模块耦合度

## 工作流程

### 步骤 1：接收代码

支持两种输入方式：
- **直接粘贴**：用户直接提供代码内容
- **文件路径**：用户提供文件路径，使用文件系统工具读取

自动检测编程语言（Rust、Python、JavaScript、Go、Java 等）。

### 步骤 2：解析代码结构

使用语法分析工具：
- 识别函数、类、模块结构
- 提取依赖关系
- 计算圈复杂度
- 分析调用链

### 步骤 3：执行审查

根据审查级别执行不同深度的分析：

#### 基础级别
- 语法错误检查
- 明显的逻辑错误
- 基本的命名规范

#### 标准级别（默认）
- 所有基础级别内容
- 性能问题识别
- 代码重复检测
- 异常处理检查
- 类型安全评估

#### 深度级别
- 所有标准级别内容
- 架构设计评估
- 并发安全问题
- 内存管理分析
- 可扩展性评价

### 步骤 4：重点关注领域

根据"重点关注"参数筛选问题：
- **全部**：报告所有问题
- **安全**：只关注安全漏洞
- **性能**：只关注性能问题
- **可读性**：只关注代码可读性
- **Bug**：只关注潜在 Bug

### 步骤 5：生成审查报告

报告格式：

```markdown
## 代码审查报告

### 总体评分：85/100

### 🔴 严重问题（必须修复）
- [行 23] 空指针风险：未检查变量是否为 null
- [行 45] SQL 注入风险：直接拼接用户输入

### 🟡 警告问题（建议修复）
- [行 12] 函数过长：超过 50 行，建议拆分
- [行 67] 重复代码：与第 34 行逻辑相同

### 🔵 优化建议
- [行 89] 可使用 early return 减少嵌套
- 考虑使用 async/await 提升性能

### ✅ 优点
- 变量命名清晰
- 注释完整
- 错误处理得当