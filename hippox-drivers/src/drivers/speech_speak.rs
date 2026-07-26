//! Speech speak driver - text to speech synthesis
//!
//! This driver provides text-to-speech capabilities across all platforms,
//! allowing the computer to speak text aloud with configurable voice, rate, and volume.
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
/// Driver for text-to-speech synthesis
#[derive(Debug)]
pub struct SpeechSpeakDriver;
#[async_trait::async_trait]
impl Driver for SpeechSpeakDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "speech_speak";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Convert text to speech and speak it aloud through the computer's speakers";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this driver when the user wants the computer to speak, read text aloud, \
         provide voice feedback, or announce something. Supports voice selection, \
         speaking rate adjustment, and volume control.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "The text to speak aloud".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello, I am your AI assistant".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "voice".to_string(),
                param_type: "string".to_string(),
                description: "Voice to use. Platform specific: Windows (e.g., 'Microsoft Zira', 'Microsoft Huihui'), macOS (e.g., 'Samantha', 'Ting-Ting'), Linux (e.g., 'en-us')".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(Value::String("Microsoft Zira".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "rate".to_string(),
                param_type: "integer".to_string(),
                description: "Speaking rate/speed. Range: -10 to 10 (Windows), 0 to 10 (macOS/Linux). Default: 0".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(2.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "volume".to_string(),
                param_type: "integer".to_string(),
                description: "Speaking volume. Range: 0 to 100. Default: 100".to_string(),
                required: false,
                default: Some(Value::Number(100.into())),
                example: Some(Value::Number(75.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "style".to_string(),
                param_type: "string".to_string(),
                description: "Speaking style/emotion (Windows only): 'normal', 'cheerful', 'sad', 'angry', 'fearful', 'disdainful'".to_string(),
                required: false,
                default: Some(Value::String("normal".to_string())),
                example: Some(Value::String("cheerful".to_string())),
                enum_values: Some(vec![
                    "normal".to_string(),
                    "cheerful".to_string(),
                    "sad".to_string(),
                    "angry".to_string(),
                    "fearful".to_string(),
                    "disdainful".to_string(),
                ]),
            },
            DriverParameter {
                name: "async".to_string(),
                param_type: "boolean".to_string(),
                description: "Speak asynchronously (don't wait for completion). Default: false".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "speech_speak",
            "parameters": {
                "text": "Hello, I am Hippox",
                "voice": "default",
                "rate": 1,
                "volume": 100
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Speaking: Hello, I am Hippox".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::SpeechSpeak;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing speech_speak driver");
        // Extract required parameters
        let text = parameters.get("text").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("text"))?;
        let voice = parameters.get("voice").and_then(|v| v.as_str()).unwrap_or("default");
        let rate = parameters.get("rate").and_then(|v| v.as_i64()).unwrap_or(0);
        let volume = parameters.get("volume").and_then(|v| v.as_i64()).unwrap_or(100);
        let style = parameters.get("style").and_then(|v| v.as_str()).unwrap_or("normal");
        let async_mode = parameters.get("async").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Speaking text: '{}', voice: {}, rate: {}, volume: {}, style: {}, async: {}", text, voice, rate, volume, style, async_mode);
        #[cfg(target_os = "windows")]
        {
            speak_windows(text, voice, rate, volume, style, async_mode)?;
        }
        #[cfg(target_os = "macos")]
        {
            speak_macos(text, voice, rate, volume, async_mode)?;
        }
        #[cfg(target_os = "linux")]
        {
            speak_linux(text, voice, rate, volume, async_mode)?;
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            return Err(DriverError::execution("Speech not supported on this platform".to_string()));
        }
        let result = format!("Speaking: {}", text);
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("text").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("text"))?;
        return Ok(());
    }
}
#[cfg(target_os = "windows")]
fn speak_windows(text: &str, voice: &str, rate: i64, volume: i64, style: &str, async_mode: bool) -> DriverResult<()> {
    use std::process::Command;
    debug!("Speaking on Windows with PowerShell");
    let escaped_text = text.replace('\'', "''");
    let mut cmd_script = String::new();
    cmd_script.push_str("Add-Type -AssemblyName System.Speech; ");
    cmd_script.push_str("$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer; ");
    if voice != "default" {
        cmd_script.push_str(&format!("$synth.SelectVoice('{}'); ", voice.replace('\'', "''")));
    }
    let rate_clamped = rate.clamp(-10, 10);
    if rate_clamped != 0 {
        cmd_script.push_str(&format!("$synth.Rate = {}; ", rate_clamped));
    }
    let volume_clamped = volume.clamp(0, 100);
    if volume_clamped != 100 {
        cmd_script.push_str(&format!("$synth.Volume = {}; ", volume_clamped));
    }
    if style != "normal" {
        let style_map = match style {
            "cheerful" => "[System.Speech.Synthesis.PromoteEmphasis]::Cheerful",
            "sad" => "[System.Speech.Synthesis.PromoteEmphasis]::Sad",
            "angry" => "[System.Speech.Synthesis.PromoteEmphasis]::Angry",
            "fearful" => "[System.Speech.Synthesis.PromoteEmphasis]::Fearful",
            "disdainful" => "[System.Speech.Synthesis.PromoteEmphasis]::Disdainful",
            _ => "[System.Speech.Synthesis.PromoteEmphasis]::Normal",
        };
        cmd_script.push_str(&format!(
            "$builder = New-Object System.Speech.Synthesis.PromptBuilder; \
             $builder.StartStyle(${}); \
             $builder.AppendText('{}'); \
             $builder.EndStyle(); \
             $synth.Speak($builder); ",
            style_map, escaped_text
        ));
    } else {
        cmd_script.push_str(&format!("$synth.Speak('{}'); ", escaped_text));
    }
    let mut cmd = Command::new("powershell");
    cmd.args(&["-Command", &cmd_script]);
    if async_mode {
        cmd.spawn().map_err(|e| DriverError::execution(format!("Failed to spawn PowerShell: {}", e)))?;
    } else {
        let output = cmd.output().map_err(|e| DriverError::execution(format!("Failed to execute PowerShell: {}", e)))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DriverError::execution(format!("Failed to speak: {}", stderr)));
        }
    }
    info!("Speech completed on Windows");
    return Ok(());
}
#[cfg(target_os = "macos")]
fn speak_macos(text: &str, voice: &str, rate: i64, volume: i64, async_mode: bool) -> DriverResult<()> {
    use std::process::Command;
    debug!("Speaking on macOS with 'say' command");
    let mut cmd = Command::new("say");
    if voice != "default" {
        cmd.arg("-v").arg(voice);
    }
    if rate != 0 {
        let rate_clamped = rate.clamp(0, 10);
        let rate_value = 100 + (rate_clamped * 20);
        cmd.arg("-r").arg(rate_value.to_string());
    }
    if volume != 100 {
        let volume_clamped = volume.clamp(0, 100);
        let _ = Command::new("osascript").args(&["-e", &format!("set volume output volume {}", volume_clamped)]).output();
    }
    cmd.arg(text);
    if async_mode {
        cmd.spawn().map_err(|e| DriverError::execution(format!("Failed to spawn 'say' command: {}", e)))?;
    } else {
        let output = cmd.output().map_err(|e| DriverError::execution(format!("Failed to execute 'say' command: {}", e)))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DriverError::execution(format!("Failed to speak: {}", stderr)));
        }
    }
    // Restore volume if changed
    if volume != 100 {
        let _ = Command::new("osascript").args(&["-e", "set volume output volume 100"]).output();
    }
    info!("Speech completed on macOS");
    return Ok(());
}
#[cfg(target_os = "linux")]
fn speak_linux(text: &str, voice: &str, rate: i64, volume: i64, async_mode: bool) -> DriverResult<()> {
    use std::process::Command;
    debug!("Speaking on Linux");
    let mut cmd = None;
    // Try espeak-ng first
    if Command::new("espeak-ng").arg("--version").output().is_ok() {
        debug!("Using espeak-ng for speech");
        let mut c = Command::new("espeak-ng");
        if voice != "default" {
            c.arg("-v").arg(voice);
        }
        if rate != 0 {
            let rate_clamped = rate.clamp(-10, 10);
            let rate_value = 80 + ((rate_clamped + 10) * 8);
            c.arg("-s").arg(rate_value.to_string());
        }
        if volume != 100 {
            let volume_clamped = volume.clamp(0, 100);
            let volume_value = volume_clamped * 2;
            c.arg("-a").arg(volume_value.to_string());
        }
        c.arg(text);
        cmd = Some(c);
    }
    // Try espeak (legacy)
    else if Command::new("espeak").arg("--version").output().is_ok() {
        debug!("Using espeak for speech");
        let mut c = Command::new("espeak");
        if voice != "default" {
            c.arg("-v").arg(voice);
        }
        if rate != 0 {
            let rate_clamped = rate.clamp(-10, 10);
            let rate_value = 80 + ((rate_clamped + 10) * 8);
            c.arg("-s").arg(rate_value.to_string());
        }
        if volume != 100 {
            let volume_clamped = volume.clamp(0, 100);
            let volume_value = volume_clamped * 2;
            c.arg("-a").arg(volume_value.to_string());
        }
        c.arg(text);
        cmd = Some(c);
    }
    // Try spd-say (speech-dispatcher)
    else if Command::new("spd-say").arg("--version").output().is_ok() {
        debug!("Using spd-say for speech");
        let mut c = Command::new("spd-say");
        if rate != 0 {
            let rate_clamped = rate.clamp(-100, 100);
            c.arg("-r").arg(rate_clamped.to_string());
        }
        if volume != 100 {
            let volume_clamped = volume.clamp(0, 100);
            c.arg("-v").arg(volume_clamped.to_string());
        }
        c.arg(text);
        cmd = Some(c);
    }
    if let Some(mut cmd) = cmd {
        if async_mode {
            cmd.spawn().map_err(|e| DriverError::execution(format!("Failed to spawn speech command: {}", e)))?;
        } else {
            let output = cmd.output().map_err(|e| DriverError::execution(format!("Failed to execute speech command: {}", e)))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(DriverError::execution(format!("Failed to speak: {}", stderr)));
            }
        }
        info!("Speech completed on Linux");
        return Ok(());
    } else {
        return Err(DriverError::execution(format!(
            "No TTS engine found. Please install espeak, espeak-ng, or speech-dispatcher.\n\
             Ubuntu/Debian: sudo apt install espeak\n\
             Fedora: sudo dnf install espeak\n\
             Arch: sudo pacman -S espeak"
        )));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_speech_speak_metadata() {
        let driver = SpeechSpeakDriver;
        assert_eq!(driver.name(), "speech_speak");
        assert_eq!(driver.category(), DriverCategory::SpeechSpeak);
        assert!(!driver.description().is_empty());
        assert!(!driver.usage_hint().is_empty());
    }
    #[test]
    fn test_speech_speak_parameters() {
        let driver = SpeechSpeakDriver;
        let params = driver.parameters();
        assert_eq!(params.len(), 6);
        let param_names: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
        assert!(param_names.contains(&"text"));
        assert!(param_names.contains(&"voice"));
        assert!(param_names.contains(&"rate"));
        assert!(param_names.contains(&"volume"));
        assert!(param_names.contains(&"style"));
        assert!(param_names.contains(&"async"));
        let text_param = params.iter().find(|p| p.name == "text").unwrap();
        assert!(text_param.required);
        assert_eq!(text_param.param_type, "string");
        let voice_param = params.iter().find(|p| p.name == "voice").unwrap();
        assert!(!voice_param.required);
        assert_eq!(voice_param.default, Some(Value::String("default".to_string())));
    }
    #[test]
    fn test_speech_speak_validation() {
        let driver = SpeechSpeakDriver;
        let mut valid_params = HashMap::new();
        valid_params.insert("text".to_string(), Value::String("Hello".to_string()));
        assert!(driver.validate(&valid_params).is_ok());
        let empty_params = HashMap::new();
        assert!(driver.validate(&empty_params).is_err());
        let mut wrong_type = HashMap::new();
        wrong_type.insert("text".to_string(), Value::Number(123.into()));
        assert!(driver.validate(&wrong_type).is_err());
    }
}
