//! Unified result types for Hippox core library

use std::fmt;

/// Unified result type for all Hippox operations
///
/// This type wraps any operation result with a common structure,
/// providing consistent error handling and success data access.
#[derive(Debug, Clone)]
pub struct HippoxResult<T> {
    /// Indicates whether the operation was successful
    pub status: HippoxResultStatus,

    /// The data payload if operation succeeded
    pub data: Option<T>,

    /// Error message if operation failed
    pub error: Option<String>,

    /// Optional error code for programmatic error handling
    pub error_code: Option<u16>,

    /// Total input tokens consumed during the operation
    pub input_tokens: u64,

    /// Total output tokens consumed during the operation
    pub output_tokens: u64,
}

/// Status of a Hippox operation result
#[derive(Debug, Clone)]
pub enum HippoxResultStatus {
    /// Operation completed successfully with optional message
    SUCCESS(String),
    /// Operation failed with specific error type
    ERROR(HippoxError),
}

/// Specific error types for Hippox operations
#[derive(Debug, Clone)]
pub enum HippoxError {
    /// System-level error (IO, file system, etc.)
    SYSTEM(String),
    /// Network-related error (connection timeout, DNS, etc.)
    NETWORK(String),
    /// Timeout error
    TIMEOUT(String),
}

impl HippoxError {
    /// Get the error code for this error type
    pub fn error_code(&self) -> u16 {
        match self {
            HippoxError::SYSTEM(_) => 1000,
            HippoxError::NETWORK(_) => 1001,
            HippoxError::TIMEOUT(_) => 1002,
        }
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        match self {
            HippoxError::SYSTEM(msg) => msg,
            HippoxError::NETWORK(msg) => msg,
            HippoxError::TIMEOUT(msg) => msg,
        }
    }
}

impl fmt::Display for HippoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HippoxError::SYSTEM(msg) => write!(f, "System error: {}", msg),
            HippoxError::NETWORK(msg) => write!(f, "Network error: {}", msg),
            HippoxError::TIMEOUT(msg) => write!(f, "Timeout: {}", msg),
        }
    }
}

impl<T> HippoxResult<T> {
    /// Create a successful result with data
    pub fn ok(data: T) -> Self {
        Self {
            status: HippoxResultStatus::SUCCESS(String::new()),
            data: Some(data),
            error: None,
            error_code: None,
            input_tokens: 0,
            output_tokens: 0,
        }
    }

    /// Create a successful result with data and token usage
    pub fn ok_with_tokens(data: T, input_tokens: u64, output_tokens: u64) -> Self {
        Self {
            status: HippoxResultStatus::SUCCESS(String::new()),
            data: Some(data),
            error: None,
            error_code: None,
            input_tokens,
            output_tokens,
        }
    }

