use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::{format_number, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct StatisticsSkill;

#[async_trait::async_trait]
impl Skill for StatisticsSkill {
    fn name(&self) -> &str {
        "math_statistics"
    }

    fn description(&self) -> &str {
        "Calculate statistical values from a set of numbers"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user asks to calculate statistics like sum, mean, median, mode, min, or max from a list of numbers"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "numbers".to_string(),
                param_type: "array".to_string(),
                description: "Array of numbers to analyze, e.g., [1, 2, 3, 4, 5]".to_string(),
                required: true,
                default: None,
                example: Some(json!([1, 2, 3, 4, 5])),
                enum_values: None,
            },
            SkillParameter {
                name: "operation".to_string(),
                param_type: "string".to_string(),
                description: "Statistical operation: sum, mean, average, min, max, median, mode"
                    .to_string(),
                required: true,
                default: None,
                example: Some(Value::String("mean".to_string())),
                enum_values: Some(vec![
                    "sum".to_string(),
                    "mean".to_string(),
                    "average".to_string(),
                    "min".to_string(),
                    "max".to_string(),
                    "median".to_string(),
                    "mode".to_string(),
                ]),
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
            "action": "math_statistics",
            "parameters": {
                "numbers": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                "operation": "mean"
            }
        })
    }

    fn example_output(&self) -> String {
        "mean = 5.50".to_string()
    }

    fn category(&self) -> &str {
        "math"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let numbers_json = parameters
            .get("numbers")
            .ok_or_else(|| anyhow::anyhow!("Missing 'numbers' parameter"))?;
        let numbers_array = numbers_json
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("'numbers' must be an array"))?;
        let mut numbers = Vec::new();
        for num in numbers_array {
            let value = num
                .as_f64()
                .or_else(|| num.as_str().and_then(|s| s.parse::<f64>().ok()))
                .ok_or_else(|| anyhow::anyhow!("Invalid number in array: {:?}", num))?;
            numbers.push(value);
        }
        if numbers.is_empty() {
            anyhow::bail!("Numbers array is empty");
        }
        let operation = parameters
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'operation' parameter"))?;
        let result = match operation {
            "sum" => numbers.iter().sum::<f64>(),
            "mean" | "average" => numbers.iter().sum::<f64>() / numbers.len() as f64,
            "min" => numbers.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            "max" => numbers.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            "median" => {
                let mut sorted = numbers.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let mid = sorted.len() / 2;
                if sorted.len() % 2 == 0 {
                    (sorted[mid - 1] + sorted[mid]) / 2.0
                } else {
                    sorted[mid]
                }
            }
            "mode" => {
                use std::collections::HashMap;
                let mut counts = HashMap::new();
                for &num in &numbers {
                    *counts.entry(num.to_string()).or_insert(0) += 1;
                }
                let max_count = *counts.values().max().unwrap_or(&0);
                let modes: Vec<_> = counts
                    .iter()
                    .filter(|(_, count)| **count == max_count)
                    .map(|(num, _)| num.clone())
                    .collect();
                return Ok(format!("Mode: {}", modes.join(", ")));
            }
            _ => anyhow::bail!("Unknown operation: {}", operation),
        };
        let precision = parameters
            .get("precision")
            .and_then(|v| v.as_u64())
            .unwrap_or(2);
        Ok(format!(
            "{} = {}",
            operation,
            format_number(result, precision as usize)
        ))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("numbers")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: numbers"))?;
        parameters
            .get("operation")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: operation"))?;
        Ok(())
    }
}
