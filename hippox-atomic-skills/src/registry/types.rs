//! Skill category enumeration

use serde::{Deserialize, Serialize};

/// Skill category enumeration with metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillCategory {
    /// Basic example skills for demonstration and testing
    Basic,
    /// File read/write, directory operations, compression/extraction
    File,
    /// Mathematical operations, statistics, unit conversion, random numbers, hashing
    Math,
    /// HTTP requests, DNS lookup, Ping, TCP/UDP communication, FTP
    Network,
    /// System information, power management, environment variables, clipboard
    OperatingSystem,
    /// Process listing, start, terminate, monitoring
    OperatingSystemProcess,
    /// Process memory reading, scanning, module base address
    OperatingSystemMemory,
    /// Markdown, CSV, XML, Excel, PDF, JSON, YAML, TOML
    Document,
    /// DingTalk, Feishu, WeChat Work, Telegram
    SocialPlatform,
    /// PostgreSQL, MySQL, Redis, SQLite
    Database,
    /// Text comparison, sorting, deduplication, filtering, regex
    Text,
    /// Kubernetes, Docker, GitHub
    Devops,
    /// Image processing (resize, convert, crop, compress)
    Media,
    /// Bitcoin, EVM, Solana wallet operations
    Blockchain,
    /// have head browser navigation, clicking, form filling, screenshot, JS execution
    HaveHeadBrowser,
    /// Window minimize, maximize, move, close, always on top
    Window,
    /// Key presses, shortcuts, text input
    Keyboard,
    /// Mouse movement, clicking, dragging, scrolling
    Mouse,
    /// Volume control, device switching, recording, playback
    Audio,
    /// Application launch, close, install, uninstall
    Application,
    /// Monitor info, resolution, brightness, orientation, refresh rate
    Display,
    /// WiFi scan, connect, hotspot, configuration management
    Wifi,
    /// Bluetooth device scan, pair, connect, file transfer, BLE
    Bluetooth,
    /// Execute system commands and scripts
    Terminal,
    /// E-mail
    Email,
    /// scheduled tasks
    ScheduledTasks,
    /// time
    Time,
    /// cryptography
    Cryptography,
    /// speech speak
    /// Text-to-speech, voice broadcast
    SpeechSpeak,
    OperatingSystemServices,
    OperatingSystemSecurity,
}