    /// Create a successful result with data and message
    pub fn ok_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            status: HippoxResultStatus::SUCCESS(message.into()),
            data: Some(data),
            error: None,
            error_code: None,
            input_tokens: 0,
            output_tokens: 0,
        }
    }

    /// Create a successful result with data, message and token usage
    pub fn ok_with_message_and_tokens(
        data: T,
        message: impl Into<String>,
        input_tokens: u64,
        output_tokens: u64,
    ) -> Self {
        Self {
            status: HippoxResultStatus::SUCCESS(message.into()),
            data: Some(data),
            error: None,
            error_code: None,
            input_tokens,
            output_tokens,
        }
    }

    /// Create an error result
    pub fn err(error: HippoxError) -> Self {
        let error_code = Some(error.error_code());
        let error_msg = Some(error.to_string());
        Self {
            status: HippoxResultStatus::ERROR(error),
            data: None,
            error: error_msg,
            error_code,
            input_tokens: 0,
            output_tokens: 0,
        }
    }

    /// Create a system error result
    pub fn system_error(msg: impl Into<String>) -> Self {
        Self::err(HippoxError::SYSTEM(msg.into()))
    }

    /// Create a network error result
    pub fn network_error(msg: impl Into<String>) -> Self {
        Self::err(HippoxError::NETWORK(msg.into()))
    }

    /// Create a timeout error result
    pub fn timeout_error(msg: impl Into<String>) -> Self {
        Self::err(HippoxError::TIMEOUT(msg.into()))
    }

    /// Check if the result is successful
    pub fn is_ok(&self) -> bool {
        matches!(self.status, HippoxResultStatus::SUCCESS(_))
    }

    /// Check if the result is an error
    pub fn is_err(&self) -> bool {
        matches!(self.status, HippoxResultStatus::ERROR(_))
    }

    /// Get the data, panics if result is an error
    pub fn unwrap(self) -> T {
        match self.data {
            Some(data) => data,
            None => panic!("Called unwrap on an error result: {:?}", self.error),
        }
    }

    /// Get the data or a default value
    pub fn unwrap_or(self, default: T) -> T {
        self.data.unwrap_or(default)
    }

    /// Get the data or compute from a closure
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.data.unwrap_or_else(f)
    }

    /// Get the error if any
    pub fn error(&self) -> Option<&HippoxError> {
        match &self.status {
            HippoxResultStatus::ERROR(err) => Some(err),
            _ => None,
        }
    }

    /// Get the success message if any
    pub fn success_message(&self) -> Option<&str> {
        match &self.status {
            HippoxResultStatus::SUCCESS(msg) => {
                if msg.is_empty() {
                    None
                } else {
                    Some(msg)
                }
            }
            _ => None,
        }
    }

    /// Convert to a standard Result type
    pub fn into_result(self) -> Result<T, HippoxError> {
        match self.status {
            HippoxResultStatus::SUCCESS(_) => self.data.ok_or_else(|| {
                HippoxError::SYSTEM("Missing data in successful result".to_string())
            }),
            HippoxResultStatus::ERROR(err) => Err(err),
        }
    }

    /// Map the inner data to a different type
    pub fn map<U, F>(self, f: F) -> HippoxResult<U>
    where
        F: FnOnce(T) -> U,
    {
        match self.status {
            HippoxResultStatus::SUCCESS(msg) => HippoxResult {
                status: HippoxResultStatus::SUCCESS(msg),
                data: self.data.map(f),
                error: None,
                error_code: None,
                input_tokens: self.input_tokens,
                output_tokens: self.output_tokens,
            },
            HippoxResultStatus::ERROR(err) => HippoxResult {
                status: HippoxResultStatus::ERROR(err),
                data: None,
                error: self.error,
                error_code: self.error_code,
                input_tokens: self.input_tokens,
                output_tokens: self.output_tokens,
            },
        }
    }

    /// Get a reference to the data if present
    pub fn as_ref(&self) -> Option<&T> {
        self.data.as_ref()
    }

    /// Get a mutable reference to the data if present
    pub fn as_mut(&mut self) -> Option<&mut T> {
        self.data.as_mut()
    }

    /// Get token usage as a tuple (input_tokens, output_tokens)
    pub fn token_usage(&self) -> (u64, u64) {
        (self.input_tokens, self.output_tokens)
    }

    /// Check if token usage is recorded
    pub fn has_token_usage(&self) -> bool {
        self.input_tokens > 0 || self.output_tokens > 0
    }
}

impl<T> From<Result<T, String>> for HippoxResult<T> {
    fn from(result: Result<T, String>) -> Self {
        match result {
            Ok(data) => Self::ok(data),
            Err(err) => Self::system_error(err),
        }
    }
}

impl<T> From<Result<T, anyhow::Error>> for HippoxResult<T> {
    fn from(result: Result<T, anyhow::Error>) -> Self {
        match result {
            Ok(data) => Self::ok(data),
            Err(err) => Self::system_error(err.to_string()),
        }
    }
}

impl<T: fmt::Display> fmt::Display for HippoxResult<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.status {
            HippoxResultStatus::SUCCESS(msg) => match &self.data {
                Some(data) => {
                    if msg.is_empty() {
                        if self.has_token_usage() {
                            write!(
                                f,
                                "Success: {} (tokens: input={}, output={})",
                                data, self.input_tokens, self.output_tokens
                            )
                        } else {
                            write!(f, "Success: {}", data)
                        }
                    } else {
                        if self.has_token_usage() {
                            write!(
                                f,
                                "Success ({}): {} (tokens: input={}, output={})",
                                msg, data, self.input_tokens, self.output_tokens
                            )
                        } else {
                            write!(f, "Success ({}): {}", msg, data)
                        }
                    }
                }
                None => write!(f, "Success"),
            },
            HippoxResultStatus::ERROR(err) => write!(f, "{}", err),
        }
    }
}

/// Type alias for HippoxResult with String data (most common use case)
pub type HippoxStringResult = HippoxResult<String>;

/// Type alias for HippoxResult with Vec<String> data (batch operations)
pub type HippoxBatchResult = HippoxResult<Vec<String>>;

/// Type alias for HippoxResult with bool data (status operations)
pub type HippoxBoolResult = HippoxResult<bool>;

/// Type alias for HippoxResult with () data (void operations)
pub type HippoxVoidResult = HippoxResult<()>;
