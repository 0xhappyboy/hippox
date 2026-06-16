use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct HelloWorldSkill;

#[async_trait::async_trait]
impl Skill for HelloWorldSkill {
    fn name(&self) -> &str {
        "helloworld"
    }

    fn description(&self) -> &str {
        "Greet a user by name"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user asks to be greeted or when you need to introduce yourself"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "name".to_string(),
            param_type: "string".to_string(),
            description: "The name of the person to greet".to_string(),
            required: false,
            default: Some(Value::String("World".to_string())),
            example: Some(Value::String("Alice".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "helloworld",
            "parameters": {
                "name": "Alice"
            }
        })
    }

    fn example_output(&self) -> String {
        "Hello, Alice!".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Basic
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let name = parameters
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");
        Ok(format!("Hello, {}!", name))
    }
}
