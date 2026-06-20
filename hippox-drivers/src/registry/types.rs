//! Driver category enumeration

use serde::{Deserialize, Serialize};

/// Driver category enumeration with metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DriverCategory {
    /// Basic example drivers for demonstration and testing
    Basic,
    /// File read/write, directory operations, compression/extraction
    File,
    /// Mathematical operations, statistics, unit conversion, random numbers, hashing
    Math,
    /// HTTP requests, DNS lookup, Ping, TCP/UDP communication, FTP
    Network,
    /// System information, power management, environment variables, clipboard
    OperatingSystemBasis,
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
    /// CPU information, usage, load, frequency, features, temperature
    OperatingSystemCpu,
    /// GPU information, usage, memory, temperature, fan, power, processes
    OperatingSystemGpu,
    /// Disk information, usage, partitions, I/O, SMART, encryption
    OperatingSystemDisk,
}

impl DriverCategory {
    /// Convert string to DriverCategory enum
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
    /// `Some(DriverCategory)` if the string matches a category name, otherwise `None`
    ///
    /// # Examples
    /// ```
    /// let category = DriverCategory::from_str("file_ops");
    /// assert_eq!(category, Some(DriverCategory::File));
    /// ```
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "basic" => Some(DriverCategory::Basic),
            "file_ops" => Some(DriverCategory::File),
            "math_ops" => Some(DriverCategory::Math),
            "network_ops" => Some(DriverCategory::Network),
            "system_ops" => Some(DriverCategory::OperatingSystemBasis),
            "process_ops" => Some(DriverCategory::OperatingSystemProcess),
            "memory_ops" => Some(DriverCategory::OperatingSystemMemory),
            "cpu_ops" => Some(DriverCategory::OperatingSystemCpu),
            "gpu_ops" => Some(DriverCategory::OperatingSystemGpu),
            "disk_ops" => Some(DriverCategory::OperatingSystemDisk),
            "document_ops" => Some(DriverCategory::Document),
            "social_ops" => Some(DriverCategory::SocialPlatform),
            "database_ops" => Some(DriverCategory::Database),
            "text_ops" => Some(DriverCategory::Text),
            "devops_ops" => Some(DriverCategory::Devops),
            "media_ops" => Some(DriverCategory::Media),
            "blockchain_ops" => Some(DriverCategory::Blockchain),
            "browser_ops" => Some(DriverCategory::HaveHeadBrowser),
            "window_ops" => Some(DriverCategory::Window),
            "keyboard_ops" => Some(DriverCategory::Keyboard),
            "mouse_ops" => Some(DriverCategory::Mouse),
            "audio_ops" => Some(DriverCategory::Audio),
            "app_ops" => Some(DriverCategory::Application),
            "display_ops" => Some(DriverCategory::Display),
            "wifi_ops" => Some(DriverCategory::Wifi),
            "bluetooth_ops" => Some(DriverCategory::Bluetooth),
            "terminal_ops" => Some(DriverCategory::Terminal),
            "email_ops" => Some(DriverCategory::Email),
            "schedule_ops" => Some(DriverCategory::ScheduledTasks),
            "time_ops" => Some(DriverCategory::Time),
            "crypto_ops" => Some(DriverCategory::Cryptography),
            "tts_play_on_speaker" => Some(DriverCategory::SpeechSpeak),
            "service_ops" => Some(DriverCategory::OperatingSystemServices),
            "security_ops" => Some(DriverCategory::OperatingSystemSecurity),
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
    /// let category = DriverCategory::File;
    /// assert_eq!(category.name(), "file_ops");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            DriverCategory::Basic => "basic",
            DriverCategory::File => "file_ops",
            DriverCategory::Math => "math_ops",
            DriverCategory::Network => "network_ops",
            DriverCategory::OperatingSystemBasis => "operating_system_ops",
            DriverCategory::OperatingSystemProcess => "operating_system_process_ops",
            DriverCategory::OperatingSystemMemory => "operating_system_memory_ops",
            DriverCategory::OperatingSystemCpu => "operating_system_cpu_ops",
            DriverCategory::OperatingSystemGpu => "operating_system_gpu_ops",
            DriverCategory::OperatingSystemDisk => "operating_system_disk_ops",
            DriverCategory::Document => "document_ops",
            DriverCategory::SocialPlatform => "social_ops",
            DriverCategory::Database => "database_ops",
            DriverCategory::Text => "text_ops",
            DriverCategory::Devops => "devops_ops",
            DriverCategory::Media => "media_ops",
            DriverCategory::Blockchain => "blockchain_ops",
            DriverCategory::HaveHeadBrowser => "browser_ops",
            DriverCategory::Window => "window_ops",
            DriverCategory::Keyboard => "keyboard_ops",
            DriverCategory::Mouse => "mouse_ops",
            DriverCategory::Audio => "audio_ops",
            DriverCategory::Application => "app_ops",
            DriverCategory::Display => "display_ops",
            DriverCategory::Wifi => "wifi_ops",
            DriverCategory::Bluetooth => "bluetooth_ops",
            DriverCategory::Terminal => "terminal_ops",
            DriverCategory::Email => "email_ops",
            DriverCategory::ScheduledTasks => "schedule_ops",
            DriverCategory::Time => "time_ops",
            DriverCategory::Cryptography => "crypto_ops",
            DriverCategory::SpeechSpeak => "tts_play_on_speaker",
            DriverCategory::OperatingSystemServices => "operating_system_service_ops",
            DriverCategory::OperatingSystemSecurity => "operating_system_security_ops",
        }
    }

    /// Get the display name of the category
    pub fn display_name(&self) -> &'static str {
        match self {
            DriverCategory::Basic => "Basic Skills",
            DriverCategory::File => "File System",
            DriverCategory::Math => "Mathematics",
            DriverCategory::Network => "Network",
            DriverCategory::OperatingSystemBasis => "Operating System Basis Operations",
            DriverCategory::OperatingSystemProcess => "Operating System Process Management",
            DriverCategory::OperatingSystemMemory => "Operating System Process Memory Operations",
            DriverCategory::OperatingSystemCpu => "Operating System CPU Operations",
            DriverCategory::OperatingSystemGpu => "Operating System GPU Operations",
            DriverCategory::OperatingSystemDisk => "Operating System Disk Operations",
            DriverCategory::Document => "Document Processing",
            DriverCategory::SocialPlatform => "Social Platform",
            DriverCategory::Database => "Database",
            DriverCategory::Text => "Text Processing",
            DriverCategory::Devops => "DevOps",
            DriverCategory::Media => "Media Processing",
            DriverCategory::Blockchain => "Blockchain",
            DriverCategory::HaveHeadBrowser => "Have Head Browser Control",
            DriverCategory::Window => "Window Control",
            DriverCategory::Keyboard => "Keyboard Control",
            DriverCategory::Mouse => "Mouse Control",
            DriverCategory::Audio => "Audio Control",
            DriverCategory::Application => "Application Control",
            DriverCategory::Display => "Display Control",
            DriverCategory::Wifi => "WiFi Management",
            DriverCategory::Bluetooth => "Bluetooth Management",
            DriverCategory::Terminal => "Terminal Commands",
            DriverCategory::Email => "Email",
            DriverCategory::ScheduledTasks => "Scheduled Tasks",
            DriverCategory::Time => "Time & Date",
            DriverCategory::Cryptography => "Cryptography",
            DriverCategory::SpeechSpeak => "Speech Speak",
            DriverCategory::OperatingSystemServices => "Operating System Services Operations",
            DriverCategory::OperatingSystemSecurity => "Operating System Security Operations",
        }
    }

    /// Get the description of the category
    pub fn description(&self) -> &'static str {
        match self {
            DriverCategory::Basic => "Basic example skills for demonstration and testing",
            DriverCategory::File => {
                "File read/write, directory operations, archive compression/extraction"
            }
            DriverCategory::Math => {
                "Mathematical calculations, statistics, unit conversion, random generation, hashing"
            }
            DriverCategory::Network => {
                "HTTP/HTTPS requests, DNS lookup, Ping/TCP/UDP, FTP, port scanning, HTML parsing, SSH execution, webhook notifications, network diagnostics"
            }
            DriverCategory::OperatingSystemBasis => {
                "Core system operations: clipboard, system info, reboot/shutdown, sleep/hibernate, screen lock, logout, uptime, hostname, user info, memory info, battery status, desktop notifications"
            }
            DriverCategory::OperatingSystemProcess => {
                "Process listing, starting, terminating, and monitoring"
            }
            DriverCategory::OperatingSystemMemory => {
                "Low-level process memory access for debugging, reverse engineering, and security analysis"
            }
            DriverCategory::OperatingSystemCpu => {
                "CPU information, usage monitoring, load averages, cache info, frequency, features, temperature, and affinity control"
            }
            DriverCategory::OperatingSystemGpu => {
                "GPU information, usage monitoring, memory, temperature, fan speed, power, processes, clock speeds, and video engine utilization"
            }
            DriverCategory::OperatingSystemDisk => {
                "Disk information, usage, partitions, I/O statistics, SMART health, encryption status, and TRIM support"
            }
            DriverCategory::Document => {
                "Markdown, CSV, XML, Excel, PDF, JSON, YAML, TOML document processing"
            }
            DriverCategory::SocialPlatform => {
                "Send notifications via DingTalk, Feishu, WeChat Work, Telegram"
            }
            DriverCategory::Database => "Database operations for PostgreSQL, MySQL, Redis, SQLite",
            DriverCategory::Text => {
                "Text comparison, sorting, deduplication, filtering, regex operations"
            }
            DriverCategory::Devops => "Kubernetes, Docker, and GitHub operations",
            DriverCategory::Media => "Image processing: resize, convert, crop, rotate, compress",
            DriverCategory::Blockchain => "Bitcoin, EVM, and Solana wallet operations",
            DriverCategory::HaveHeadBrowser => {
                "Have Head Browser automation: navigation, clicking, form filling, screenshot, JS execution"
            }
            DriverCategory::Window => {
                "Window management: minimize, maximize, move, close, pin to top"
            }
            DriverCategory::Keyboard => {
                "Keyboard input simulation: key presses, shortcuts, text typing"
            }
            DriverCategory::Mouse => "Mouse control: movement, clicking, dragging, scrolling",
            DriverCategory::Audio => {
                "Audio control: volume adjustment, device switching, recording, playback"
            }
            DriverCategory::Application => {
                "Application lifecycle: launch, close, install, uninstall"
            }
            DriverCategory::Display => {
                "Display settings: monitor info, resolution, brightness, orientation, refresh rate"
            }
            DriverCategory::Wifi => {
                "WiFi management: scan, connect, hotspot creation, DNS/proxy configuration"
            }
            DriverCategory::Bluetooth => {
                "Bluetooth management: scan, pair, connect, file transfer, BLE operations"
            }
            DriverCategory::Terminal => "Execute system commands and scripts",
            DriverCategory::Email => "Send and manage emails via SMTP/IMAP",
            DriverCategory::ScheduledTasks => "Schedule and manage cron jobs or scheduled tasks",
            DriverCategory::Time => {
                "Get current time, date, timezone conversion, system time management"
            }
            DriverCategory::Cryptography => {
                "Cryptographic operations: hashing (MD5, SHA256, SHA512), Base64 encoding/decoding"
            }
            DriverCategory::SpeechSpeak => "Used to convert text into spoken audio",
            DriverCategory::OperatingSystemServices => {
                "Service management: list, start, stop, restart, enable, disable, and manage system services"
            }
            DriverCategory::OperatingSystemSecurity => {
                "Security operations: weak password detection, security policy assessment, CVE query, threat intelligence, phishing detection"
            }
        }
    }

    /// Get the icon/emoji for the category
    pub fn icon(&self) -> &'static str {
        match self {
            DriverCategory::Basic => "🧪",
            DriverCategory::File => "📁",
            DriverCategory::Math => "🔢",
            DriverCategory::Network => "🌐",
            DriverCategory::OperatingSystemBasis => "💻",
            DriverCategory::OperatingSystemProcess => "⚙️",
            DriverCategory::OperatingSystemMemory => "🧠",
            DriverCategory::OperatingSystemCpu => "🔄",
            DriverCategory::OperatingSystemGpu => "🎮",
            DriverCategory::OperatingSystemDisk => "💾",
            DriverCategory::Document => "📄",
            DriverCategory::SocialPlatform => "📱",
            DriverCategory::Database => "🗄️",
            DriverCategory::Text => "📝",
            DriverCategory::Devops => "🚀",
            DriverCategory::Media => "🎨",
            DriverCategory::Blockchain => "⛓️",
            DriverCategory::HaveHeadBrowser => "🌍",
            DriverCategory::Window => "🪟",
            DriverCategory::Keyboard => "⌨️",
            DriverCategory::Mouse => "🖱️",
            DriverCategory::Audio => "🎵",
            DriverCategory::Application => "📱",
            DriverCategory::Display => "🖥️",
            DriverCategory::Wifi => "📶",
            DriverCategory::Bluetooth => "📳",
            DriverCategory::Terminal => ">$",
            DriverCategory::Email => "✉️",
            DriverCategory::ScheduledTasks => "⏰",
            DriverCategory::Time => "🕐",
            DriverCategory::Cryptography => "🔐",
            DriverCategory::SpeechSpeak => "🎤",
            DriverCategory::OperatingSystemServices => "🔧",
            DriverCategory::OperatingSystemSecurity => "🛡️",
        }
    }

    /// Get the display priority (lower number = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            DriverCategory::Basic => 100,
            DriverCategory::File => 10,
            DriverCategory::Math => 20,
            DriverCategory::Network => 30,
            DriverCategory::OperatingSystemBasis => 40,
            DriverCategory::OperatingSystemProcess => 41,
            DriverCategory::OperatingSystemMemory => 43,
            DriverCategory::OperatingSystemCpu => 43,
            DriverCategory::OperatingSystemGpu => 44,
            DriverCategory::OperatingSystemDisk => 45,
            DriverCategory::Document => 50,
            DriverCategory::SocialPlatform => 55,
            DriverCategory::Database => 70,
            DriverCategory::Text => 80,
            DriverCategory::Devops => 90,
            DriverCategory::Media => 110,
            DriverCategory::Blockchain => 120,
            DriverCategory::HaveHeadBrowser => 125,
            DriverCategory::Window => 130,
            DriverCategory::Keyboard => 150,
            DriverCategory::Mouse => 151,
            DriverCategory::Audio => 155,
            DriverCategory::Application => 160,
            DriverCategory::Display => 165,
            DriverCategory::Wifi => 170,
            DriverCategory::Bluetooth => 180,
            DriverCategory::Email => 185,
            DriverCategory::ScheduledTasks => 190,
            DriverCategory::Time => 195,
            DriverCategory::Cryptography => 200,
            DriverCategory::Terminal => 250,
            DriverCategory::SpeechSpeak => 255,
            DriverCategory::OperatingSystemServices => 35,
            DriverCategory::OperatingSystemSecurity => 36,
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
        use DriverCategory::*;
        vec![
            Basic.metadata(),
            File.metadata(),
            Math.metadata(),
            Network.metadata(),
            OperatingSystemBasis.metadata(),
            OperatingSystemProcess.metadata(),
            OperatingSystemMemory.metadata(),
            OperatingSystemCpu.metadata(),
            OperatingSystemGpu.metadata(),
            OperatingSystemDisk.metadata(),
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
