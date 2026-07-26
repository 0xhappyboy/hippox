//! Driver result and error types
//!
//! This module defines the error types and result aliases used throughout
//! the driver system. It provides a comprehensive set of error variants
//! for different failure scenarios.
use std::fmt;
use std::result;

use serde::Deserialize;
use serde::Serialize;
/// Driver-specific error type
///
/// Represents all possible errors that can occur during driver operations.
/// Each variant provides detailed context about the failure to aid in
/// debugging and error handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriverError {
    /// Parameter validation failed
    ///
    /// Occurs when a parameter value does not meet validation rules
    /// (e.g., value out of range, invalid format).
    Validation { field: String, message: String },
    /// Required parameter missing
    ///
    /// Occurs when a required parameter is not provided in the driver call.
    MissingParameter { field: String },
    /// Invalid parameter type
    ///
    /// Occurs when a parameter value type does not match the expected type
    /// (e.g., string when integer was expected).
    InvalidType { field: String, expected: String, actual: String },
    /// Invalid enum value
    ///
    /// Occurs when a string value is not in the allowed set of enum values.
    InvalidEnumValue { field: String, value: String, allowed: Vec<String> },
    /// Execution failed
    ///
    /// Occurs when the driver's core execution logic fails.
    Execution { message: String },
    /// Driver not found
    ///
    /// Occurs when attempting to invoke a driver that is not registered.
    DriverNotFound { name: String },
    /// I/O error
    ///
    /// Occurs during file system, network, or other I/O operations.
    Io { message: String },
    /// Configuration error
    ///
    /// Occurs when driver configuration is invalid or missing.
    Config { message: String },
    /// Internal error
    ///
    /// Occurs due to internal system errors (e.g., invariants violated).
    Internal { message: String },
    /// Context error
    ///
    /// Occurs when there is an issue with the driver execution context.
    Context { message: String },
    /// Timeout error
    ///
    /// Occurs when an operation exceeds the allowed time limit.
    Timeout { duration: Option<String> },
    /// Permission denied
    ///
    /// Occurs when the driver does not have sufficient permissions
    /// to perform the requested operation.
    PermissionDenied { resource: String },
    /// Generic error with optional source
    ///
    /// A catch-all error variant for cases that don't fit other categories.
    /// Can optionally include a source error for context.
    Generic { message: String, source: Option<String> },
}
impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation { field, message } => {
                write!(f, "Validation error for '{}': {}", field, message)?;
            }
            Self::MissingParameter { field } => {
                write!(f, "Required parameter '{}' is missing", field)?;
            }
            Self::InvalidType { field, expected, actual } => {
                write!(f, "Parameter '{}' expects type '{}', got '{}'", field, expected, actual)?;
            }
            Self::InvalidEnumValue { field, value, allowed } => {
                write!(f, "Value '{}' for '{}' not in allowed values: {:?}", value, field, allowed)?;
            }
            Self::Execution { message } => {
                write!(f, "Execution failed: {}", message)?;
            }
            Self::DriverNotFound { name } => {
                write!(f, "Driver '{}' not found", name)?;
            }
            Self::Io { message } => {
                write!(f, "I/O error: {}", message)?;
            }
            Self::Config { message } => {
                write!(f, "Configuration error: {}", message)?;
            }
            Self::Internal { message } => {
                write!(f, "Internal error: {}", message)?;
            }
            Self::Context { message } => {
                write!(f, "Context error: {}", message)?;
            }
            Self::Timeout { duration } => {
                if let Some(d) = duration {
                    write!(f, "Timeout after {}", d)?;
                } else {
                    write!(f, "Timeout occurred")?;
                }
            }
            Self::PermissionDenied { resource } => {
                write!(f, "Permission denied for '{}'", resource)?;
            }
            Self::Generic { message, source } => {
                if let Some(src) = source {
                    write!(f, "{}: {}", message, src)?;
                } else {
                    write!(f, "{}", message)?;
                }
            }
        }
        return Ok(());
    }
}
impl std::error::Error for DriverError {}
/// Result type alias for driver operations
///
/// This type alias simplifies the signature of driver methods by
/// standardizing on `DriverError` as the error type.
pub type DriverResult<T> = result::Result<T, DriverError>;
// Convenience constructors
impl DriverError {
    /// Creates a validation error for a specific field
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        return Self::Validation { field: field.into(), message: message.into() };
    }
    /// Creates a missing parameter error
    pub fn missing_parameter(field: impl Into<String>) -> Self {
        return Self::MissingParameter { field: field.into() };
    }
    /// Creates an invalid type error
    pub fn invalid_type(field: impl Into<String>, expected: impl Into<String>, actual: impl Into<String>) -> Self {
        return Self::InvalidType { field: field.into(), expected: expected.into(), actual: actual.into() };
    }
    /// Creates an execution error
    pub fn execution(message: impl Into<String>) -> Self {
        return Self::Execution { message: message.into() };
    }
    /// Creates a driver not found error
    pub fn not_found(name: impl Into<String>) -> Self {
        return Self::DriverNotFound { name: name.into() };
    }
    /// Creates an I/O error
    pub fn io(message: impl Into<String>) -> Self {
        return Self::Io { message: message.into() };
    }
    /// Creates an invalid enum value error
    pub fn invalid_enum_value(field: impl Into<String>, value: impl Into<String>, allowed: Vec<String>) -> Self {
        return Self::InvalidEnumValue { field: field.into(), value: value.into(), allowed };
    }
    /// Creates an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        return Self::Internal { message: message.into() };
    }
    /// Creates a timeout error
    pub fn timeout(duration: Option<impl Into<String>>) -> Self {
        return Self::Timeout { duration: duration.map(|d| d.into()) };
    }
}
