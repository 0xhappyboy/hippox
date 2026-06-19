use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};

fn get_param_string(params: &HashMap<String, Value>, name: &str) -> Result<String> {
    params
        .get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: {}", name))
}

fn get_param_bool(params: &HashMap<String, Value>, name: &str, default: bool) -> bool {
    params
        .get(name)
        .and_then(|v| v.as_bool())
        .unwrap_or(default)
}

fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    params.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
}

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
    cleaned
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug)]
pub struct SendEmailDriver;

#[async_trait::async_trait]
impl Driver for SendEmailDriver {
    fn name(&self) -> &str {
        "send_email"
    }

    fn description(&self) -> &str {
        "Send an email via SMTP server"
    }

    fn usage_hint(&self) -> &str {
        "Use this driver when the user wants to send an email, notify someone via email"
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Email
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "send_email", "parameters": { "smtp_host": "smtp.gmail.com", "username": "user@gmail.com", "password": "password", "from": "bot@example.com", "to": "user@example.com", "subject": "Hello", "body": "Test email" } })
    }

    fn example_output(&self) -> String {
        "Email sent successfully to user@example.com".to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.driver_name())
            .map(String::from);
        let cb = callback;
        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Starting email send".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(5), None);
        }
        let smtp_host = get_param_string(parameters, "smtp_host")?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("SMTP host: {}", smtp_host)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        let smtp_port = get_param_u64(parameters, "smtp_port", 587) as u16;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("SMTP port: {}", smtp_port)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(15), None);
        }
        let username = get_param_string(parameters, "username")?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Username: {}", username)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        let password = get_param_string(parameters, "password")?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Password provided".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(25), None);
        }
        let from_addr = get_param_string(parameters, "from")?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("From: {}", from_addr)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        let to_addr = get_param_string(parameters, "to")?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("To: {}", to_addr)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(35), None);
        }
        let subject = get_param_string(parameters, "subject")?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Subject: {}", subject)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        let body = get_param_string(parameters, "body")?;
        if let Some(cb) = cb {
            let body_preview = if body.len() > 50 {
                format!("{}...", &body[..50])
            } else {
                body.clone()
            };
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Body: {}", body_preview)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(45), None);
        }
        let cc = parameters.get("cc").and_then(|v| v.as_str());
        if let Some(cb) = cb {
            if let Some(cc_addr) = cc {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some(format!("CC: {}", cc_addr)),
                );
            }
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        let bcc = parameters.get("bcc").and_then(|v| v.as_str());
        if let Some(cb) = cb {
            if let Some(bcc_addr) = bcc {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some(format!("BCC: {}", bcc_addr)),
                );
            }
            cb.on_progress(task_id.clone(), driver_index, Some(55), None);
        }
        let is_html = get_param_bool(parameters, "is_html", true);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("HTML format: {}", is_html)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(60), None);
        }
        let timeout_secs = get_param_u64(parameters, "timeout", 30);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Timeout: {}s", timeout_secs)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(65), None);
        }
        use lettre::message::Mailbox;
        use lettre::{
            AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
            transport::smtp::authentication::Credentials,
        };
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Building email message".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(70), None);
        }
        let to_parsed: Mailbox = to_addr.parse()?;
        let from_parsed: Mailbox = from_addr.parse()?;
        let mut email_builder = Message::builder()
            .from(from_parsed)
            .to(to_parsed)
            .subject(subject);
        if let Some(cc_addr) = cc {
            email_builder = email_builder.cc(cc_addr.parse()?);
        }
        if let Some(bcc_addr) = bcc {
            email_builder = email_builder.bcc(bcc_addr.parse()?);
        }
        let email = if is_html {
            let plain_body = strip_html_tags(&body);
            email_builder.multipart(lettre::message::MultiPart::alternative_plain_html(
                plain_body, body,
            ))?
        } else {
            email_builder.body(body)?
        };
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Connecting to SMTP server".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(75), None);
        }
        let creds = Credentials::new(username, password);
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)?
            .port(smtp_port)
            .credentials(creds)
            .build();
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Sending email".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(80), None);
        }
        tokio::time::timeout(Duration::from_secs(timeout_secs), mailer.send(email))
            .await
            .map_err(|_| anyhow::anyhow!("Email send timeout after {} seconds", timeout_secs))?
            .map_err(|e| anyhow::anyhow!("SMTP error: {}", e))?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Email sent to {}", to_addr)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                driver_index,
                Some("send_email".to_string()),
                Some(format!("Email sent successfully to {}", to_addr)),
            );
        }
        Ok(format!("Email sent successfully to {}", to_addr))
    }
}
