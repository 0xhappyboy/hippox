use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCategory, format_number, types::{Skill, SkillParameter}, validate_number
};

#[derive(Debug)]
pub struct PowerSkill;

#[async_trait::async_trait]
impl Skill for PowerSkill {
    fn name(&self) -> &str {
        "math_power"
    }

    fn description(&self) -> &str {
        "Calculate power, square root, or cube root operations"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user asks to calculate powers, exponents, square roots, or cube roots"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "base".to_string(),
                param_type: "string".to_string(),
                description: "Base number for power operation".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("2".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "exponent".to_string(),
                param_type: "string".to_string(),
                description: "Exponent for power operation".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("10".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "sqrt".to_string(),
                param_type: "string".to_string(),
                description: "Number to calculate square root (alternative to base+exponent)"
                    .to_string(),
                required: false,
                default: None,
                example: Some(Value::String("16".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "cbrt".to_string(),
                param_type: "string".to_string(),
                description: "Number to calculate cube root (alternative to base+exponent)"
                    .to_string(),
                required: false,
                default: None,
                example: Some(Value::String("27".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "precision".to_string(),
                param_type: "integer".to_string(),
                description: "Number of decimal places in the result".to_string(),
                required: false,
                default: Some(Value::Number(2.into())),
                example: Some(Value::Number(2.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "math_power",
            "parameters": {
                "base": "2",
                "exponent": "10"
            }
        })
    }

    fn example_output(&self) -> String {
        "1024.00".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Math
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        if let Some(value) = parameters.get("sqrt").and_then(|v| v.as_str()) {
            let num = validate_number(value)?;
            if num < 0.0 {
                anyhow::bail!("Cannot calculate square root of negative number: {}", num);
            }
            let result = num.sqrt();
            let precision = parameters
                .get("precision")
                .and_then(|v| v.as_u64())
                .unwrap_or(2);
            return Ok(format_number(result, precision as usize));
        }
        if let Some(value) = parameters.get("cbrt").and_then(|v| v.as_str()) {
            let num = validate_number(value)?;
            let result = num.cbrt();
            let precision = parameters
                .get("precision")
                .and_then(|v| v.as_u64())
                .unwrap_or(2);
            return Ok(format_number(result, precision as usize));
        }
        let base = parameters
            .get("base")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'base' parameter"))?;
        let exponent = parameters
            .get("exponent")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'exponent' parameter"))?;
        let base_num = validate_number(base)?;
        let exp_num = validate_number(exponent)?;
        let result = base_num.powf(exp_num);
        let precision = parameters
            .get("precision")
            .and_then(|v| v.as_u64())
            .unwrap_or(2);
        Ok(format_number(result, precision as usize))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        let has_power = parameters.contains_key("base") && parameters.contains_key("exponent");
        let has_sqrt = parameters.contains_key("sqrt");
        let has_cbrt = parameters.contains_key("cbrt");
        if !has_power && !has_sqrt && !has_cbrt {
            anyhow::bail!("Missing parameters: provide (base + exponent) or (sqrt) or (cbrt)");
        }
        Ok(())
    }
}
