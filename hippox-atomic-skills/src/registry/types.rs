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
    Net,
    /// System information, power management, environment variables, clipboard
    Os,
    /// Process listing, start, terminate, monitoring
    Process,
    /// Markdown, CSV, XML, Excel, PDF, JSON, YAML, TOML
    Document,
    /// Email, DingTalk, Feishu, WeChat Work, Telegram
    Message,
    /// PostgreSQL, MySQL, Redis, SQLite
    Db,
    /// Text comparison, sorting, deduplication, filtering, regex
    Text,
    /// Kubernetes, Docker, GitHub
    Devops,
    /// Image processing (resize, convert, crop, compress)
    Media,
    /// Bitcoin, EVM, Solana wallet operations
    Blockchain,
    /// Browser navigation, clicking, form filling, screenshot, JS execution
    Browser,
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
}

impl SkillCategory {
    /// Get the string representation of the category
    pub fn as_str(&self) -> &'static str {
        match self {
            SkillCategory::Basic => "basic",
            SkillCategory::File => "file",
            SkillCategory::Math => "math",
            SkillCategory::Net => "net",
            SkillCategory::Os => "os",
            SkillCategory::Process => "process",
            SkillCategory::Document => "document",
            SkillCategory::Message => "message",
            SkillCategory::Db => "db",
            SkillCategory::Text => "text",
            SkillCategory::Devops => "devops",
            SkillCategory::Media => "media",
            SkillCategory::Blockchain => "blockchain",
            SkillCategory::Browser => "browser",
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
        }
    }

    /// Get the display name of the category
    pub fn display_name(&self) -> &'static str {
        match self {
            SkillCategory::Basic => "Basic Skills",
            SkillCategory::File => "File System",
            SkillCategory::Math => "Mathematics",
            SkillCategory::Net => "Network",
            SkillCategory::Os => "Operating System",
            SkillCategory::Process => "Process Management",
            SkillCategory::Document => "Document Processing",
            SkillCategory::Message => "Messaging",
            SkillCategory::Db => "Database",
            SkillCategory::Text => "Text Processing",
            SkillCategory::Devops => "DevOps",
            SkillCategory::Media => "Media Processing",
            SkillCategory::Blockchain => "Blockchain",
            SkillCategory::Browser => "Browser Control",
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
            SkillCategory::Net => {
                "HTTP requests, DNS lookup, Ping, TCP/UDP communication, FTP operations"
            }
            SkillCategory::Os => {
                "System information, power management, environment variables, clipboard operations"
            }
            SkillCategory::Process => "Process listing, starting, terminating, and monitoring",
            SkillCategory::Document => {
                "Markdown, CSV, XML, Excel, PDF, JSON, YAML, TOML document processing"
            }
            SkillCategory::Message => {
                "Send notifications via Email, DingTalk, Feishu, WeChat Work, Telegram"
            }
            SkillCategory::Db => "Database operations for PostgreSQL, MySQL, Redis, SQLite",
            SkillCategory::Text => {
                "Text comparison, sorting, deduplication, filtering, regex operations"
            }
            SkillCategory::Devops => "Kubernetes, Docker, and GitHub operations",
            SkillCategory::Media => "Image processing: resize, convert, crop, rotate, compress",
            SkillCategory::Blockchain => "Bitcoin, EVM, and Solana wallet operations",
            SkillCategory::Browser => {
                "Browser automation: navigation, clicking, form filling, screenshot, JS execution"
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
        }
    }

    /// Get the icon/emoji for the category
    pub fn icon(&self) -> &'static str {
        match self {
            SkillCategory::Basic => "🧪",
            SkillCategory::File => "📁",
            SkillCategory::Math => "🔢",
            SkillCategory::Net => "🌐",
            SkillCategory::Os => "💻",
            SkillCategory::Process => "⚙️",
            SkillCategory::Document => "📄",
            SkillCategory::Message => "💬",
            SkillCategory::Db => "🗄️",
            SkillCategory::Text => "📝",
            SkillCategory::Devops => "🚀",
            SkillCategory::Media => "🎨",
            SkillCategory::Blockchain => "⛓️",
            SkillCategory::Browser => "🌍",
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
        }
    }

    /// Get the display priority (lower number = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            SkillCategory::Basic => 100,
            SkillCategory::File => 10,
            SkillCategory::Math => 20,
            SkillCategory::Net => 30,
            SkillCategory::Os => 40,
            SkillCategory::Process => 41,
            SkillCategory::Document => 50,
            SkillCategory::Message => 60,
            SkillCategory::Db => 70,
            SkillCategory::Text => 80,
            SkillCategory::Devops => 90,
            SkillCategory::Media => 110,
            SkillCategory::Blockchain => 120,
            SkillCategory::Browser => 130,
            SkillCategory::Window => 140,
            SkillCategory::Speech => 150,
            SkillCategory::Keyboard => 160,
            SkillCategory::Mouse => 161,
            SkillCategory::Audio => 170,
            SkillCategory::Application => 180,
            SkillCategory::Display => 190,
            SkillCategory::Wifi => 200,
            SkillCategory::Bluetooth => 210,
            SkillCategory::Terminal => 250,
        }
    }

    /// Get complete metadata for the category
    pub fn metadata(&self) -> CategoryMetadata {
        CategoryMetadata {
            name: self.as_str(),
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
            Net.metadata(),
            Os.metadata(),
            Process.metadata(),
            Document.metadata(),
            Message.metadata(),
            Db.metadata(),
            Text.metadata(),
            Devops.metadata(),
            Media.metadata(),
            Blockchain.metadata(),
            Browser.metadata(),
            Window.metadata(),
            Speech.metadata(),
            Keyboard.metadata(),
            Mouse.metadata(),
            Audio.metadata(),
            Application.metadata(),
            Display.metadata(),
            Wifi.metadata(),
            Bluetooth.metadata(),
            Terminal.metadata(),
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
