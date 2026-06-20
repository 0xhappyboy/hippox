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
│                      DRIVER REGISTRY                        │
│                                                           │
│  DriverRegistryMap = HashMap<DriverCategory,               │
│                      HashMap<String, Arc<dyn Driver>>>    │
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
  Runtime: register_driver(category, name, driver)

Query:

  get_driver_by_name("read") → Driver impl → execute()
```

## Core Type

```rust
pub type DriverRegistryMap = HashMap<DriverCategory, HashMap<String, Arc<dyn Driver>>>;
```

## Main Functions

| Function                                       | Description                             |
| ---------------------------------------------- | --------------------------------------- |
| get_registry()                                 | Get read lock on the registry           |
| get_registry_mut()                             | Get write lock on the registry          |
| register_driver(category, name, driver)          | Dynamically register a driver            |
| get_all_drivers()                               | Get all registered drivers               |
| get_driver_by_name(name)                        | Find a driver by name                    |
| get_driver_by_name_and_category(name, category) | Find a driver by name and category       |
| has_driver(name)                                | Check if a driver exists                 |
| list_drivers_names()                            | List all driver names                    |
| list_drivers_name_by_category(category)         | List driver names in a category          |
| get_drivers_by_category(category)               | Get drivers by category string           |
| get_drivers_by_category_list(categories)        | Get drivers by multiple categories       |
| list_drivers_name_by_category_list(categories)  | List driver names by multiple categories |
| get_all_categorys()                            | Get all category names                  |
| get_driver_category()                           | Get categories with driver counts        |
| get_driver_category_names()                     | Get all category names                  |
| get_driver_category_name_and_describe()         | Get category names with descriptions    |
| generate_driver_registry_table_json_str()       | Generate registry JSON string           |

## DriverCategory Methods

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
