//! WiFi ping gateway skill - test connection to gateway
use super::common::{get_default_gateway, ping_gateway};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for pinging the default gateway
#[derive(Debug)]
pub struct WifiPingGatewayDriver;
#[async_trait::async_trait]
impl Driver for WifiPingGatewayDriver {
    fn name(&self) -> &str {
        return "wifi_ping_gateway";
    }
    fn description(&self) -> &str {
        return "Ping the default gateway to test WiFi connection quality";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to test the connection to your router. High latency or packet loss indicates WiFi issues.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "count".to_string(),
            param_type: "integer".to_string(),
            description: "Number of ping packets to send (default: 4)".to_string(),
            required: false,
            default: Some(Value::Number(4.into())),
            example: Some(Value::Number(10.into())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_ping_gateway",
            "parameters": {
                "count": 4
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Gateway: 192.168.1.1, Ping: 2.5ms (0% loss)".to_string();
    }
    fn category(&self) -> DriverCategory {
        return DriverCategory::Wifi;
    }
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing wifi_ping_gateway driver");
        let gateway = get_default_gateway().map_err(|e| {
            debug!("Failed to get default gateway: {}", e);
            return DriverError::execution(format!("Failed to get default gateway: {}", e));
        })?;
        let (success, avg_time) = ping_gateway(&gateway).map_err(|e| {
            debug!("Failed to ping gateway: {}", e);
            return DriverError::execution(format!("Failed to ping gateway: {}", e));
        })?;
        if success {
            info!("Gateway: {}, Ping: {}ms", gateway, avg_time);
            return Ok(format!("Gateway: {}, Ping: {}ms", gateway, avg_time));
        } else {
            info!("Gateway: {}, Ping failed (timeout or unreachable)", gateway);
            return Ok(format!("Gateway: {}, Ping failed (timeout or unreachable)", gateway));
        }
    }
}
