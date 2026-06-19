<p align="center">
    <img src="https://raw.githubusercontent.com/0xhappyboy/hippox/main/assets/logo/logo-1.png" alt="Portal" width="100" height="100">
</p>
<h1 align="center">
    hippox-drivers
</h1>
<h4 align="center">
All indivisible atomic driver units in Hippox.
</h4>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

## Working Principle

```
┌─────────────────────────────────────────────────────────────┐
│                      SKILL REGISTRY                        │
│                                                           │
│  SkillRegistryMap = HashMap<SkillCategory,               │
│                      HashMap<String, Arc<dyn Skill>>>    │
│                                                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │ File     │  │ Math     │  │ Net      │              │
│  ├──────────┤  ├──────────┤  ├──────────┤              │
│  │ read     │  │ calc     │  │ http     │              │
│  │ write    │  │ power    │  │ ping     │              │
│  │ delete   │  │ stats    │  │ dns      │              │
│  │ ...      │  │ ...      │  │ ...      │              │
│  └──────────┘  └──────────┘  └──────────┘              │
└─────────────────────────────────────────────────────────────┘

Registration:

  Compile-time: file_register() / math_register() / net_register()
  Runtime: register_skill(category, name, skill)

Query:

  get_skill_by_name("read") → Skill impl → execute()
```

## Core Type

```rust
pub type SkillRegistryMap = HashMap<SkillCategory, HashMap<String, Arc<dyn Skill>>>;
```

## Main Functions

| Function                                       | Description                             |
| ---------------------------------------------- | --------------------------------------- |
| get_registry()                                 | Get read lock on the registry           |
| get_registry_mut()                             | Get write lock on the registry          |
| register_skill(category, name, skill)          | Dynamically register a skill            |
| get_all_skills()                               | Get all registered skills               |
| get_skill_by_name(name)                        | Find a skill by name                    |
| get_skill_by_name_and_category(name, category) | Find a skill by name and category       |
| has_skill(name)                                | Check if a skill exists                 |
| list_skills_names()                            | List all skill names                    |
| list_skills_name_by_category(category)         | List skill names in a category          |
| get_skills_by_category(category)               | Get skills by category string           |
| get_skills_by_category_list(categories)        | Get skills by multiple categories       |
| list_skills_name_by_category_list(categories)  | List skill names by multiple categories |
| get_all_categorys()                            | Get all category names                  |
| get_skill_category()                           | Get categories with skill counts        |
| get_skill_category_names()                     | Get all category names                  |
| get_skill_category_name_and_describe()         | Get category names with descriptions    |
| generate_skill_registry_table_json_str()       | Generate registry JSON string           |

## SkillCategory Methods

| Method           | Description                          |
| ---------------- | ------------------------------------ |
| from_str(s)      | Convert string to enum               |
| name()           | Convert enum to string (lowercase)   |
| display_name()   | Get human-readable display name      |
| description()    | Get category description             |
| icon()           | Get category icon/emoji              |
| priority()       | Get display priority (lower = first) |
| metadata()       | Get complete category metadata       |
| all_categories() | Get metadata for all categories      |
