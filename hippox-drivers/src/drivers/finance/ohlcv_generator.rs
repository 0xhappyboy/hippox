//! OHLCV data generator for financial scenarios
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
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use chrono::Utc;
use rand::Rng;
use rand::rngs::ThreadRng;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct OhlcvGeneratorDriver;

#[async_trait::async_trait]
impl Driver for OhlcvGeneratorDriver {
    fn name(&self) -> &str {
        "finance_ohlcv_generator"
    }

    fn description(&self) -> &str {
        "Generate simulated OHLCV market data. Each call = 1 second."
    }

    fn usage_hint(&self) -> &str {
        "Use 'calls' for number of data points (1 call = 1 second)."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "finance_ohlcv_generator",
            "parameters": {
                "calls": 5000,
                "initial_price": 100.0,
                "volatility": 0.002,
                "trend": 0.0001
            }
        })
    }

    fn example_output(&self) -> String {
        r#"{"fields":["o","h","l","c","v","t"],"data":[[100.00,100.50,99.80,100.20,12345,1700000000]],"count":1}"#.to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Finance
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let calls = parameters
            .get("calls")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'calls'"))?;
        if calls == 0 || calls > 1_000_000 {
            anyhow::bail!("calls must be between 1 and 1,000,000");
        }
        let start_time = parameters
            .get("start_time")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| Utc::now().timestamp());
        let initial_price = parameters
            .get("initial_price")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0);
        let volatility = parameters
            .get("volatility")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.002);
        let trend = parameters
            .get("trend")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0001);
        let format = parameters
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("array");
        let data =
            generate_ohlcv_data(calls as usize, start_time, initial_price, volatility, trend)?;
        let result = format_output(&data, format)?;
        Ok(result)
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("calls")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'calls'"))?;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct OhlcvPoint {
    o: f64,
    h: f64,
    l: f64,
    c: f64,
    v: u64,
    t: i64,
}

fn generate_ohlcv_data(
    count: usize,
    start_time: i64,
    initial_price: f64,
    volatility: f64,
    trend: f64,
) -> Result<Vec<OhlcvPoint>> {
    let mut data = Vec::with_capacity(count);
    let mut current_price = initial_price;
    let volume_base = 1000.0;
    for i in 0..count {
        let dt = 1.0;
        let u1: f64 = rand::random::<f64>();
        let u2: f64 = rand::random::<f64>();
        let u1 = if u1 == 0.0 { 1e-10 } else { u1 };
        let z: f64 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let drift = (trend - 0.5 * volatility * volatility) * dt;
        let diffusion = volatility * z * dt.sqrt();
        let return_rate = drift + diffusion;
        let open = current_price;
        let close = open * (1.0 + return_rate);
        let high_low_range = open * volatility * (0.5 + rand::random::<f64>());
        let high = open.max(close) + high_low_range * rand::random::<f64>();
        let low = open.min(close) - high_low_range * (1.0 - rand::random::<f64>());
        let high = high.max(open).max(close);
        let low = low.min(open).min(close);
        let volume_multiplier = 0.5 + rand::random::<f64>();
        let volume =
            (volume_base * (1.0 + (return_rate * 100.0).abs() * 2.0) * volume_multiplier) as u64;
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
    Ok(data)
}

fn format_output(data: &[OhlcvPoint], format: &str) -> Result<String> {
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
            Ok(serde_json::to_string(&result)?)
        }
        _ => {
            let fields = vec!["o", "h", "l", "c", "v", "t"];
            let rows: Vec<Vec<Value>> = data
                .iter()
                .map(|p| {
                    vec![
                        json!(p.o),
                        json!(p.h),
                        json!(p.l),
                        json!(p.c),
                        json!(p.v),
                        json!(p.t),
                    ]
                })
                .collect();

            let result = json!({
                "fields": fields,
                "data": rows,
                "count": data.len(),
                "description": format!("{} data points ({} seconds)", data.len(), data.len()),
            });
            Ok(serde_json::to_string(&result)?)
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
