pub enum ConfigInitMethod {
    TomlFile(String),
    JsonFile(String),
    ParamsJsonStr(String),
}
