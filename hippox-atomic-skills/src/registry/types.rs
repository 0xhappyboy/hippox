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
    /// Text-to-speech, voice broadcast
    Speech,
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
    SpeechSpeak,
    OperatingSystemServices,
    OperatingSystemSecurity,
}

impl SkillCategory {
    /// Convert string to SkillCategory enum
    ///
    /// # Arguments
    /// * `s` - Category name string (e.g., "basic", "file", "math")
    ///
    /// # Returns
    /// `Some(SkillCategory)` if the string matches a category name, otherwise `None`
    ///
    /// # Examples
    /// ```
    /// let category = SkillCategory::from_str("basic");
    /// assert_eq!(category, Some(SkillCategory::Basic));
    /// ```
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "basic" => Some(SkillCategory::Basic),
            "file" => Some(SkillCategory::File),
            "math" => Some(SkillCategory::Math),
            "network" => Some(SkillCategory::Network),
            "operating_system" => Some(SkillCategory::OperatingSystem),
            "operating_system_process" => Some(SkillCategory::OperatingSystemProcess),
            "operating_system_memory" => Some(SkillCategory::OperatingSystemMemory),
            "document" => Some(SkillCategory::Document),
            "social_platform" => Some(SkillCategory::SocialPlatform),
            "database" => Some(SkillCategory::Database),
            "text" => Some(SkillCategory::Text),
            "devops" => Some(SkillCategory::Devops),
            "media" => Some(SkillCategory::Media),
            "blockchain" => Some(SkillCategory::Blockchain),
            "have_head_browser" => Some(SkillCategory::HaveHeadBrowser),
            "window" => Some(SkillCategory::Window),
            "speech" => Some(SkillCategory::Speech),
            "keyboard" => Some(SkillCategory::Keyboard),
            "mouse" => Some(SkillCategory::Mouse),
            "audio" => Some(SkillCategory::Audio),
            "application" => Some(SkillCategory::Application),
            "display" => Some(SkillCategory::Display),
            "wifi" => Some(SkillCategory::Wifi),
            "bluetooth" => Some(SkillCategory::Bluetooth),
            "terminal" => Some(SkillCategory::Terminal),
            "operating_system_services" => Some(SkillCategory::OperatingSystemServices),
            "operating_system_security" => Some(SkillCategory::OperatingSystemSecurity),
            _ => None,
        }
    }

    /// Get the string representation of the category
    ///
    /// # Returns
    /// The category name as a string
    ///
    /// # Examples
    /// ```
    /// let name = SkillCategory::Basic.name();
    /// assert_eq!(name, "basic");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            SkillCategory::Basic => "basic",
            SkillCategory::File => "file",
            SkillCategory::Math => "math",
            SkillCategory::Network => "network",
            SkillCategory::OperatingSystem => "operating_system",
            SkillCategory::OperatingSystemProcess => "operating_system_process",
            SkillCategory::OperatingSystemMemory => "operating_system_memory",
            SkillCategory::Document => "document",
            SkillCategory::SocialPlatform => "social_platform",
            SkillCategory::Database => "database",
            SkillCategory::Text => "text",
            SkillCategory::Devops => "devops",
            SkillCategory::Media => "media",
            SkillCategory::Blockchain => "blockchain",
            SkillCategory::HaveHeadBrowser => "have_head_browser",
            SkillCategory::Window => "window",
            SkillCategory::Speech => "speech",
            SkillCategory::Keyboard => "keyboard",
            SkillCategory::Mouse => "mouse",
            SkillCategory::Audio => "audio",
            SkillCategory::Application => "application",
            SkillCategory::Display => "display",
            SkillCategory::Wifi => "wifi",
            SkillCategory::Bluetooth => "bluetooth",
            SkillCategory::Terminal => "terminal",
            SkillCategory::Email => "email",
            SkillCategory::ScheduledTasks => "scheduled_tasks",
            SkillCategory::Time => "time",
            SkillCategory::Cryptography => "cryptography",
            SkillCategory::SpeechSpeak => "speech_speak",
            SkillCategory::OperatingSystemServices => "operating_system_services",
            SkillCategory::OperatingSystemSecurity => "operating_system_security",
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
            SkillCategory::Speech => "Speech Synthesis",
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
                "HTTP requests, DNS lookup, Ping, TCP/UDP communication, FTP operations"
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
            SkillCategory::Speech => "Text-to-speech synthesis and voice broadcast",
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
            SkillCategory::Speech => "🔊",
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
            SkillCategory::Speech => 140,
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
            Speech.metadata(),
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
