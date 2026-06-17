use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCallback, SkillCategory, SkillContext, types::{Skill, SkillParameter}
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
        "Use this skill when the user wants to send an email, notify someone via email"
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Email
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "smtp_host".to_string(),
                param_type: "string".to_string(),
                description: "SMTP server host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("smtp.gmail.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "smtp_port".to_string(),
                param_type: "integer".to_string(),
                description: "SMTP server port".to_string(),
                required: false,
                default: Some(Value::Number(587.into())),
                example: Some(Value::Number(587.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "SMTP username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user@gmail.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "SMTP password".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("your_password".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "from".to_string(),
                param_type: "string".to_string(),
                description: "Sender email address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("bot@example.com".to_string())),
                enum_values: None,
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
                description: "Email body content".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("This is a test email".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "cc".to_string(),
                param_type: "string".to_string(),
                description: "CC recipient".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("cc@example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "bcc".to_string(),
                param_type: "string".to_string(),
                description: "BCC recipient".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("bcc@example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "is_html".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether the body is HTML".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
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
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let smtp_host = get_param_string(parameters, "smtp_host")?;
        let smtp_port = get_param_u64(parameters, "smtp_port", 587) as u16;
        let username = get_param_string(parameters, "username")?;
        let password = get_param_string(parameters, "password")?;
        let from_addr = get_param_string(parameters, "from")?;
        let to_addr = get_param_string(parameters, "to")?;
        let subject = get_param_string(parameters, "subject")?;
        let body = get_param_string(parameters, "body")?;
        let cc = parameters.get("cc").and_then(|v| v.as_str());
        let bcc = parameters.get("bcc").and_then(|v| v.as_str());
        let is_html = get_param_bool(parameters, "is_html", true);
        use lettre::message::Mailbox;
        use lettre::{
            AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
            transport::smtp::authentication::Credentials,
        };
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
            email_builder.multipart(lettre::message::MultiPart::alternative_plain_html(
                String::new(),
                body,
            ))?
        } else {
            email_builder.body(body)?
        };
        let creds = Credentials::new(username, password);
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)?
            .port(smtp_port)
            .credentials(creds)
            .build();
        mailer.send(email).await?;
        Ok(format!("Email sent successfully to {}", to_addr))
    }
}
