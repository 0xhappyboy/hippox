use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    config::{get_smtp_instance, list_smtp_instances},
    executors::types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct SendEmailSkill;

#[async_trait::async_trait]
impl Skill for SendEmailSkill {
    fn name(&self) -> &str {
        "send_email"
    }

    fn description(&self) -> &str {
        "Send an email via SMTP server"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to send an email, notify someone via email, or send a message to an email address"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_smtp_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "SMTP instance ID (use 'list_smtp_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("smtp_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "to".to_string(),
                param_type: "string".to_string(),
                description: "Recipient email address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user@example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "subject".to_string(),
                param_type: "string".to_string(),
                description: "Email subject line".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello from Hippo".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "body".to_string(),
                param_type: "string".to_string(),
                description: "Email body content (supports HTML)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "<h1>Hello</h1><p>This is a test email.</p>".to_string(),
                )),
                enum_values: None,
            },
            SkillParameter {
                name: "from".to_string(),
                param_type: "string".to_string(),
                description: "Sender email address (overrides instance default)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("bot@example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "cc".to_string(),
                param_type: "string".to_string(),
                description: "CC recipient email address".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("cc@example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "bcc".to_string(),
                param_type: "string".to_string(),
                description: "BCC recipient email address".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("bcc@example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "is_html".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether the body is HTML (default: true)".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "SMTP server host (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("smtp.gmail.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "SMTP server port (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(587.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "SMTP username (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("user@gmail.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "SMTP password (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("your_password".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "send_email",
            "parameters": {
                "instance_id": "smtp_prod",
                "to": "user@example.com",
                "subject": "Hello",
                "body": "This is a test email"
            }
        })
    }

    fn example_output(&self) -> String {
        "Email sent successfully to user@example.com [instance: smtp_prod]".to_string()
    }

    fn category(&self) -> &str {
        "messaging"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let to = parameters
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'to' parameter"))?;
        let subject = parameters
            .get("subject")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'subject' parameter"))?;
        let body = parameters
            .get("body")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'body' parameter"))?;

        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());

        // Get instance configuration
        let instance = if let Some(id) = instance_id {
            get_smtp_instance(id)
                .ok_or_else(|| anyhow::anyhow!("SMTP instance not found: {}", id))?
        } else {
            let instances = list_smtp_instances();
            instances.into_iter().next().ok_or_else(|| {
                anyhow::anyhow!("No SMTP instance configured. Please add an SMTP instance first.")
            })?
        };

        // Parameter priority: parameter > instance config
        let smtp_host = parameters
            .get("host")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| instance.host.as_str());
        let smtp_port = parameters
            .get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or(instance.port.into()) as u16;
        let smtp_username = parameters
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| instance.username.as_str());
        let smtp_password = parameters
            .get("password")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| instance.password.as_str());
        let from_override = parameters
            .get("from")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let cc = parameters.get("cc").and_then(|v| v.as_str());
        let bcc = parameters.get("bcc").and_then(|v| v.as_str());
        let is_html = parameters
            .get("is_html")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let smtp_from = if from_override.is_empty() {
            instance.from.as_str()
        } else {
            from_override
        };

        if smtp_host.is_empty() {
            anyhow::bail!("SMTP host not configured for instance: {}", instance.name);
        }

        use lettre::message::MultiPart;
        use lettre::{
            AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::Mailbox,
            transport::smtp::authentication::Credentials,
        };

        let to_addr: Mailbox = to.parse()?;
        let from_addr: Mailbox = smtp_from.parse()?;
        let mut email_builder = Message::builder()
            .from(from_addr)
            .to(to_addr)
            .subject(subject);
        if let Some(cc_addr) = cc {
            let cc_parsed: Mailbox = cc_addr.parse()?;
            email_builder = email_builder.cc(cc_parsed);
        }
        if let Some(bcc_addr) = bcc {
            let bcc_parsed: Mailbox = bcc_addr.parse()?;
            email_builder = email_builder.bcc(bcc_parsed);
        }

        let email = if is_html {
            email_builder.multipart(MultiPart::alternative_plain_html(
                String::new(),
                body.to_string(),
            ))?
        } else {
            email_builder.body(body.to_string())?
        };

        let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(smtp_host)?
            .port(smtp_port)
            .credentials(creds)
            .build();

        match mailer.send(email).await {
            Ok(_) => Ok(format!(
                "Email sent successfully to {} [instance: {}]",
                to, instance.name
            )),
            Err(e) => Err(anyhow::anyhow!("Failed to send email: {}", e)),
        }
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: to"))?;
        parameters
            .get("subject")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: subject"))?;
        parameters
            .get("body")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: body"))?;
        Ok(())
    }
}
