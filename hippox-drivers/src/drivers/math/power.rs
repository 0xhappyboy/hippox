//! Math power skill
//!
//! This driver provides functionality to calculate powers, square roots,
//! and cube roots.
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory, format_number,
    types::{Driver, DriverParameter},
    validate_number,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for power operations
#[derive(Debug)]
pub struct PowerDriver;
#[async_trait::async_trait]
impl Driver for PowerDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "math_power"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Calculate power, square root, or cube root operations"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when the user asks to calculate powers, exponents, square roots, or cube roots"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "base".to_string(),
                param_type: "string".to_string(),
                description: "Base number for power operation".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("2".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "exponent".to_string(),
                param_type: "string".to_string(),
                description: "Exponent for power operation".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("10".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "sqrt".to_string(),
                param_type: "string".to_string(),
                description: "Number to calculate square root (alternative to base+exponent)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("16".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "cbrt".to_string(),
                param_type: "string".to_string(),
                description: "Number to calculate cube root (alternative to base+exponent)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("27".to_string())),
                enum_values: None,
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
            "action": "math_power",
            "parameters": {
                "base": "2",
                "exponent": "10"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "1024.00".to_string();
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
        debug!("Executing math_power driver");
        let precision = parameters.get("precision").and_then(|v| v.as_u64()).unwrap_or(2);
        if let Some(value) = parameters.get("sqrt").and_then(|v| v.as_str()) {
            debug!("Calculating square root of: {}", value);
            let num = validate_number(value).map_err(|e| {
                debug!("Failed to validate number: {}", e);
                return crate::DriverError::validation("sqrt", format!("Invalid number: {}", e));
            })?;
            if num < 0.0 {
                warn!("Cannot calculate square root of negative number: {}", num);
                return Err(crate::DriverError::validation("sqrt", format!("Cannot calculate square root of negative number: {}", num)));
            }
            let result = num.sqrt();
            info!("Square root of {} = {}", num, result);
            return Ok(format_number(result, precision as usize));
        }
        if let Some(value) = parameters.get("cbrt").and_then(|v| v.as_str()) {
            debug!("Calculating cube root of: {}", value);
            let num = validate_number(value).map_err(|e| {
                debug!("Failed to validate number: {}", e);
                return crate::DriverError::validation("cbrt", format!("Invalid number: {}", e));
            })?;
            let result = num.cbrt();
            info!("Cube root of {} = {}", num, result);
            return Ok(format_number(result, precision as usize));
        }
        let base = parameters.get("base").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'base' parameter");
            return crate::DriverError::missing_parameter("base");
        })?;
        let exponent = parameters.get("exponent").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'exponent' parameter");
            return crate::DriverError::missing_parameter("exponent");
        })?;
        debug!("Calculating power: {} ^ {}", base, exponent);
        let base_num = validate_number(base).map_err(|e| {
            debug!("Failed to validate base: {}", e);
            return crate::DriverError::validation("base", format!("Invalid base: {}", e));
        })?;
        let exp_num = validate_number(exponent).map_err(|e| {
            debug!("Failed to validate exponent: {}", e);
            return crate::DriverError::validation("exponent", format!("Invalid exponent: {}", e));
        })?;
        let result = base_num.powf(exp_num);
        info!("{} ^ {} = {}", base_num, exp_num, result);
        return Ok(format_number(result, precision as usize));
    }
    /// Validates parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        let has_power = parameters.contains_key("base") && parameters.contains_key("exponent");
        let has_sqrt = parameters.contains_key("sqrt");
        let has_cbrt = parameters.contains_key("cbrt");
        if !has_power && !has_sqrt && !has_cbrt {
            return Err(crate::DriverError::validation("parameters", "Missing parameters: provide (base + exponent) or (sqrt) or (cbrt)"));
        }
        if has_power {
            if parameters.get("base").and_then(|v| v.as_str()).is_none() {
                return Err(crate::DriverError::missing_parameter("base"));
            }
            if parameters.get("exponent").and_then(|v| v.as_str()).is_none() {
                return Err(crate::DriverError::missing_parameter("exponent"));
            }
        }
        return Ok(());
    }
}
