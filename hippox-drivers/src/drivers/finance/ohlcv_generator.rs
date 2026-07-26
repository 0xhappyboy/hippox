//! OHLCV data generator for financial scenarios
//!
//! This driver provides functionality to generate simulated OHLCV (Open, High, Low, Close, Volume)
//! market data for financial analysis and testing purposes.
//!
//! ## Request (LLM calls the driver)
//! ```json
//! {
//!     "action": "finance_ohlcv_generator",
//!     "parameters": {
//!         "calls": 5000,
//!         "start_time": 1700000000,
//!         "initial_price": 100.0,
//!         "volatility": 0.002,
//!         "trend": 0.0001,
//!         "format": "array"
//!     }
//! }
//! ```
//!
//! ## Response
//! ```json
//! {
//!     "fields": ["o", "h", "l", "c", "v", "t"],
//!     "data": [[100.00, 100.50, 99.80, 100.20, 12345, 1700000000], ...],
//!     "count": 5000,
//!     "description": "5000 data points (5000 seconds of market data)"
//! }
//! ```
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use chrono::Utc;
use rand::Rng;
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for generating simulated OHLCV market data
#[derive(Debug)]
pub struct OhlcvGeneratorDriver;
#[async_trait::async_trait]
impl Driver for OhlcvGeneratorDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "finance_ohlcv_generator"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Generate simulated OHLCV market data. Each call = 1 second."
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use 'calls' for number of data points (1 call = 1 second)."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "calls".to_string(),
                param_type: "integer".to_string(),
                description: "Number of data points (1 = 1 second)".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(5000.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "start_time".to_string(),
                param_type: "integer".to_string(),
                description: "Unix timestamp in seconds".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(1700000000.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "initial_price".to_string(),
                param_type: "number".to_string(),
                description: "Starting price".to_string(),
                required: false,
                default: Some(json!(100.0)),
                example: Some(json!(100.0)),
                enum_values: None,
            },
            DriverParameter {
                name: "volatility".to_string(),
                param_type: "number".to_string(),
                description: "Volatility per second".to_string(),
                required: false,
                default: Some(json!(0.002)),
                example: Some(json!(0.005)),
                enum_values: None,
            },
            DriverParameter {
                name: "trend".to_string(),
                param_type: "number".to_string(),
                description: "Trend drift per second".to_string(),
                required: false,
                default: Some(json!(0.0001)),
                example: Some(json!(0.0003)),
                enum_values: None,
            },
            DriverParameter {
                name: "format".to_string(),
                param_type: "string".to_string(),
                description: "Output format: 'array' or 'object'".to_string(),
                required: false,
                default: Some(Value::String("array".to_string())),
                example: Some(Value::String("array".to_string())),
                enum_values: Some(vec!["array".to_string(), "object".to_string()]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "finance_ohlcv_generator",
            "parameters": {
                "calls": 5000,
                "initial_price": 100.0,
                "volatility": 0.002,
                "trend": 0.0001
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"{"fields":["o","h","l","c","v","t"],"data":[[100.00,100.50,99.80,100.20,12345,1700000000]],"count":1}"#.to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Finance;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing finance_ohlcv_generator driver");
        let calls = parameters.get("calls").and_then(|v| v.as_u64()).ok_or_else(|| {
            debug!("Missing 'calls' parameter");
            return crate::DriverError::missing_parameter("calls");
        })?;
        if calls == 0 || calls > 1_000_000 {
            warn!("calls must be between 1 and 1,000,000, got: {}", calls);
            return Err(crate::DriverError::validation("calls", format!("calls must be between 1 and 1,000,000, got: {}", calls)));
        }
        let start_time = parameters.get("start_time").and_then(|v| v.as_i64()).unwrap_or_else(|| Utc::now().timestamp());
        let initial_price = parameters.get("initial_price").and_then(|v| v.as_f64()).unwrap_or(100.0);
        let volatility = parameters.get("volatility").and_then(|v| v.as_f64()).unwrap_or(0.002);
        let trend = parameters.get("trend").and_then(|v| v.as_f64()).unwrap_or(0.0001);
        let format = parameters.get("format").and_then(|v| v.as_str()).unwrap_or("array");
        debug!("Generating {} data points with initial_price: {}, volatility: {}, trend: {}", calls, initial_price, volatility, trend);
        let data = generate_ohlcv_data(calls as usize, start_time, initial_price, volatility, trend).map_err(|e| {
            debug!("Failed to generate OHLCV data: {}", e);
            return crate::DriverError::execution(format!("Failed to generate OHLCV data: {}", e));
        })?;
        info!("Successfully generated {} OHLCV data points", data.len());
        let result = format_output(&data, format).map_err(|e| {
            debug!("Failed to format output: {}", e);
            return crate::DriverError::execution(format!("Failed to format output: {}", e));
        })?;
        return Ok(result);
    }
    /// Validates parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        if parameters.get("calls").and_then(|v| v.as_u64()).is_none() {
            return Err(crate::DriverError::missing_parameter("calls"));
        }
        return Ok(());
    }
}
/// OHLCV data point structure
#[derive(Debug, Clone, serde::Serialize)]
struct OhlcvPoint {
    o: f64,
    h: f64,
    l: f64,
    c: f64,
    v: u64,
    t: i64,
}
/// Generate OHLCV data using geometric Brownian motion simulation
fn generate_ohlcv_data(count: usize, start_time: i64, initial_price: f64, volatility: f64, trend: f64) -> DriverResult<Vec<OhlcvPoint>> {
    let mut data = Vec::with_capacity(count);
    let mut current_price = initial_price;
    let volume_base = 1000.0;
    for i in 0..count {
        let dt = 1.0;
        // Generate random normal using Box-Muller transform
        let u1: f64 = rand::random::<f64>();
        let u2: f64 = rand::random::<f64>();
        let u1 = if u1 == 0.0 { 1e-10 } else { u1 };
        let z: f64 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        // Geometric Brownian Motion
        let drift = (trend - 0.5 * volatility * volatility) * dt;
        let diffusion = volatility * z * dt.sqrt();
        let return_rate = drift + diffusion;
        let open = current_price;
        let close = open * (1.0 + return_rate);
        // Generate high and low with random range
        let high_low_range = open * volatility * (0.5 + rand::random::<f64>());
        let high = open.max(close) + high_low_range * rand::random::<f64>();
        let low = open.min(close) - high_low_range * (1.0 - rand::random::<f64>());
        // Ensure high is actually high and low is actually low
        let high = high.max(open).max(close);
        let low = low.min(open).min(close);
        // Generate volume correlated with price movement
        let volume_multiplier = 0.5 + rand::random::<f64>();
        let volume = (volume_base * (1.0 + (return_rate * 100.0).abs() * 2.0) * volume_multiplier) as u64;
        let volume = volume.max(100).min(100000);
        let timestamp = start_time + i as i64;
        data.push(OhlcvPoint {
            o: (open * 100.0).round() / 100.0,
            h: (high * 100.0).round() / 100.0,
            l: (low * 100.0).round() / 100.0,
            c: (close * 100.0).round() / 100.0,
            v: volume,
            t: timestamp,
        });
        current_price = close;
    }
    return Ok(data);
}
/// Format OHLCV data as JSON output
fn format_output(data: &[OhlcvPoint], format: &str) -> DriverResult<String> {
    match format {
        "object" => {
            let data_json: Vec<Value> = data
                .iter()
                .map(|p| {
                    json!({
                        "o": p.o,
                        "h": p.h,
                        "l": p.l,
                        "c": p.c,
                        "v": p.v,
                        "t": p.t,
                    })
                })
                .collect();
            let result = json!({
                "data": data_json,
                "count": data.len(),
                "description": format!("{} data points ({} seconds)", data.len(), data.len()),
            });
            return Ok(serde_json::to_string(&result).map_err(|e| {
                debug!("Failed to serialize JSON: {}", e);
                return crate::DriverError::execution(format!("Failed to serialize JSON: {}", e));
            })?);
        }
        _ => {
            let fields = vec!["o", "h", "l", "c", "v", "t"];
            let rows: Vec<Vec<Value>> = data.iter().map(|p| vec![json!(p.o), json!(p.h), json!(p.l), json!(p.c), json!(p.v), json!(p.t)]).collect();
            let result = json!({
                "fields": fields,
                "data": rows,
                "count": data.len(),
                "description": format!("{} data points ({} seconds)", data.len(), data.len()),
            });
            return Ok(serde_json::to_string(&result).map_err(|e| {
                debug!("Failed to serialize JSON: {}", e);
                return crate::DriverError::execution(format!("Failed to serialize JSON: {}", e));
            })?);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ohlcv_generation() {
        let data = generate_ohlcv_data(100, 1700000000, 100.0, 0.002, 0.0001).unwrap();
        assert_eq!(data.len(), 100);
        for point in data {
            assert!(point.h >= point.o);
            assert!(point.h >= point.c);
            assert!(point.l <= point.o);
            assert!(point.l <= point.c);
            assert!(point.v > 0);
            assert!(point.v <= 100000);
        }
    }
    #[test]
    fn test_format_output() {
        let data = generate_ohlcv_data(10, 1700000000, 100.0, 0.002, 0.0001).unwrap();
        let json_str = format_output(&data, "array").unwrap();
        let parsed: Value = serde_json::from_str(&json_str).unwrap();
        assert!(parsed["fields"].is_array());
        assert!(parsed["data"].is_array());
        assert_eq!(parsed["count"].as_u64().unwrap(), 10);
    }
}
