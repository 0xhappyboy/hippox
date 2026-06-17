use crate::SkillCallback;
use crate::SkillContext;
use crate::{Skill, SkillCategory, SkillParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::net::ToSocketAddrs;
use std::path::Path;
use suppaftp::types::{FileType, FormatControl};

fn get_param_string(params: &HashMap<String, Value>, name: &str) -> Result<String> {
    params
        .get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: {}", name))
}

fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    params.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
}

/// FTP Upload Skill
#[derive(Debug)]
pub struct FtpUploadSkill;

#[async_trait::async_trait]
impl Skill for FtpUploadSkill {
    fn name(&self) -> &str {
        "ftp_upload"
    }
    fn description(&self) -> &str {
        "Upload a file to FTP server"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to upload files to an FTP server"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::Network
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "FTP server hostname or IP address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ftp.example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "FTP server port".to_string(),
                required: false,
                default: Some(Value::Number(21.into())),
                example: Some(Value::Number(21.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "FTP username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user@example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "FTP password".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("secret123".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "local_path".to_string(),
                param_type: "string".to_string(),
                description: "Local file path to upload".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/home/user/document.pdf".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "remote_path".to_string(),
                param_type: "string".to_string(),
                description: "Remote path where to upload the file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/uploads/document.pdf".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "mode".to_string(),
                param_type: "string".to_string(),
                description: "Transfer mode (binary or ascii)".to_string(),
                required: false,
                default: Some(Value::String("binary".to_string())),
                example: Some(Value::String("binary".to_string())),
                enum_values: Some(vec!["binary".to_string(), "ascii".to_string()]),
            },
            SkillParameter {
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
        json!({ "action": "ftp_upload", "parameters": { "host": "ftp.example.com", "username": "user", "password": "pass", "local_path": "/tmp/file.txt", "remote_path": "/uploads/file.txt" } })
    }

    fn example_output(&self) -> String {
        "Successfully uploaded /tmp/file.txt to ftp.example.com:/uploads/file.txt (1024 bytes)"
            .to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        use suppaftp::FtpStream;

        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 21) as u16;
        let username = get_param_string(parameters, "username")?;
        let password = get_param_string(parameters, "password")?;
        let local_path = get_param_string(parameters, "local_path")?;
        let remote_path = get_param_string(parameters, "remote_path")?;
        let timeout = get_param_u64(parameters, "timeout", 30);
        let mode = parameters
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("binary");

        if !Path::new(&local_path).exists() {
            anyhow::bail!("Local file not found: {}", local_path);
        }
        let file_size = fs::metadata(&local_path)?.len();

        let addr_str = format!("{}:{}", host, port);
        let addr = addr_str
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid address: {}", addr_str))?;
        let mut ftp = FtpStream::connect_timeout(addr, std::time::Duration::from_secs(timeout))?;
        ftp.login(&username, &password)?;

        if mode == "binary" {
            ftp.transfer_type(FileType::Binary)?;
        } else {
            ftp.transfer_type(FileType::Ascii(FormatControl::NonPrint))?;
        }

        if let Some(parent) = Path::new(&remote_path).parent() {
            let parent_str = parent.to_string_lossy();
            if !parent_str.is_empty() && parent_str != "/" {
                let _ = ftp.mkdir(&parent_str);
            }
        }

        let mut file = fs::File::open(&local_path)?;
        ftp.put_file(&remote_path, &mut file)?;
        ftp.quit()?;

        Ok(format!(
            "Successfully uploaded {} to {}:{}{} ({} bytes)",
            local_path, host, port, remote_path, file_size
        ))
    }
}

/// FTP Download Skill
#[derive(Debug)]
pub struct FtpDownloadSkill;

#[async_trait::async_trait]
impl Skill for FtpDownloadSkill {
    fn name(&self) -> &str {
        "ftp_download"
    }
    fn description(&self) -> &str {
        "Download a file from FTP server"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to download files from an FTP server"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::Network
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "FTP server hostname or IP address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ftp.example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "FTP server port".to_string(),
                required: false,
                default: Some(Value::Number(21.into())),
                example: Some(Value::Number(21.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "FTP username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user@example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "FTP password".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("secret123".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "remote_path".to_string(),
                param_type: "string".to_string(),
                description: "Remote file path to download".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/uploads/document.pdf".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "local_path".to_string(),
                param_type: "string".to_string(),
                description: "Local path where to save the file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/home/user/document.pdf".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "mode".to_string(),
                param_type: "string".to_string(),
                description: "Transfer mode (binary or ascii)".to_string(),
                required: false,
                default: Some(Value::String("binary".to_string())),
                example: Some(Value::String("binary".to_string())),
                enum_values: Some(vec!["binary".to_string(), "ascii".to_string()]),
            },
            SkillParameter {
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
        json!({ "action": "ftp_download", "parameters": { "host": "ftp.example.com", "username": "user", "password": "pass", "remote_path": "/uploads/file.txt", "local_path": "/tmp/file.txt" } })
    }

    fn example_output(&self) -> String {
        "Successfully downloaded ftp.example.com:/uploads/file.txt to /tmp/file.txt (1024 bytes)"
            .to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        use suppaftp::FtpStream;

        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 21) as u16;
        let username = get_param_string(parameters, "username")?;
        let password = get_param_string(parameters, "password")?;
        let remote_path = get_param_string(parameters, "remote_path")?;
        let local_path = get_param_string(parameters, "local_path")?;
        let timeout = get_param_u64(parameters, "timeout", 30);

        let addr_str = format!("{}:{}", host, port);
        let addr = addr_str
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid address: {}", addr_str))?;
        let mut ftp = FtpStream::connect_timeout(addr, std::time::Duration::from_secs(timeout))?;
        ftp.login(&username, &password)?;

        if let Some(parent) = Path::new(&local_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut stream = ftp.retr_as_stream(&remote_path)?;
        let mut content = Vec::new();
        std::io::copy(&mut stream, &mut content)?;
        fs::write(&local_path, content)?;
        let file_size = fs::metadata(&local_path)?.len();
        ftp.quit()?;

        Ok(format!(
            "Successfully downloaded {}:{}{} to {} ({} bytes)",
            host, port, remote_path, local_path, file_size
        ))
    }
}

/// FTP List Skill
#[derive(Debug)]
pub struct FtpListSkill;

#[async_trait::async_trait]
impl Skill for FtpListSkill {
    fn name(&self) -> &str {
        "ftp_list"
    }
    fn description(&self) -> &str {
        "List directory contents on FTP server"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to see what files are in an FTP directory"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::Network
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "FTP server hostname or IP address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ftp.example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "FTP server port".to_string(),
                required: false,
                default: Some(Value::Number(21.into())),
                example: Some(Value::Number(21.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "FTP username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user@example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "FTP password".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("secret123".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "directory".to_string(),
                param_type: "string".to_string(),
                description: "Remote directory to list".to_string(),
                required: false,
                default: Some(Value::String("/".to_string())),
                example: Some(Value::String("/uploads".to_string())),
                enum_values: None,
            },
            SkillParameter {
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
        json!({ "action": "ftp_list", "parameters": { "host": "ftp.example.com", "username": "user", "password": "pass", "directory": "/uploads" } })
    }

    fn example_output(&self) -> String {
        "Directory listing for /uploads:\nfile1.txt (1024 bytes)\nfile2.pdf (51200 bytes)\nfolder/"
            .to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        use suppaftp::FtpStream;

        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 21) as u16;
        let username = get_param_string(parameters, "username")?;
        let password = get_param_string(parameters, "password")?;
        let directory = parameters
            .get("directory")
            .and_then(|v| v.as_str())
            .unwrap_or("/");
        let timeout = get_param_u64(parameters, "timeout", 30);

        let addr_str = format!("{}:{}", host, port);
        let addr = addr_str
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid address: {}", addr_str))?;
        let mut ftp = FtpStream::connect_timeout(addr, std::time::Duration::from_secs(timeout))?;
        ftp.login(&username, &password)?;

        let listing = ftp.list(Some(directory))?;
        let mut result = format!("Directory listing for {}:\n", directory);
        for line in listing {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                let size = parts[4];
                let name = parts[8..].join(" ");
                if line.starts_with('d') {
                    result.push_str(&format!("{}/\n", name));
                } else {
                    result.push_str(&format!("{} ({} bytes)\n", name, size));
                }
            } else {
                result.push_str(&format!("{}\n", line));
            }
        }
        ftp.quit()?;
        Ok(result.trim_end().to_string())
    }
}

/// FTP Delete Skill
#[derive(Debug)]
pub struct FtpDeleteSkill;

#[async_trait::async_trait]
impl Skill for FtpDeleteSkill {
    fn name(&self) -> &str {
        "ftp_delete"
    }
    fn description(&self) -> &str {
        "Delete a file from FTP server"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to delete files on an FTP server"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::Network
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "FTP server hostname or IP address".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ftp.example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "FTP server port".to_string(),
                required: false,
                default: Some(Value::Number(21.into())),
                example: Some(Value::Number(21.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "FTP username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user@example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "FTP password".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("secret123".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "remote_path".to_string(),
                param_type: "string".to_string(),
                description: "Remote file path to delete".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/uploads/old_file.txt".to_string())),
                enum_values: None,
            },
            SkillParameter {
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
        json!({ "action": "ftp_delete", "parameters": { "host": "ftp.example.com", "username": "user", "password": "pass", "remote_path": "/uploads/old_file.txt" } })
    }

    fn example_output(&self) -> String {
        "Successfully deleted /uploads/old_file.txt from ftp.example.com".to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        use suppaftp::FtpStream;

        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 21) as u16;
        let username = get_param_string(parameters, "username")?;
        let password = get_param_string(parameters, "password")?;
        let remote_path = get_param_string(parameters, "remote_path")?;
        let timeout = get_param_u64(parameters, "timeout", 30);

        let addr_str = format!("{}:{}", host, port);
        let addr = addr_str
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid address: {}", addr_str))?;
        let mut ftp = FtpStream::connect_timeout(addr, std::time::Duration::from_secs(timeout))?;
        ftp.login(&username, &password)?;
        ftp.rm(&remote_path)?;
        ftp.quit()?;

        Ok(format!(
            "Successfully deleted {} from {}:{}",
            remote_path, host, port
        ))
    }
}
