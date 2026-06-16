<p align="center">
    <img src="https://raw.githubusercontent.com/0xhappyboy/hippox/main/assets/logo/logo-1.png" alt="Portal" width="100" height="100">
</p>
<h1 align="center">
    hippox-atomic-skills
</h1>
<h4 align="center">
Hippox中所有不可分割的原子skill单元.
</h4>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

## 工作原理

```text
┌─────────────────────────────────────────────────────────────┐
│                      技 能 注 册 表                        │
│                                                           │
│  SkillRegistryMap = HashMap<SkillCategory,               │
│                      HashMap<String, Arc<dyn Skill>>>    │
│                                                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │ 文件     │  │ 数学     │  │ 网络     │              │
│  ├──────────┤  ├──────────┤  ├──────────┤              │
│  │ 读文件   │  │ 计算器   │  │ HTTP     │              │
│  │ 写文件   │  │ 幂运算   │  │ Ping     │              │
│  │ 删文件   │  │ 统计     │  │ DNS      │              │
│  │ ...      │  │ ...      │  │ ...      │              │
│  └──────────┘  └──────────┘  └──────────┘              │
└─────────────────────────────────────────────────────────────┘

注册方式:

  编译时: file_register() / math_register() / net_register()
  运行时: register_skill(分类, 名称, 技能)

查询方式:

  get_skill_by_name("读文件") → 技能实现 → execute()
```

## 核心类型

```rust
pub type SkillRegistryMap = HashMap<SkillCategory, HashMap<String, Arc<dyn Skill>>>;
```

### 主要函数

| 函数                                           | 说明                   |
| ---------------------------------------------- | ---------------------- |
| get_registry()                                 | 获取注册表读锁         |
| get_registry_mut()                             | 获取注册表写锁         |
| register_skill(category, name, skill)          | 动态注册技能           |
| get_all_skills()                               | 获取所有技能           |
| get_skill_by_name(name)                        | 按名称查找技能         |
| get_skill_by_name_and_category(name, category) | 按名称和分类查找       |
| has_skill(name)                                | 检查技能是否存在       |
| list_skills_names()                            | 列出所有技能名称       |
| list_skills_name_by_category(category)         | 列出指定分类的技能名称 |
| get_skills_by_category(category)               | 按分类字符串获取技能   |
| get_skills_by_category_list(categories)        | 按多个分类获取技能     |
| list_skills_name_by_category_list(categories)  | 按多个分类获取技能名称 |
| get_all_categorys()                            | 获取所有分类名称       |
| get_skill_category()                           | 获取各分类及其技能数量 |
| get_skill_category_names()                     | 获取所有分类名称       |
| get_skill_category_name_and_describe()         | 获取分类名称及描述     |
| generate_skill_registry_table_json_str()       | 生成注册表 JSON 字符串 |

### SkillCategory 方法

| 方法             | 说明                 |
| ---------------- | -------------------- |
| from_str(s)      | 字符串转枚举         |
| name()           | 枚举转字符串（小写） |
| display_name()   | 获取显示名称         |
| description()    | 获取分类描述         |
| icon()           | 获取分类图标         |
| priority()       | 获取显示优先级       |
| metadata()       | 获取完整元数据       |
| all_categories() | 获取所有分类元数据   |
