//! Application install driver
//!
//! This driver provides functionality to install applications using the
//! system package manager (winget on Windows, apt/yum on Linux, brew on macOS).
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for installing applications via package managers
#[derive(Debug)]
pub struct ApplicationControlInstallDriver;
#[async_trait::async_trait]
impl Driver for ApplicationControlInstallDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "application_control_install"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Install an application using the system package manager"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to install software packages. On Windows, uses winget. On Linux, uses apt/yum. On macOS, uses brew."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "package".to_string(),
            param_type: "string".to_string(),
            description: "Package name to install".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("firefox".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "application_control_install",
            "parameters": {
                "package": "firefox"
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Package firefox installed successfully".to_string()
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        DriverCategory::Application
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing application_control_install driver");
        // Extract the package name parameter
        let package = parameters.get("package").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'package' parameter");
            DriverError::missing_parameter("package")
        })?;
        debug!("Installing package: {}", package);
        #[cfg(target_os = "windows")]
        {
            info!("Installing package via winget: {}", package);
            let output =
                std::process::Command::new("winget").args(["install", package, "--accept-package-agreements", "--silent"]).output().map_err(|e| {
                    let msg = format!("Failed to execute winget: {}", e);
                    warn!("{}", msg);
                    DriverError::execution(msg)
                })?;
            if output.status.success() {
                info!("Package installed successfully: {}", package);
                Ok(format!("Package {} installed successfully", package))
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Package installation failed: {}", error);
                Err(DriverError::execution(format!("Failed to install package: {}", error)))
            }
        }
        #[cfg(target_os = "linux")]
        {
            info!("Installing package via apt-get: {}", package);
            let output = std::process::Command::new("sudo").args(["apt-get", "install", "-y", package]).output().map_err(|e| {
                let msg = format!("Failed to execute apt-get: {}", e);
                warn!("{}", msg);
                DriverError::execution(msg)
            })?;
            if output.status.success() {
                info!("Package installed successfully: {}", package);
                Ok(format!("Package {} installed successfully", package))
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Package installation failed: {}", error);
                Err(DriverError::execution(format!("Failed to install package: {}", error)))
            }
        }
        #[cfg(target_os = "macos")]
        {
            info!("Installing package via brew: {}", package);
            let output = std::process::Command::new("brew").args(["install", package]).output().map_err(|e| {
                let msg = format!("Failed to execute brew: {}", e);
                warn!("{}", msg);
                DriverError::execution(msg)
            })?;
            if output.status.success() {
                info!("Package installed successfully: {}", package);
                Ok(format!("Package {} installed successfully", package))
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Package installation failed: {}", error);
                Err(DriverError::execution(format!("Failed to install package: {}", error)))
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            let msg = "Install not implemented on this platform";
            warn!("{}", msg);
            Err(DriverError::execution(msg))
        }
    }
}
