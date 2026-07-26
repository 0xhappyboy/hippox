//! Unit converter skill
//!
//! This driver provides functionality to convert between different units
//! of measurement including length units.
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverError;
use crate::DriverResult;
use crate::{
    DriverCategory, format_number,
    types::{Driver, DriverParameter},
    validate_number,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for unit conversion
#[derive(Debug)]
pub struct UnitConverterDriver;
#[async_trait::async_trait]
impl Driver for UnitConverterDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "unit_converter";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Convert between different units of measurement";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user asks to convert between units like meters to feet, kilometers to miles, etc.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "value".to_string(),
                param_type: "string".to_string(),
                description: "The numeric value to convert".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("100".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "from".to_string(),
                param_type: "string".to_string(),
                description: "Source unit (m, km, cm, mm, mi, ft, in)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("km".to_string())),
                enum_values: Some(vec![
                    "m".to_string(),
                    "km".to_string(),
                    "cm".to_string(),
                    "mm".to_string(),
                    "mi".to_string(),
                    "ft".to_string(),
                    "in".to_string(),
                ]),
            },
            DriverParameter {
                name: "to".to_string(),
                param_type: "string".to_string(),
                description: "Target unit (m, km, cm, mm, mi, ft, in)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("miles".to_string())),
                enum_values: Some(vec![
                    "m".to_string(),
                    "km".to_string(),
                    "cm".to_string(),
                    "mm".to_string(),
                    "mi".to_string(),
                    "ft".to_string(),
                    "in".to_string(),
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
            "action": "unit_converter",
            "parameters": {
                "value": "100",
                "from": "km",
                "to": "miles"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "100 km = 62.14 miles".to_string();
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
        debug!("Executing unit_converter driver");
        let value_str = parameters.get("value").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'value' parameter");
            return DriverError::missing_parameter("value");
        })?;
        let from_unit = parameters
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                debug!("Missing 'from' parameter");
                return DriverError::missing_parameter("from");
            })?
            .to_lowercase();
        let to_unit = parameters
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                debug!("Missing 'to' parameter");
                return DriverError::missing_parameter("to");
            })?
            .to_lowercase();
        debug!("Converting {} from {} to {}", value_str, from_unit, to_unit);
        let value = validate_number(value_str).map_err(|e| {
            debug!("Failed to validate number: {}", e);
            return DriverError::validation("value", format!("Invalid number: {}", e));
        })?;
        let result = convert_units(value, &from_unit, &to_unit).map_err(|e| {
            debug!("Failed to convert units: {}", e);
            return DriverError::execution(format!("Failed to convert units: {}", e));
        })?;
        let precision = parameters.get("precision").and_then(|v| v.as_u64()).unwrap_or(2);
        let output = format!("{} {} = {} {}", value, from_unit, format_number(result, precision as usize), to_unit);
        info!("Conversion result: {}", output);
        return Ok(output);
    }
    /// Validates parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        if parameters.get("value").and_then(|v| v.as_str()).is_none() {
            return Err(DriverError::missing_parameter("value"));
        }
        if parameters.get("from").and_then(|v| v.as_str()).is_none() {
            return Err(DriverError::missing_parameter("from"));
        }
        if parameters.get("to").and_then(|v| v.as_str()).is_none() {
            return Err(DriverError::missing_parameter("to"));
        }
        return Ok(());
    }
}
/// Convert between length units
fn convert_units(value: f64, from: &str, to: &str) -> Result<f64, String> {
    let to_meters = |unit: &str, val: f64| -> Result<f64, String> {
        match unit {
            "m" | "meter" | "meters" => Ok(val),
            "km" | "kilometer" | "kilometers" => Ok(val * 1000.0),
            "cm" | "centimeter" | "centimeters" => Ok(val / 100.0),
            "mm" | "millimeter" | "millimeters" => Ok(val / 1000.0),
            "mi" | "mile" | "miles" => Ok(val * 1609.344),
            "ft" | "foot" | "feet" => Ok(val * 0.3048),
            "in" | "inch" | "inches" => Ok(val * 0.0254),
            _ => Err(format!("Unknown length unit: {}", unit)),
        }
    };
    let from_meters = to_meters(from, value)?;
    let from_meters_to_target = |unit: &str, val: f64| -> Result<f64, String> {
        match unit {
            "m" | "meter" | "meters" => Ok(val),
            "km" | "kilometer" | "kilometers" => Ok(val / 1000.0),
            "cm" | "centimeter" | "centimeters" => Ok(val * 100.0),
            "mm" | "millimeter" | "millimeters" => Ok(val * 1000.0),
            "mi" | "mile" | "miles" => Ok(val / 1609.344),
            "ft" | "foot" | "feet" => Ok(val / 0.3048),
            "in" | "inch" | "inches" => Ok(val / 0.0254),
            _ => Err(format!("Unknown length unit: {}", unit)),
        }
    };
    return from_meters_to_target(to, from_meters);
}
