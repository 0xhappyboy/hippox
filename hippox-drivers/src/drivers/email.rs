//! Email driver module
//!
//! This module provides functionality to send emails via SMTP server
//! with support for HTML content, attachments, and CC/BCC recipients.
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
/// Retrieves a string parameter from the parameters map
///
/// # Arguments
/// * `params` - Parameters map
/// * `name` - Parameter name
///
/// # Returns
/// * `DriverResult<String>` - Parameter value on success
fn get_param_string(params: &HashMap<String, Value>, name: &str) -> DriverResult<String> {
    return params.get(name).and_then(|v| v.as_str()).map(|s| s.to_string()).ok_or_else(|| DriverError::missing_parameter(name));
}
/// Retrieves a boolean parameter from the parameters map with a default value
///
/// # Arguments
/// * `params` - Parameters map
/// * `name` - Parameter name
/// * `default` - Default value if parameter is not present
///
/// # Returns
/// * `bool` - Parameter value or default
fn get_param_bool(params: &HashMap<String, Value>, name: &str, default: bool) -> bool {
    return params.get(name).and_then(|v| v.as_bool()).unwrap_or(default);
}
/// Retrieves a u64 parameter from the parameters map with a default value
///
/// # Arguments
/// * `params` - Parameters map
/// * `name` - Parameter name
/// * `default` - Default value if parameter is not present
///
/// # Returns
/// * `u64` - Parameter value or default
fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    return params.get(name).and_then(|v| v.as_u64()).unwrap_or(default);
}
/// Strips HTML tags from HTML content
///
/// # Arguments
/// * `html` - HTML content
///
/// # Returns
/// * `String` - Plain text content
fn strip_html_tags(html: &str) -> String {
    let mut result = html.to_string();
    result = result.replace("<br>", "\n");
    result = result.replace("<br/>", "\n");
    result = result.replace("<br />", "\n");
    result = result.replace("</p>", "\n\n");
    result = result.replace("</div>", "\n");
    result = result.replace("</h1>", "\n");
    result = result.replace("</h2>", "\n");
    result = result.replace("</h3>", "\n");
    result = result.replace("</h4>", "\n");
    result = result.replace("</h5>", "\n");
    result = result.replace("</h6>", "\n");
    result = result.replace("</li>", "\n");
    result = result.replace("<li>", "• ");
    let mut in_tag = false;
    let mut cleaned = String::with_capacity(result.len());
    for c in result.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            cleaned.push(c);
        }
    }
    return cleaned.lines().map(|line| line.trim()).filter(|line| !line.is_empty()).collect::<Vec<_>>().join("\n");
}
/// Driver for sending emails via SMTP
#[derive(Debug)]
pub struct SendEmailDriver;
#[async_trait::async_trait]
impl Driver for SendEmailDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "send_email";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Send an email via SMTP server";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this driver when the user wants to send an email, notify someone via email";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Email;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "smtp_host".to_string(),
                param_type: "string".to_string(),
                description: "SMTP server host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("smtp.gmail.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "smtp_port".to_string(),
                param_type: "integer".to_string(),
                description: "SMTP server port".to_string(),
                required: false,
                default: Some(Value::Number(587.into())),
                example: Some(Value::Number(587.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "SMTP username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user@gmail.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "SMTP password".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("your_password".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "from".to_string(),
                param_type: "string".to_string(),
                description: "Sender email address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("bot@example.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "to".to_string(),
                param_type: "string".to_string(),
                description: "Recipient email address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user@example.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "subject".to_string(),
                param_type: "string".to_string(),
                description: "Email subject line".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello from Hippo".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "body".to_string(),
                param_type: "string".to_string(),
                description: "Email body content".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("This is a test email".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "cc".to_string(),
                param_type: "string".to_string(),
                description: "CC recipient".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("cc@example.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "bcc".to_string(),
                param_type: "string".to_string(),
                description: "BCC recipient".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("bcc@example.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "is_html".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether the body is HTML".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Connection timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "send_email",
            "parameters": {
                "smtp_host": "smtp.gmail.com",
                "username": "user@gmail.com",
                "password": "password",
                "from": "bot@example.com",
                "to": "user@example.com",
                "subject": "Hello",
                "body": "Test email"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Email sent successfully to user@example.com".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing send_email driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting email send".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(5), None);
        }
        // Extract required parameters
        let smtp_host = get_param_string(parameters, "smtp_host")?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("SMTP host: {}", smtp_host)));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        let smtp_port = get_param_u64(parameters, "smtp_port", 587) as u16;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("SMTP port: {}", smtp_port)));
            cb.on_progress(task_id.clone(), driver_index, Some(15), None);
        }
        let username = get_param_string(parameters, "username")?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Username: {}", username)));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        let password = get_param_string(parameters, "password")?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Password provided".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(25), None);
        }
        let from_addr = get_param_string(parameters, "from")?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("From: {}", from_addr)));
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        let to_addr = get_param_string(parameters, "to")?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("To: {}", to_addr)));
            cb.on_progress(task_id.clone(), driver_index, Some(35), None);
        }
        let subject = get_param_string(parameters, "subject")?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Subject: {}", subject)));
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        let body = get_param_string(parameters, "body")?;
        if let Some(cb) = callback {
            let body_preview = if body.len() > 50 { format!("{}...", &body[..50]) } else { body.clone() };
            cb.on_log(task_id.clone(), driver_index, Some(format!("Body: {}", body_preview)));
            cb.on_progress(task_id.clone(), driver_index, Some(45), None);
        }
        let cc = parameters.get("cc").and_then(|v| v.as_str());
        if let Some(cb) = callback {
            if let Some(cc_addr) = cc {
                cb.on_log(task_id.clone(), driver_index, Some(format!("CC: {}", cc_addr)));
            }
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        let bcc = parameters.get("bcc").and_then(|v| v.as_str());
        if let Some(cb) = callback {
            if let Some(bcc_addr) = bcc {
                cb.on_log(task_id.clone(), driver_index, Some(format!("BCC: {}", bcc_addr)));
            }
            cb.on_progress(task_id.clone(), driver_index, Some(55), None);
        }
        let is_html = get_param_bool(parameters, "is_html", true);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("HTML format: {}", is_html)));
            cb.on_progress(task_id.clone(), driver_index, Some(60), None);
        }
        let timeout_secs = get_param_u64(parameters, "timeout", 30);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Timeout: {}s", timeout_secs)));
            cb.on_progress(task_id.clone(), driver_index, Some(65), None);
        }
        // Build and send email
        use lettre::message::Mailbox;
        use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, transport::smtp::authentication::Credentials};
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Building email message".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(70), None);
        }
        let to_parsed: Mailbox = to_addr.parse().map_err(|e| DriverError::execution(format!("Invalid 'to' address: {}", e)))?;
        let from_parsed: Mailbox = from_addr.parse().map_err(|e| DriverError::execution(format!("Invalid 'from' address: {}", e)))?;
        let mut email_builder = Message::builder().from(from_parsed).to(to_parsed).subject(subject);
        if let Some(cc_addr) = cc {
            email_builder = email_builder.cc(cc_addr.parse().map_err(|e| DriverError::execution(format!("Invalid CC address: {}", e)))?);
        }
        if let Some(bcc_addr) = bcc {
            email_builder = email_builder.bcc(bcc_addr.parse().map_err(|e| DriverError::execution(format!("Invalid BCC address: {}", e)))?);
        }
        let email = if is_html {
            let plain_body = strip_html_tags(&body);
            email_builder
                .multipart(lettre::message::MultiPart::alternative_plain_html(plain_body, body))
                .map_err(|e| DriverError::execution(format!("Failed to build multipart email: {}", e)))?
        } else {
            email_builder.body(body).map_err(|e| DriverError::execution(format!("Failed to build email body: {}", e)))?
        };
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Connecting to SMTP server".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(75), None);
        }
        let creds = Credentials::new(username, password);
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
            .map_err(|e| DriverError::execution(format!("Failed to create SMTP relay: {}", e)))?
            .port(smtp_port)
            .credentials(creds)
            .build();
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Sending email".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(80), None);
        }
        tokio::time::timeout(Duration::from_secs(timeout_secs), mailer.send(email))
            .await
            .map_err(|_| DriverError::execution(format!("Email send timeout after {} seconds", timeout_secs)))?
            .map_err(|e| DriverError::execution(format!("SMTP error: {}", e)))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Email sent to {}", to_addr)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("send_email".to_string()), Some(format!("Email sent successfully to {}", to_addr)));
        }
        let result = format!("Email sent successfully to {}", to_addr);
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        get_param_string(parameters, "smtp_host")?;
        get_param_string(parameters, "username")?;
        get_param_string(parameters, "password")?;
        get_param_string(parameters, "from")?;
        get_param_string(parameters, "to")?;
        get_param_string(parameters, "subject")?;
        get_param_string(parameters, "body")?;
        return Ok(());
    }
}
