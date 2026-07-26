//! Math statistics skill
//!
//! This driver provides functionality to calculate statistical values
//! from a set of numbers including sum, mean, median, mode, min, and max.
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory, format_number,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for statistical calculations
#[derive(Debug)]
pub struct StatisticsDriver;
#[async_trait::async_trait]
impl Driver for StatisticsDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "math_statistics"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Calculate statistical values from a set of numbers"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when the user asks to calculate statistics like sum, mean, median, mode, min, or max from a list of numbers"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "numbers".to_string(),
                param_type: "array".to_string(),
                description: "Array of numbers to analyze, e.g., [1, 2, 3, 4, 5]".to_string(),
                required: true,
                default: None,
                example: Some(json!([1, 2, 3, 4, 5])),
                enum_values: None,
            },
            DriverParameter {
                name: "operation".to_string(),
                param_type: "string".to_string(),
                description: "Statistical operation: sum, mean, average, min, max, median, mode".to_string(),
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
            DriverParameter {
                name: "precision".to_string(),
                param_type: "integer".to_string(),
                description: "Number of decimal places in the result".to_string(),
                required: false,
                default: Some(Value::Number(2.into())),
                example: Some(Value::Number(2.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "math_statistics",
            "parameters": {
                "numbers": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                "operation": "mean"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "mean = 5.50".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Math;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing math_statistics driver");
        let numbers_json = parameters.get("numbers").ok_or_else(|| {
            debug!("Missing 'numbers' parameter");
            return crate::DriverError::missing_parameter("numbers");
        })?;
        let numbers_array = numbers_json.as_array().ok_or_else(|| {
            debug!("'numbers' must be an array");
            return crate::DriverError::invalid_type("numbers", "array", "non-array");
        })?;
        let mut numbers = Vec::new();
        for num in numbers_array {
            let value = num.as_f64().or_else(|| num.as_str().and_then(|s| s.parse::<f64>().ok())).ok_or_else(|| {
                debug!("Invalid number in array: {:?}", num);
                return crate::DriverError::validation("numbers", format!("Invalid number in array: {:?}", num));
            })?;
            numbers.push(value);
        }
        if numbers.is_empty() {
            warn!("Numbers array is empty");
            return Err(crate::DriverError::validation("numbers", "Numbers array is empty"));
        }
        let operation = parameters.get("operation").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'operation' parameter");
            return crate::DriverError::missing_parameter("operation");
        })?;
        debug!("Calculating {} for {} numbers", operation, numbers.len());
        let precision = parameters.get("precision").and_then(|v| v.as_u64()).unwrap_or(2);
        let result = match operation {
            "sum" => numbers.iter().sum::<f64>(),
            "mean" | "average" => numbers.iter().sum::<f64>() / numbers.len() as f64,
            "min" => numbers.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            "max" => numbers.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            "median" => {
                let mut sorted = numbers.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let mid = sorted.len() / 2;
                if sorted.len() % 2 == 0 { (sorted[mid - 1] + sorted[mid]) / 2.0 } else { sorted[mid] }
            }
            "mode" => {
                use std::collections::HashMap;
                let mut counts = HashMap::new();
                for &num in &numbers {
                    *counts.entry(num.to_string()).or_insert(0) += 1;
                }
                let max_count = *counts.values().max().unwrap_or(&0);
                let modes: Vec<_> = counts.iter().filter(|(_, count)| **count == max_count).map(|(num, _)| num.clone()).collect();
                let result_str = format!("Mode: {}", modes.join(", "));
                info!("{}", result_str);
                return Ok(result_str);
            }
            _ => {
                warn!("Unknown operation: {}", operation);
                return Err(crate::DriverError::invalid_enum_value(
                    "operation",
                    operation.to_string(),
                    vec![
                        "sum".to_string(),
                        "mean".to_string(),
                        "average".to_string(),
                        "min".to_string(),
                        "max".to_string(),
                        "median".to_string(),
                        "mode".to_string(),
                    ],
                ));
            }
        };
        let output = format!("{} = {}", operation, format_number(result, precision as usize));
        info!("Statistical result: {}", output);
        return Ok(output);
    }
    /// Validates parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        if parameters.get("numbers").and_then(|v| v.as_array()).is_none() {
            return Err(crate::DriverError::missing_parameter("numbers"));
        }
        if parameters.get("operation").and_then(|v| v.as_str()).is_none() {
            return Err(crate::DriverError::missing_parameter("operation"));
        }
        return Ok(());
    }
}
