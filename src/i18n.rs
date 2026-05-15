use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::sync::RwLock;

#[derive(Debug, Deserialize)]
struct Translations {
    app: HashMap<String, String>,
    skill: HashMap<String, String>,
    error: HashMap<String, String>,
    prompt: HashMap<String, String>,
}

static CURRENT_LANG: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new("en".to_string()));
static TRANSLATIONS: Lazy<RwLock<HashMap<String, Translations>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn init() {
    load_translations();
    let lang = std::env::var("HIPPO_LANG").unwrap_or_else(|_| "en".to_string());
    set_language(&lang);
}

fn load_translations() {
    let mut translations = HashMap::new();
    
    for lang in &["en", "zh"] {
        let path = format!("i18n/{}.toml", lang);
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(trans) = toml::from_str(&content) {
                translations.insert(lang.to_string(), trans);
            }
        }
    }
    
    let mut store = TRANSLATIONS.write().unwrap();
    *store = translations;
}

pub fn set_language(lang: &str) {
    let mut current = CURRENT_LANG.write().unwrap();
    *current = lang.to_string();
}

pub fn t(key: &str) -> String {
    let lang = get_language();
    let translations = TRANSLATIONS.read().unwrap();
    
    if let Some(trans) = translations.get(&lang) {
        if let Some(value) = get_value(trans, key) {
            return value;
        }
    }
    
    if let Some(trans) = translations.get("en") {
        if let Some(value) = get_value(trans, key) {
            return value;
        }
    }
    
    key.to_string()
}

pub fn t_with_args(key: &str, args: &[String]) -> String {
    let mut result = t(key);
    for (i, arg) in args.iter().enumerate() {
        result = result.replace(&format!("{{{}}}", i), arg);
    }
    result
}

fn get_value(trans: &Translations, key: &str) -> Option<String> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() != 2 {
        return None;
    }
    
    match parts[0] {
        "app" => trans.app.get(parts[1]).cloned(),
        "skill" => trans.skill.get(parts[1]).cloned(),
        "error" => trans.error.get(parts[1]).cloned(),
        "prompt" => trans.prompt.get(parts[1]).cloned(),
        _ => None,
    }
}

fn get_language() -> String {
    CURRENT_LANG.read().unwrap().clone()
}

#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::t($key)
    };
    ($key:expr, $($arg:expr),*) => {{
        let args = vec![$($arg.to_string()),*];
        $crate::i18n::t_with_args($key, &args)
    }};
}