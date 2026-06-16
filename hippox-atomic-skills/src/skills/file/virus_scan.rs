//! Virus scan skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use crate::{file_exists, scan_file_for_viruses, validate_path};

/// Common virus signatures (hex patterns)
/// In production, this would be a much larger database
pub const VIRUS_SIGNATURES: &[&str] = &[
    // EICAR test virus (harmless test pattern)
    "5844454f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f",
    // Common malware patterns (simplified for demonstration)
    "4d5a90000300000004000000ffff0000", // PE header with suspicious flags
    "7f454c46010101000000000000000000", // ELF header with suspicious flags
    "5a4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d", // Suspicious pattern
    "42494949494949494949494949494949", // Suspicious pattern
    "43434343434343434343434343434343", // Suspicious pattern
    "44444444444444444444444444444444", // Suspicious pattern
    "45454545454545454545454545454545", // Suspicious pattern
];

#[derive(Debug)]
pub struct VirusScanSkill;

#[async_trait::async_trait]
impl Skill for VirusScanSkill {
    fn name(&self) -> &str {
        "file_virus_scan"
    }

    fn description(&self) -> &str {
        "Scan a file for viruses using signature-based detection"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to scan a file for known virus signatures. Note: This is a basic signature-based scanner, not a full antivirus solution."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the file to scan".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/suspicious_file.exe".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "recursive".to_string(),
                param_type: "boolean".to_string(),
                description: "Scan directory recursively (default: false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "file_virus_scan",
            "parameters": {
                "path": "/tmp/suspicious_file.exe",
                "recursive": false
            }
        })
    }

    fn example_output(&self) -> String {
        "INFECTED: /tmp/suspicious_file.exe\nVirus: Virus signature: 4d5a9000...\nFile size: 1024 bytes\nScan time: 2024-01-01 00:00:00".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::File
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let recursive = parameters
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let validated_path = validate_path(path, None)?;
        if !validated_path.exists() {
            anyhow::bail!("Path not found: {}", path);
        }

        let mut results = Vec::new();

        if validated_path.is_file() {
            let result =
                scan_file_for_viruses(&validated_path.to_string_lossy(), VIRUS_SIGNATURES)?;
            results.push(result);
        } else if validated_path.is_dir() && recursive {
            for entry in walkdir::WalkDir::new(&validated_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let file_path = entry.path().to_string_lossy().to_string();
                if let Ok(result) = scan_file_for_viruses(&file_path, VIRUS_SIGNATURES) {
                    results.push(result);
                }
            }
        } else {
            anyhow::bail!("Path is a directory. Use recursive=true to scan directory contents.");
        }

        if results.is_empty() {
            return Ok("No files scanned".to_string());
        }

        let mut output = String::new();
        let mut infected_count = 0;
        let mut clean_count = 0;

        for result in results {
            let status = if result.infected { "INFECTED" } else { "CLEAN" };
            if result.infected {
                infected_count += 1;
            } else {
                clean_count += 1;
            }

            output.push_str(&format!("{}: {}\n", status, result.path));
            if let Some(virus) = result.virus_name {
                output.push_str(&format!("  Virus: {}\n", virus));
            }
            output.push_str(&format!("  Size: {} bytes\n", result.file_size));
            output.push_str(&format!("  Scan time: {}\n", result.scan_time));
        }

        output.push_str(&format!(
            "\nSummary: {} infected, {} clean",
            infected_count, clean_count
        ));

        if infected_count > 0 {
            output.push_str("\nWARNING: Infected files detected!");
        } else {
            output.push_str("\nNo viruses detected.");
        }

        Ok(output)
    }
}