impl SkillCategory {
    /// Convert string to SkillCategory enum
    ///
    /// # Important
    /// This function MUST be kept in sync with the `name()` method.
    /// The string patterns here must match exactly with what `name()` returns.
    /// These names are used as the category index for LLM intent classification.
    /// If `name()` returns `"file_ops"`, this function must accept `"file_ops"` as input.
    ///
    /// # Arguments
    /// * `s` - Category name string (e.g., "file_ops", "network_ops", "security_ops")
    ///
    /// # Returns
    /// `Some(SkillCategory)` if the string matches a category name, otherwise `None`
    ///
    /// # Examples
    /// ```
    /// let category = SkillCategory::from_str("file_ops");
    /// assert_eq!(category, Some(SkillCategory::File));
    /// ```
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "basic" => Some(SkillCategory::Basic),
            "file_ops" => Some(SkillCategory::File),
            "math_ops" => Some(SkillCategory::Math),
            "network_ops" => Some(SkillCategory::Network),
            "system_ops" => Some(SkillCategory::OperatingSystem),
            "process_ops" => Some(SkillCategory::OperatingSystemProcess),
            "memory_ops" => Some(SkillCategory::OperatingSystemMemory),
            "document_ops" => Some(SkillCategory::Document),
            "social_ops" => Some(SkillCategory::SocialPlatform),
            "database_ops" => Some(SkillCategory::Database),
            "text_ops" => Some(SkillCategory::Text),
            "devops_ops" => Some(SkillCategory::Devops),
            "media_ops" => Some(SkillCategory::Media),
            "blockchain_ops" => Some(SkillCategory::Blockchain),
            "browser_ops" => Some(SkillCategory::HaveHeadBrowser),
            "window_ops" => Some(SkillCategory::Window),
            "keyboard_ops" => Some(SkillCategory::Keyboard),
            "mouse_ops" => Some(SkillCategory::Mouse),
            "audio_ops" => Some(SkillCategory::Audio),
            "app_ops" => Some(SkillCategory::Application),
            "display_ops" => Some(SkillCategory::Display),
            "wifi_ops" => Some(SkillCategory::Wifi),
            "bluetooth_ops" => Some(SkillCategory::Bluetooth),
            "terminal_ops" => Some(SkillCategory::Terminal),
            "email_ops" => Some(SkillCategory::Email),
            "schedule_ops" => Some(SkillCategory::ScheduledTasks),
            "time_ops" => Some(SkillCategory::Time),
            "crypto_ops" => Some(SkillCategory::Cryptography),
            "tts_play_on_speaker" => Some(SkillCategory::SpeechSpeak),
            "service_ops" => Some(SkillCategory::OperatingSystemServices),
            "security_ops" => Some(SkillCategory::OperatingSystemSecurity),
            _ => None,
        }
    }

    /// Get the string representation of the category
    /// Returns the LLM-readable name identifier for this category.
    ///
    /// # Important
    /// This name is the primary key used for category identification in the LLM
    /// skill routing system. It is sent to the LLM during intent analysis to help
    /// the model select the appropriate skill category for a given user request.
    ///
    /// The naming convention uses descriptive, human-readable terms that clearly
    /// indicate the category's purpose (e.g., "file_ops", "network_ops", "security_ops")
    /// so the LLM can accurately match user intent to the correct category during
    /// the initial intent classification phase.
    ///
    /// # Returns
    /// A static string identifier used for LLM-based skill category selection.
    ///
    /// # Examples
    /// ```
    /// let category = SkillCategory::File;
    /// assert_eq!(category.name(), "file_ops");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            SkillCategory::Basic => "basic",
            SkillCategory::File => "file_ops",
            SkillCategory::Math => "math_ops",
            SkillCategory::Network => "network_ops",
            SkillCategory::OperatingSystem => "system_ops",
            SkillCategory::OperatingSystemProcess => "process_ops",
            SkillCategory::OperatingSystemMemory => "memory_ops",
            SkillCategory::Document => "document_ops",
            SkillCategory::SocialPlatform => "social_ops",
            SkillCategory::Database => "database_ops",
            SkillCategory::Text => "text_ops",
            SkillCategory::Devops => "devops_ops",
            SkillCategory::Media => "media_ops",
            SkillCategory::Blockchain => "blockchain_ops",
            SkillCategory::HaveHeadBrowser => "browser_ops",
            SkillCategory::Window => "window_ops",
            SkillCategory::Keyboard => "keyboard_ops",
            SkillCategory::Mouse => "mouse_ops",
            SkillCategory::Audio => "audio_ops",
            SkillCategory::Application => "app_ops",
            SkillCategory::Display => "display_ops",
            SkillCategory::Wifi => "wifi_ops",
            SkillCategory::Bluetooth => "bluetooth_ops",
            SkillCategory::Terminal => "terminal_ops",
            SkillCategory::Email => "email_ops",
            SkillCategory::ScheduledTasks => "schedule_ops",
            SkillCategory::Time => "time_ops",
            SkillCategory::Cryptography => "crypto_ops",
            SkillCategory::SpeechSpeak => "tts_play_on_speaker",
            SkillCategory::OperatingSystemServices => "service_ops",
            SkillCategory::OperatingSystemSecurity => "security_ops",
        }
    }

    /// Get the display name of the category
    pub fn display_name(&self) -> &'static str {
        match self {
            SkillCategory::Basic => "Basic Skills",
            SkillCategory::File => "File System",
            SkillCategory::Math => "Mathematics",
            SkillCategory::Network => "Network",
            SkillCategory::OperatingSystem => "Operating System",
            SkillCategory::OperatingSystemProcess => "Operating System Process Management",
            SkillCategory::OperatingSystemMemory => "Operating System Process Memory Operations",
            SkillCategory::Document => "Document Processing",
            SkillCategory::SocialPlatform => "Social Platform",
            SkillCategory::Database => "Database",
            SkillCategory::Text => "Text Processing",
            SkillCategory::Devops => "DevOps",
            SkillCategory::Media => "Media Processing",
            SkillCategory::Blockchain => "Blockchain",
            SkillCategory::HaveHeadBrowser => "Have Head Browser Control",
            SkillCategory::Window => "Window Control",
            SkillCategory::Keyboard => "Keyboard Control",
            SkillCategory::Mouse => "Mouse Control",
            SkillCategory::Audio => "Audio Control",
            SkillCategory::Application => "Application Control",
            SkillCategory::Display => "Display Control",
            SkillCategory::Wifi => "WiFi Management",
            SkillCategory::Bluetooth => "Bluetooth Management",
            SkillCategory::Terminal => "Terminal Commands",
            SkillCategory::Email => "Email",
            SkillCategory::ScheduledTasks => "Scheduled Tasks",
            SkillCategory::Time => "Time & Date",
            SkillCategory::Cryptography => "Cryptography",
            SkillCategory::SpeechSpeak => "Speech Speak",
            SkillCategory::OperatingSystemServices => "Operating System Services",
            SkillCategory::OperatingSystemSecurity => "Operating System Security",
        }
    }

    /// Get the description of the category
    pub fn description(&self) -> &'static str {
        match self {
            SkillCategory::Basic => "Basic example skills for demonstration and testing",
            SkillCategory::File => {
                "File read/write, directory operations, archive compression/extraction"
            }
            SkillCategory::Math => {
                "Mathematical calculations, statistics, unit conversion, random generation, hashing"
            }
            SkillCategory::Network => {
                "HTTP/HTTPS requests, DNS lookup, Ping/TCP/UDP, FTP, port scanning, HTML parsing, SSH execution, webhook notifications, network diagnostics"
            }
            SkillCategory::OperatingSystem => {
                "System information, power management, environment variables, clipboard operations"
            }
            SkillCategory::OperatingSystemProcess => {
                "Process listing, starting, terminating, and monitoring"
            }
            SkillCategory::OperatingSystemMemory => {
                "Low-level process memory access for debugging, reverse engineering, and security analysis"
            }
            SkillCategory::Document => {
                "Markdown, CSV, XML, Excel, PDF, JSON, YAML, TOML document processing"
            }
            SkillCategory::SocialPlatform => {
                "Send notifications via DingTalk, Feishu, WeChat Work, Telegram"
            }
            SkillCategory::Database => "Database operations for PostgreSQL, MySQL, Redis, SQLite",
            SkillCategory::Text => {
                "Text comparison, sorting, deduplication, filtering, regex operations"
            }
            SkillCategory::Devops => "Kubernetes, Docker, and GitHub operations",
            SkillCategory::Media => "Image processing: resize, convert, crop, rotate, compress",
            SkillCategory::Blockchain => "Bitcoin, EVM, and Solana wallet operations",
            SkillCategory::HaveHeadBrowser => {
                "Have Head Browser automation: navigation, clicking, form filling, screenshot, JS execution"
            }
            SkillCategory::Window => {
                "Window management: minimize, maximize, move, close, pin to top"
            }
            SkillCategory::Keyboard => {
                "Keyboard input simulation: key presses, shortcuts, text typing"
            }
            SkillCategory::Mouse => "Mouse control: movement, clicking, dragging, scrolling",
            SkillCategory::Audio => {
                "Audio control: volume adjustment, device switching, recording, playback"
            }
            SkillCategory::Application => {
                "Application lifecycle: launch, close, install, uninstall"
            }
            SkillCategory::Display => {
                "Display settings: monitor info, resolution, brightness, orientation, refresh rate"
            }
            SkillCategory::Wifi => {
                "WiFi management: scan, connect, hotspot creation, DNS/proxy configuration"
            }
            SkillCategory::Bluetooth => {
                "Bluetooth management: scan, pair, connect, file transfer, BLE operations"
            }
            SkillCategory::Terminal => "Execute system commands and scripts",
            SkillCategory::Email => "Send and manage emails via SMTP/IMAP",
            SkillCategory::ScheduledTasks => "Schedule and manage cron jobs or scheduled tasks",
            SkillCategory::Time => {
                "Get current time, date, timezone conversion, system time management"
            }
            SkillCategory::Cryptography => {
                "Cryptographic operations: hashing (MD5, SHA256, SHA512), Base64 encoding/decoding"
            }
            SkillCategory::SpeechSpeak => "Used to convert text into spoken audio",
            SkillCategory::OperatingSystemServices => {
                "Service management: list, start, stop, restart, enable, disable, and manage system services"
            }
            SkillCategory::OperatingSystemSecurity => {
                "Security operations: weak password detection, security policy assessment, CVE query, threat intelligence, phishing detection"
            }
        }
    }

    /// Get the icon/emoji for the category
    pub fn icon(&self) -> &'static str {
        match self {
            SkillCategory::Basic => "🧪",
            SkillCategory::File => "📁",
            SkillCategory::Math => "🔢",
            SkillCategory::Network => "🌐",
            SkillCategory::OperatingSystem => "💻",
            SkillCategory::OperatingSystemProcess => "⚙️",
            SkillCategory::OperatingSystemMemory => "🧠",
            SkillCategory::Document => "📄",
            SkillCategory::SocialPlatform => "📱",
            SkillCategory::Database => "🗄️",
            SkillCategory::Text => "📝",
            SkillCategory::Devops => "🚀",
            SkillCategory::Media => "🎨",
            SkillCategory::Blockchain => "⛓️",
            SkillCategory::HaveHeadBrowser => "🌍",
            SkillCategory::Window => "🪟",
            SkillCategory::Keyboard => "⌨️",
            SkillCategory::Mouse => "🖱️",
            SkillCategory::Audio => "🎵",
            SkillCategory::Application => "📱",
            SkillCategory::Display => "🖥️",
            SkillCategory::Wifi => "📶",
            SkillCategory::Bluetooth => "📳",
            SkillCategory::Terminal => ">$",
            SkillCategory::Email => "✉️",
            SkillCategory::ScheduledTasks => "⏰",
            SkillCategory::Time => "🕐",
            SkillCategory::Cryptography => "🔐",
            SkillCategory::SpeechSpeak => "🎤",
            SkillCategory::OperatingSystemServices => "🔧",
            SkillCategory::OperatingSystemSecurity => "🛡️",
        }
    }

    /// Get the display priority (lower number = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            SkillCategory::Basic => 100,
            SkillCategory::File => 10,
            SkillCategory::Math => 20,
            SkillCategory::Network => 30,
            SkillCategory::OperatingSystem => 40,
            SkillCategory::OperatingSystemProcess => 41,
            SkillCategory::OperatingSystemMemory => 43,
            SkillCategory::Document => 50,
            SkillCategory::SocialPlatform => 55,
            SkillCategory::Database => 70,
            SkillCategory::Text => 80,
            SkillCategory::Devops => 90,
            SkillCategory::Media => 110,
            SkillCategory::Blockchain => 120,
            SkillCategory::HaveHeadBrowser => 125,
            SkillCategory::Window => 130,
            SkillCategory::Keyboard => 150,
            SkillCategory::Mouse => 151,
            SkillCategory::Audio => 155,
            SkillCategory::Application => 160,
            SkillCategory::Display => 165,
            SkillCategory::Wifi => 170,
            SkillCategory::Bluetooth => 180,
            SkillCategory::Email => 185,
            SkillCategory::ScheduledTasks => 190,
            SkillCategory::Time => 195,
            SkillCategory::Cryptography => 200,
            SkillCategory::Terminal => 250,
            SkillCategory::SpeechSpeak => 255,
            SkillCategory::OperatingSystemServices => 35,
            SkillCategory::OperatingSystemSecurity => 36,
        }
    }

    /// Get complete metadata for the category
    pub fn metadata(&self) -> CategoryMetadata {
        CategoryMetadata {
            name: self.name(),
            display_name: self.display_name(),
            description: self.description(),
            icon: self.icon(),
            priority: self.priority(),
        }
    }

    /// Get all categories with their metadata
    pub fn all_categories() -> Vec<CategoryMetadata> {
        use SkillCategory::*;
        vec![
            Basic.metadata(),
            File.metadata(),
            Math.metadata(),
            Network.metadata(),
            OperatingSystem.metadata(),
            OperatingSystemProcess.metadata(),
            OperatingSystemMemory.metadata(),
            Document.metadata(),
            SocialPlatform.metadata(),
            Database.metadata(),
            Text.metadata(),
            Devops.metadata(),
            Media.metadata(),
            Blockchain.metadata(),
            HaveHeadBrowser.metadata(),
            Window.metadata(),
            Keyboard.metadata(),
            Mouse.metadata(),
            Audio.metadata(),
            Application.metadata(),
            Display.metadata(),
            Wifi.metadata(),
            Bluetooth.metadata(),
            Email.metadata(),
            ScheduledTasks.metadata(),
            Time.metadata(),
            Cryptography.metadata(),
            Terminal.metadata(),
            SpeechSpeak.metadata(),
            OperatingSystemServices.metadata(),
            OperatingSystemSecurity.metadata(),
        ]
    }
}

/// Category metadata structure
#[derive(Debug, Clone, Serialize)]
pub struct CategoryMetadata {
    /// Category name (machine-readable)
    pub name: &'static str,
    /// Human-readable display name
    pub display_name: &'static str,
    /// Category description
    pub description: &'static str,
    /// Icon/emoji representation
    pub icon: &'static str,
    /// Priority order for display (lower = higher priority)
    pub priority: u8,
}
