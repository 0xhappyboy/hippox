/// # Skill Registry Module
///
/// This module provides a central registry for managing all available skills in the system.
/// It maintains a thread-safe, global mapping from skill names to their implementations.
/// Skills can be registered, retrieved, and listed, and the registry can generate
/// AI-friendly metadata for LLM integration.
use crate::executors::Skill;
#[cfg(any(feature = "blockchain", feature = "all"))]
use crate::executors::skills::BitcoinWalletSkill;
#[cfg(any(feature = "blockchain", feature = "all"))]
use crate::executors::skills::EvmWalletSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserBackSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserClickSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserCloseSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserExecuteJsSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserFindElementSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserForwardSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserGetTextSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserGetTitleSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserGetUrlSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserNavigateSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserRefreshSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserScreenshotSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserScrollSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserTabCloseSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserTabNewSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserTabSwitchSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserTypeSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::HaveHeadBrowserWaitSkill;
#[cfg(any(feature = "blockchain", feature = "all"))]
use crate::executors::skills::SolanaWalletSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::have_head_browser::HaveHeadBrowserElementExistsSkill;
#[cfg(any(feature = "have_head_browser", feature = "all"))]
use crate::executors::skills::have_head_browser::*;
#[cfg(any(feature = "operating_system", feature = "all"))]
use crate::executors::skills::operating_system::*;
#[cfg(any(feature = "window_control", feature = "all"))]
use crate::executors::skills::window_control::*;
use crate::executors::types::SkillMetadata;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

/// Global, lazily-initialized, thread-safe registry of all available skills.
/// Uses a read-write lock to allow concurrent reads and exclusive writes.
/// The registry is stored as a HashMap mapping skill names (String) to
/// atomic reference-counted pointers to trait objects implementing the Skill trait.
static SKILL_REGISTRY: Lazy<RwLock<HashMap<String, Arc<dyn Skill>>>> = Lazy::new(|| {
    let mut registry: HashMap<String, Arc<dyn Skill>> = HashMap::new();
    // ==================== Basic Skills ====================
    #[cfg(any(feature = "helloworld", feature = "all"))]
    registry.insert(
        "helloworld".to_string(),
        Arc::new(super::skills::HelloWorldSkill) as Arc<dyn Skill>,
    );
    // ==================== File System Skills ====================
    #[cfg(any(feature = "file", feature = "all"))]
    registry.insert(
        "file_read".to_string(),
        Arc::new(super::skills::file::ReadFileSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "file", feature = "all"))]
    registry.insert(
        "file_write".to_string(),
        Arc::new(super::skills::file::WriteFileSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "file", feature = "all"))]
    registry.insert(
        "file_delete".to_string(),
        Arc::new(super::skills::file::DeleteFileSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "file", feature = "all"))]
    registry.insert(
        "file_list".to_string(),
        Arc::new(super::skills::file::ListDirectorySkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "file", feature = "all"))]
    registry.insert(
        "file_copy".to_string(),
        Arc::new(super::skills::file::CopyFileSkill) as Arc<dyn Skill>,
    );
    // ==================== Archive Skills ====================
    #[cfg(any(feature = "file", feature = "all"))]
    registry.insert(
        "archive_zip_create".to_string(),
        Arc::new(super::skills::file::ArchiveZipCreateSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "file", feature = "all"))]
    registry.insert(
        "archive_zip_extract".to_string(),
        Arc::new(super::skills::file::ArchiveZipExtractSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "file", feature = "all"))]
    registry.insert(
        "archive_tar_create".to_string(),
        Arc::new(super::skills::file::ArchiveTarCreateSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "file", feature = "all"))]
    registry.insert(
        "archive_tar_extract".to_string(),
        Arc::new(super::skills::file::ArchiveTarExtractSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "file", feature = "all"))]
    registry.insert(
        "archive_compress".to_string(),
        Arc::new(super::skills::file::ArchiveCompressSkill) as Arc<dyn Skill>,
    );
    // ==================== Mathematics Skills ====================
    #[cfg(any(feature = "math", feature = "all"))]
    registry.insert(
        "math_calculator".to_string(),
        Arc::new(super::skills::CalculatorSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "math", feature = "all"))]
    registry.insert(
        "math_power".to_string(),
        Arc::new(super::skills::PowerSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "math", feature = "all"))]
    registry.insert(
        "math_statistics".to_string(),
        Arc::new(super::skills::StatisticsSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "math", feature = "all"))]
    registry.insert(
        "math_unit_converter".to_string(),
        Arc::new(super::skills::UnitConverterSkill) as Arc<dyn Skill>,
    );
    // ==================== Crypto & Random Skills ====================
    #[cfg(any(feature = "math", feature = "all"))]
    registry.insert(
        "hash_md5".to_string(),
        Arc::new(super::skills::math::HashMd5Skill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "math", feature = "all"))]
    registry.insert(
        "hash_sha256".to_string(),
        Arc::new(super::skills::math::HashSha256Skill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "math", feature = "all"))]
    registry.insert(
        "hash_sha512".to_string(),
        Arc::new(super::skills::math::HashSha512Skill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "math", feature = "all"))]
    registry.insert(
        "hash_file".to_string(),
        Arc::new(super::skills::math::HashFileSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "math", feature = "all"))]
    registry.insert(
        "base64_encode".to_string(),
        Arc::new(super::skills::math::Base64EncodeSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "math", feature = "all"))]
    registry.insert(
        "base64_decode".to_string(),
        Arc::new(super::skills::math::Base64DecodeSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "math", feature = "all"))]
    registry.insert(
        "random_number".to_string(),
        Arc::new(super::skills::math::RandomNumberSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "math", feature = "all"))]
    registry.insert(
        "random_string".to_string(),
        Arc::new(super::skills::math::RandomStringSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "math", feature = "all"))]
    registry.insert(
        "random_uuid".to_string(),
        Arc::new(super::skills::math::RandomUuidSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "math", feature = "all"))]
    registry.insert(
        "random_password".to_string(),
        Arc::new(super::skills::math::RandomPasswordSkill) as Arc<dyn Skill>,
    );
    // ==================== Time Skills ====================
    #[cfg(any(feature = "time", feature = "all"))]
    registry.insert(
        "time_datetime".to_string(),
        Arc::new(super::skills::DateTimeSkill) as Arc<dyn Skill>,
    );
    // ==================== Network Skills ====================
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "net_httprequest".to_string(),
        Arc::new(super::skills::HttpRequestSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "read_url".to_string(),
        Arc::new(super::skills::ReadUrlSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "ping".to_string(),
        Arc::new(super::skills::net::ping::PingSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "tcp_ping".to_string(),
        Arc::new(super::skills::net::ping::TcpPingSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "batch_ping".to_string(),
        Arc::new(super::skills::net::ping::BatchPingSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "dns_lookup".to_string(),
        Arc::new(super::skills::net::dns::DnsLookupSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "reverse_dns".to_string(),
        Arc::new(super::skills::net::dns::ReverseDnsSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "dns_batch_lookup".to_string(),
        Arc::new(super::skills::net::dns::DnsBatchLookupSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "dns_test".to_string(),
        Arc::new(super::skills::net::dns::DnsTestSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "ip_info".to_string(),
        Arc::new(super::skills::net::ip::IpInfoSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "ip_validate".to_string(),
        Arc::new(super::skills::net::ip::IpValidateSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "ip_range".to_string(),
        Arc::new(super::skills::net::ip::IpRangeSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "local_ip".to_string(),
        Arc::new(super::skills::net::ip::LocalIpSkill) as Arc<dyn Skill>,
    );
    // ==================== TCP/UDP/FTP Skills ====================
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "tcp_send".to_string(),
        Arc::new(super::skills::tcp::TcpSendSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "tcp_receive".to_string(),
        Arc::new(super::skills::tcp::TcpReceiveSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "udp_send".to_string(),
        Arc::new(super::skills::udp::UdpSendSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "udp_receive".to_string(),
        Arc::new(super::skills::udp::UdpReceiveSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "udp_broadcast".to_string(),
        Arc::new(super::skills::udp::UdpBroadcastSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "ftp_upload".to_string(),
        Arc::new(super::skills::ftp::FtpUploadSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "ftp_download".to_string(),
        Arc::new(super::skills::ftp::FtpDownloadSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "ftp_list".to_string(),
        Arc::new(super::skills::ftp::FtpListSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "net", feature = "all"))]
    registry.insert(
        "ftp_delete".to_string(),
        Arc::new(super::skills::ftp::FtpDeleteSkill) as Arc<dyn Skill>,
    );
    // ==================== OS Management Skills ====================
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_reboot".to_string(),
        Arc::new(super::skills::operating_system::OsRebootSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_shutdown".to_string(),
        Arc::new(super::skills::operating_system::OsShutdownSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_sleep".to_string(),
        Arc::new(super::skills::operating_system::OsSleepSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_lock".to_string(),
        Arc::new(super::skills::operating_system::OsLockSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_logout".to_string(),
        Arc::new(super::skills::operating_system::OsLogoutSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_hibernate".to_string(),
        Arc::new(super::skills::operating_system::OsHibernateSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_get_uptime".to_string(),
        Arc::new(super::skills::operating_system::OsGetUptimeSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_get_load_average".to_string(),
        Arc::new(super::skills::operating_system::OsGetLoadAverageSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_get_hostname".to_string(),
        Arc::new(super::skills::operating_system::OsGetHostnameSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_get_time".to_string(),
        Arc::new(super::skills::operating_system::OsGetTimeSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_set_time".to_string(),
        Arc::new(super::skills::operating_system::OsSetTimeSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_get_user".to_string(),
        Arc::new(super::skills::operating_system::OsGetUserSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_disk_usage".to_string(),
        Arc::new(super::skills::operating_system::OsDiskUsageSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_memory_info".to_string(),
        Arc::new(super::skills::operating_system::OsMemoryInfoSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_cpu_info".to_string(),
        Arc::new(super::skills::operating_system::OsCpuInfoSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_network_info".to_string(),
        Arc::new(super::skills::operating_system::OsNetworkInfoSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_battery_info".to_string(),
        Arc::new(super::skills::operating_system::OsBatteryInfoSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "os_notification".to_string(),
        Arc::new(super::skills::operating_system::OsNotificationSkill) as Arc<dyn Skill>,
    );
    // ==================== Process Skills ====================
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "process_list".to_string(),
        Arc::new(super::skills::operating_system::ProcessListSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "process_kill".to_string(),
        Arc::new(super::skills::operating_system::ProcessKillSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "process_kill_by_name".to_string(),
        Arc::new(super::skills::operating_system::ProcessKillByNameSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "process_is_running".to_string(),
        Arc::new(super::skills::operating_system::ProcessIsRunningSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "process_get_pid".to_string(),
        Arc::new(super::skills::operating_system::ProcessGetPidSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "process_info".to_string(),
        Arc::new(super::skills::operating_system::ProcessInfoSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "process_basic_info".to_string(),
        Arc::new(super::skills::operating_system::ProcessBasicInfoSkill) as Arc<dyn Skill>,
    );
    // ==================== System Skills ====================
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "system_systeminfo".to_string(),
        Arc::new(SystemInfoSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "port_scan".to_string(),
        Arc::new(super::skills::PortScanSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "port_lookup".to_string(),
        Arc::new(super::skills::PortLookupSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "port_test".to_string(),
        Arc::new(super::skills::PortTestSkill) as Arc<dyn Skill>,
    );
    // ==================== Clipboard Skills ====================
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "clipboard_get".to_string(),
        Arc::new(super::skills::operating_system::clipboard::ClipboardGetSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "clipboard_set".to_string(),
        Arc::new(super::skills::operating_system::clipboard::ClipboardSetSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "operating_system", feature = "all"))]
    registry.insert(
        "clipboard_clear".to_string(),
        Arc::new(super::skills::operating_system::clipboard::ClipboardClearSkill) as Arc<dyn Skill>,
    );
    // ==================== Terminal Commands ====================
    #[cfg(any(feature = "terminal_commands", feature = "all"))]
    registry.insert(
        "exec_command".to_string(),
        Arc::new(crate::executors::skills::ExecCommandSkill) as Arc<dyn Skill>,
    );
    // ==================== Document Skills ====================
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "markdown_read".to_string(),
        Arc::new(super::skills::document::MarkdownReadSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "markdown_write".to_string(),
        Arc::new(super::skills::document::MarkdownWriteSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "csv_read".to_string(),
        Arc::new(super::skills::document::CsvReadSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "csv_write".to_string(),
        Arc::new(super::skills::document::CsvWriteSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "xml_parse".to_string(),
        Arc::new(super::skills::document::XmlParseSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "xml_to_json".to_string(),
        Arc::new(super::skills::document::XmlToJsonSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "excel_read".to_string(),
        Arc::new(super::skills::document::ExcelReadSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "excel_write".to_string(),
        Arc::new(super::skills::document::ExcelWriteSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "pdf_read".to_string(),
        Arc::new(super::skills::document::PdfReadSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "pdf_merge".to_string(),
        Arc::new(super::skills::document::PdfMergeSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "pdf_info".to_string(),
        Arc::new(super::skills::document::PdfInfoSkill) as Arc<dyn Skill>,
    );
    // ==================== Messaging Skills ====================
    #[cfg(any(feature = "message", feature = "all"))]
    registry.insert(
        "send_email".to_string(),
        Arc::new(super::skills::message::SendEmailSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "message", feature = "all"))]
    registry.insert(
        "send_telegram".to_string(),
        Arc::new(super::skills::message::SendTelegramSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "message", feature = "all"))]
    registry.insert(
        "send_dingding".to_string(),
        Arc::new(super::skills::message::SendDingDingSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "message", feature = "all"))]
    registry.insert(
        "send_feishu".to_string(),
        Arc::new(super::skills::message::SendFeishuSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "message", feature = "all"))]
    registry.insert(
        "send_wecom".to_string(),
        Arc::new(super::skills::message::SendWecomSkill) as Arc<dyn Skill>,
    );
    // ==================== Database Skills ====================
    // PostgreSQL
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "postgres_query".to_string(),
        Arc::new(super::skills::postgresql::PostgresQuerySkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "postgres_execute".to_string(),
        Arc::new(super::skills::postgresql::PostgresExecuteSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "postgres_list_tables".to_string(),
        Arc::new(super::skills::postgresql::PostgresListTablesSkill) as Arc<dyn Skill>,
    );
    // MySQL
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "mysql_query".to_string(),
        Arc::new(super::skills::mysql::MysqlQuerySkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "mysql_execute".to_string(),
        Arc::new(super::skills::mysql::MysqlExecuteSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "mysql_list_tables".to_string(),
        Arc::new(super::skills::mysql::MysqlListTablesSkill) as Arc<dyn Skill>,
    );
    // Redis
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "redis_set".to_string(),
        Arc::new(super::skills::redis::RedisSetSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "redis_get".to_string(),
        Arc::new(super::skills::redis::RedisGetSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "redis_del".to_string(),
        Arc::new(super::skills::redis::RedisDelSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "redis_keys".to_string(),
        Arc::new(super::skills::redis::RedisKeysSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "redis_hset".to_string(),
        Arc::new(super::skills::redis::RedisHSetSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "redis_hget".to_string(),
        Arc::new(super::skills::redis::RedisHGetSkill) as Arc<dyn Skill>,
    );
    // SQLite
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "sqlite_query".to_string(),
        Arc::new(super::skills::sqlite::SqliteQuerySkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "sqlite_execute".to_string(),
        Arc::new(super::skills::sqlite::SqliteExecuteSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "db", feature = "all"))]
    registry.insert(
        "sqlite_list_tables".to_string(),
        Arc::new(super::skills::sqlite::SqliteListTablesSkill) as Arc<dyn Skill>,
    );
    // ==================== Text Processing Skills ====================
    #[cfg(any(feature = "text", feature = "all"))]
    registry.insert(
        "text_diff".to_string(),
        Arc::new(super::skills::text::TextDiffSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "text", feature = "all"))]
    registry.insert(
        "text_sort".to_string(),
        Arc::new(super::skills::text::TextSortSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "text", feature = "all"))]
    registry.insert(
        "text_deduplicate".to_string(),
        Arc::new(super::skills::text::TextDeduplicateSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "text", feature = "all"))]
    registry.insert(
        "text_filter".to_string(),
        Arc::new(super::skills::text::TextFilterSkill) as Arc<dyn Skill>,
    );

    // ==================== Regex Skills ====================
    #[cfg(any(feature = "text", feature = "all"))]
    registry.insert(
        "regex_match".to_string(),
        Arc::new(super::skills::regex::RegexMatchSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "text", feature = "all"))]
    registry.insert(
        "regex_find".to_string(),
        Arc::new(super::skills::regex::RegexFindSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "text", feature = "all"))]
    registry.insert(
        "regex_replace".to_string(),
        Arc::new(super::skills::regex::RegexReplaceSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "text", feature = "all"))]
    registry.insert(
        "regex_extract".to_string(),
        Arc::new(super::skills::regex::RegexExtractSkill) as Arc<dyn Skill>,
    );
    // ==================== k8s Skills ====================
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "k8s_get_pods".to_string(),
        Arc::new(super::skills::k8s::K8sGetPodsSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "k8s_describe_pod".to_string(),
        Arc::new(super::skills::k8s::K8sDescribePodSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "k8s_get_logs".to_string(),
        Arc::new(super::skills::k8s::K8sGetLogsSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "k8s_exec".to_string(),
        Arc::new(super::skills::k8s::K8sExecSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "k8s_get_deployments".to_string(),
        Arc::new(super::skills::k8s::K8sGetDeploymentsSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "k8s_scale_deployment".to_string(),
        Arc::new(super::skills::k8s::K8sScaleDeploymentSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "k8s_restart_deployment".to_string(),
        Arc::new(super::skills::k8s::K8sRestartDeploymentSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "k8s_get_nodes".to_string(),
        Arc::new(super::skills::k8s::K8sGetNodesSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "k8s_get_namespaces".to_string(),
        Arc::new(super::skills::k8s::K8sGetNamespacesSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "k8s_apply_yaml".to_string(),
        Arc::new(super::skills::k8s::K8sApplyYamlSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "k8s_delete_resource".to_string(),
        Arc::new(super::skills::k8s::K8sDeleteResourceSkill) as Arc<dyn Skill>,
    );
    // ==================== Docker Skills ====================
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "docker_ps".to_string(),
        Arc::new(super::skills::docker::DockerPsSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "docker_start_stop".to_string(),
        Arc::new(super::skills::docker::DockerStartStopSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "docker_logs".to_string(),
        Arc::new(super::skills::docker::DockerLogsSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "docker_inspect".to_string(),
        Arc::new(super::skills::docker::DockerInspectSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "docker_exec".to_string(),
        Arc::new(super::skills::docker::DockerExecSkill) as Arc<dyn Skill>,
    );
    // ==================== GitHub Skills ====================
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "github_get_repo".to_string(),
        Arc::new(super::skills::github::GithubGetRepo) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "github_create_issue".to_string(),
        Arc::new(super::skills::github::GithubCreateIssue) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "github_list_issues".to_string(),
        Arc::new(super::skills::github::GithubListIssues) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "github_star_repo".to_string(),
        Arc::new(super::skills::github::GithubStarRepo) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "github_search_repos".to_string(),
        Arc::new(super::skills::github::GithubSearchRepos) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "github_get_user".to_string(),
        Arc::new(super::skills::github::GithubGetUser) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "devops", feature = "all"))]
    registry.insert(
        "github_list_prs".to_string(),
        Arc::new(super::skills::github::GithubListPRs) as Arc<dyn Skill>,
    );
    // ==================== Scheduler Skills ====================
    #[cfg(any(feature = "task", feature = "all"))]
    registry.insert(
        "schedule_task".to_string(),
        Arc::new(super::skills::task::ScheduleTaskSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "task", feature = "all"))]
    registry.insert(
        "unschedule_task".to_string(),
        Arc::new(super::skills::task::UnscheduleTaskSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "task", feature = "all"))]
    registry.insert(
        "list_scheduled_tasks".to_string(),
        Arc::new(super::skills::task::ListScheduledTasksSkill) as Arc<dyn Skill>,
    );
    // ==================== Image Processing Skills ====================
    #[cfg(any(feature = "media", feature = "all"))]
    registry.insert(
        "image_resize".to_string(),
        Arc::new(super::skills::image::ImageResizeSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "media", feature = "all"))]
    registry.insert(
        "image_convert".to_string(),
        Arc::new(super::skills::image::ImageConvertSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "media", feature = "all"))]
    registry.insert(
        "image_info".to_string(),
        Arc::new(super::skills::image::ImageInfoSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "media", feature = "all"))]
    registry.insert(
        "image_rotate".to_string(),
        Arc::new(super::skills::image::ImageRotateSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "media", feature = "all"))]
    registry.insert(
        "image_crop".to_string(),
        Arc::new(super::skills::image::ImageCropSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "media", feature = "all"))]
    registry.insert(
        "image_compress".to_string(),
        Arc::new(super::skills::image::ImageCompressSkill) as Arc<dyn Skill>,
    );
    // ==================== Blockchain Skills ====================
    #[cfg(any(feature = "blockchain", feature = "all"))]
    registry.insert(
        "blockchain_bitcoin_wallet".to_string(),
        Arc::new(BitcoinWalletSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "blockchain", feature = "all"))]
    registry.insert(
        "blockchain_evm_wallet".to_string(),
        Arc::new(EvmWalletSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "blockchain", feature = "all"))]
    registry.insert(
        "blockchain_solana_wallet".to_string(),
        Arc::new(SolanaWalletSkill) as Arc<dyn Skill>,
    );
    // JSON Skills
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "json_read".to_string(),
        Arc::new(super::skills::document::JsonReadSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "json_write".to_string(),
        Arc::new(super::skills::document::JsonWriteSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "json_validate".to_string(),
        Arc::new(super::skills::document::JsonValidateSkill) as Arc<dyn Skill>,
    );

    // YAML Skills
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "yaml_read".to_string(),
        Arc::new(super::skills::document::YamlReadSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "yaml_write".to_string(),
        Arc::new(super::skills::document::YamlWriteSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "yaml_validate".to_string(),
        Arc::new(super::skills::document::YamlValidateSkill) as Arc<dyn Skill>,
    );

    // TOML Skills
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "toml_read".to_string(),
        Arc::new(super::skills::document::TomlReadSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "toml_write".to_string(),
        Arc::new(super::skills::document::TomlWriteSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "toml_validate".to_string(),
        Arc::new(super::skills::document::TomlValidateSkill) as Arc<dyn Skill>,
    );

    // Text/Plain Skills
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "text_read".to_string(),
        Arc::new(super::skills::document::TextReadSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "text_write".to_string(),
        Arc::new(super::skills::document::TextWriteSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "text_search".to_string(),
        Arc::new(super::skills::document::TextSearchSkill) as Arc<dyn Skill>,
    );

    // HTML Skills
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "html_read".to_string(),
        Arc::new(super::skills::document::HtmlReadSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "html_write".to_string(),
        Arc::new(super::skills::document::HtmlWriteSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "html_validate".to_string(),
        Arc::new(super::skills::document::HtmlValidateSkill) as Arc<dyn Skill>,
    );

    // PPTX Skills
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "pptx_read".to_string(),
        Arc::new(super::skills::document::PptxReadSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "pptx_info".to_string(),
        Arc::new(super::skills::document::PptxInfoSkill) as Arc<dyn Skill>,
    );

    // DOCX Skills
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "docx_read".to_string(),
        Arc::new(super::skills::document::DocxReadSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "docx_info".to_string(),
        Arc::new(super::skills::document::DocxInfoSkill) as Arc<dyn Skill>,
    );

    // OpenDocument Skills
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "ods_read".to_string(),
        Arc::new(super::skills::document::OdsReadSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "document", feature = "all"))]
    registry.insert(
        "odt_read".to_string(),
        Arc::new(super::skills::document::OdtReadSkill) as Arc<dyn Skill>,
    );
    // ==================== Headful Browser Skills ====================
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_navigate".to_string(),
        Arc::new(HaveHeadBrowserNavigateSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_click".to_string(),
        Arc::new(HaveHeadBrowserClickSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_type".to_string(),
        Arc::new(HaveHeadBrowserTypeSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_get_text".to_string(),
        Arc::new(HaveHeadBrowserGetTextSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_screenshot".to_string(),
        Arc::new(HaveHeadBrowserScreenshotSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_wait".to_string(),
        Arc::new(HaveHeadBrowserWaitSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_execute_js".to_string(),
        Arc::new(HaveHeadBrowserExecuteJsSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_get_url".to_string(),
        Arc::new(HaveHeadBrowserGetUrlSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_get_title".to_string(),
        Arc::new(HaveHeadBrowserGetTitleSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_back".to_string(),
        Arc::new(HaveHeadBrowserBackSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_forward".to_string(),
        Arc::new(HaveHeadBrowserForwardSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_refresh".to_string(),
        Arc::new(HaveHeadBrowserRefreshSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_tab_new".to_string(),
        Arc::new(HaveHeadBrowserTabNewSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_tab_close".to_string(),
        Arc::new(HaveHeadBrowserTabCloseSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_tab_switch".to_string(),
        Arc::new(HaveHeadBrowserTabSwitchSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_find_element".to_string(),
        Arc::new(HaveHeadBrowserFindElementSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_element_exists".to_string(),
        Arc::new(HaveHeadBrowserElementExistsSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_scroll".to_string(),
        Arc::new(HaveHeadBrowserScrollSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    registry.insert(
        "have_head_browser_close".to_string(),
        Arc::new(HaveHeadBrowserCloseSkill) as Arc<dyn Skill>,
    );

    // ==================== Window Control Skills ====================
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_minimize".to_string(),
        Arc::new(super::skills::window_control::WindowControlMinimizeSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_maximize".to_string(),
        Arc::new(super::skills::window_control::WindowControlMaximizeSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_restore".to_string(),
        Arc::new(super::skills::window_control::WindowControlRestoreSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_resize".to_string(),
        Arc::new(super::skills::window_control::WindowControlResizeSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_move".to_string(),
        Arc::new(super::skills::window_control::WindowControlMoveSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_close".to_string(),
        Arc::new(super::skills::window_control::WindowControlCloseSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_kill".to_string(),
        Arc::new(super::skills::window_control::WindowControlKillSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_bring_to_top".to_string(),
        Arc::new(super::skills::window_control::WindowControlBringToTopSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_send_to_back".to_string(),
        Arc::new(super::skills::window_control::WindowControlSendToBackSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_set_always_on_top".to_string(),
        Arc::new(super::skills::window_control::WindowControlSetAlwaysOnTopSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_get_title".to_string(),
        Arc::new(super::skills::window_control::WindowControlGetTitleSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_get_process".to_string(),
        Arc::new(super::skills::window_control::WindowControlGetProcessSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_screenshot".to_string(),
        Arc::new(super::skills::window_control::WindowControlScreenshotSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_ocr_region".to_string(),
        Arc::new(super::skills::window_control::WindowControlOcrRegionSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_list".to_string(),
        Arc::new(super::skills::window_control::WindowControlListSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_find".to_string(),
        Arc::new(super::skills::window_control::WindowControlFindSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_activate".to_string(),
        Arc::new(super::skills::window_control::WindowControlActivateSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_get_position".to_string(),
        Arc::new(super::skills::window_control::WindowControlGetPositionSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_get_focus".to_string(),
        Arc::new(super::skills::window_control::WindowControlGetFocusSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_get_selected".to_string(),
        Arc::new(super::skills::window_control::WindowControlGetSelectedSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_send_keys".to_string(),
        Arc::new(super::skills::window_control::WindowControlSendKeysSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_send_shortcut".to_string(),
        Arc::new(super::skills::window_control::WindowControlSendShortcutSkill) as Arc<dyn Skill>,
    );
    #[cfg(any(feature = "window_control", feature = "all"))]
    registry.insert(
        "window_control_wait_for_focus".to_string(),
        Arc::new(super::skills::window_control::WindowControlWaitForFocusSkill) as Arc<dyn Skill>,
    );
    RwLock::new(registry)
});

/// Generates a list of metadata for all registered skills.
/// This is useful for AI systems that need to understand available capabilities.
///
/// # Returns
/// A vector of `SkillMetadata` containing information about each registered skill,
/// including its name, category, description, and parameter schema.
///
/// # Example
/// ```
/// let metadata = generate_ai_registry();
/// for skill in metadata {
///     println!("Skill: {} - {}", skill.name, skill.description);
/// }
/// ```
pub fn generate_ai_registry() -> Vec<SkillMetadata> {
    let registry = get_registry();
    registry
        .values()
        .map(|skill| skill.get_metadata())
        .collect()
}

/// Generates a comprehensive JSON representation of the skill registry.
/// This includes version information, total skill count, and all skill metadata,
/// along with instructions for AI systems on how to invoke skills.
///
/// # Returns
/// A `serde_json::Value` containing the complete registry information in JSON format.
///
/// # JSON Structure
/// ```json
/// {
///   "version": "1.0",
///   "total_skills": 50,
///   "skills": [...],
///   "instruction": "You can call a skill by returning a JSON object..."
/// }
/// ```
pub fn generate_skill_registry_table_json() -> Value {
    let metadata = generate_ai_registry();
    serde_json::json!({
        "version": "1.0",
        "total_skills": metadata.len(),
        "skills": metadata,
        "instruction": r#"You can call a skill by returning a JSON object with 'action' and 'parameters' fields. Example: {"action": "calculator", "parameters": {"expression": "2+3"}}"#
    })
}

/// Generates a pretty-printed JSON string representation of the skill registry.
/// This is convenient for logging, debugging, or sending to LLM APIs.
///
/// # Returns
/// A formatted JSON string containing the complete registry information.
/// If serialization fails, the function will panic (which is expected in normal operation).
pub fn generate_skill_registry_table_json_str() -> String {
    serde_json::to_string_pretty(&generate_skill_registry_table_json()).unwrap()
}

/// Acquires a read lock on the global skill registry and returns a guard.
/// This allows concurrent read access to the registry.
///
/// # Returns
/// A read guard that provides access to the underlying HashMap.
///
/// # Panics
/// Will panic if the lock is poisoned (which should not happen under normal operation).
pub fn get_registry() -> std::sync::RwLockReadGuard<'static, HashMap<String, Arc<dyn Skill>>> {
    SKILL_REGISTRY.read().unwrap()
}

/// Acquires a write lock on the global skill registry and returns a guard.
/// This allows exclusive write access to the registry.
///
/// # Returns
/// A write guard that provides mutable access to the underlying HashMap.
///
/// # Panics
/// Will panic if the lock is poisoned (which should not happen under normal operation).
pub fn get_registry_mut() -> std::sync::RwLockWriteGuard<'static, HashMap<String, Arc<dyn Skill>>> {
    SKILL_REGISTRY.write().unwrap()
}

/// Retrieves a skill by name from the registry.
///
/// # Arguments
/// * `name` - The name of the skill to retrieve (e.g., "file_read", "math_calculator")
///
/// # Returns
/// An `Option` containing an `Arc<dyn Skill>` if the skill exists, otherwise `None`.
///
/// # Example
/// ```
/// if let Some(skill) = get_skill("file_read") {
///     println!("Skill found!");
/// }
/// ```
pub fn get_skill(name: &str) -> Option<Arc<dyn Skill>> {
    get_registry().get(name).cloned()
}

/// Dynamically registers a new skill into the global registry.
/// This allows runtime addition of skills after the initial registry initialization.
///
/// # Arguments
/// * `name` - The unique name to associate with the skill
/// * `skill` - An atomic reference-counted pointer to the skill implementation
///
/// # Example
/// ```
/// let my_skill = Arc::new(MyCustomSkill);
/// register_skill("my_custom_skill".to_string(), my_skill);
/// ```
pub fn register_skill(name: String, skill: Arc<dyn Skill>) {
    get_registry_mut().insert(name, skill);
}

/// Checks whether a skill with the given name exists in the registry.
///
/// # Arguments
/// * `name` - The name of the skill to check
///
/// # Returns
/// `true` if the skill exists, `false` otherwise.
pub fn has_skill(name: &str) -> bool {
    get_registry().contains_key(name)
}

/// Returns a list of all registered skill names.
///
/// # Returns
/// A vector containing the names of all skills in the registry.
pub fn list_skills() -> Vec<String> {
    get_registry().keys().cloned().collect()
}

#[cfg(test)]
mod registry_test {
    use super::*;

    #[test]
    fn print_registry() {
        println!("{:?}", generate_skill_registry_table_json_str());
    }

    /// Test that the AI registry generation returns metadata for all skills
    #[test]
    fn test_generate_ai_registry() {
        let metadata = generate_ai_registry();
        println!("Total skills: {}", metadata.len());
        assert!(
            metadata.len() > 50,
            "Expected at least 50 skills, got {}",
            metadata.len()
        );
        for skill in &metadata {
            println!(
                "  - {} ({}): {}",
                skill.name, skill.category, skill.description
            );
            assert!(!skill.name.is_empty(), "Skill name should not be empty");
            assert!(
                !skill.category.is_empty(),
                "Skill category should not be empty"
            );
        }
        let skill_names: Vec<&str> = metadata.iter().map(|s| s.name.as_str()).collect();
        assert!(
            skill_names.contains(&"file_read"),
            "file_read skill should be present"
        );
        assert!(
            skill_names.contains(&"math_calculator"),
            "math_calculator skill should be present"
        );
    }

    /// Test that the registry JSON generation produces valid output
    #[test]
    fn test_print_all_skill_json() {
        let json_value = generate_skill_registry_table_json();
        println!("Registry JSON: {:?}", json_value);
        assert!(
            json_value["version"].is_string(),
            "version field should be a string"
        );
        assert_eq!(json_value["version"].as_str().unwrap(), "1.0");
        assert!(
            json_value["total_skills"].is_u64(),
            "total_skills field should be a number"
        );
        assert!(
            json_value["total_skills"].as_u64().unwrap() > 0,
            "total_skills should be positive"
        );
        assert!(
            json_value["skills"].is_array(),
            "skills field should be an array"
        );
        assert!(
            json_value["instruction"].is_string(),
            "instruction field should be a string"
        );
        let skills_array = json_value["skills"].as_array().unwrap();
        assert_eq!(
            skills_array.len(),
            json_value["total_skills"].as_u64().unwrap() as usize,
            "skills array length should match total_skills"
        );
    }

    /// Test registry operations: get, has, list, and register
    #[test]
    fn test_registry_operations() {
        let file_read_skill = get_skill("file_read");
        assert!(file_read_skill.is_some(), "file_read skill should exist");
        let non_existent = get_skill("non_existent_skill_12345");
        assert!(
            non_existent.is_none(),
            "Non-existent skill should return None"
        );
        assert!(
            has_skill("file_read"),
            "has_skill should return true for existing skill"
        );
        assert!(
            !has_skill("non_existent_skill_12345"),
            "has_skill should return false for non-existent skill"
        );
        let all_skills = list_skills();
        assert!(
            all_skills.contains(&"file_read".to_string()),
            "list_skills should include file_read"
        );
        assert!(
            all_skills.contains(&"math_calculator".to_string()),
            "list_skills should include math_calculator"
        );
        let skill_count_before = list_skills().len();
        assert!(skill_count_before > 0, "Registry should have skills");
    }

    /// Test that metadata is consistent across different retrieval methods
    #[test]
    fn test_metadata_consistency() {
        let registry_guard = get_registry();
        for (name, skill) in registry_guard.iter() {
            let metadata = skill.get_metadata();
            assert_eq!(
                &metadata.name, name,
                "Skill metadata name should match registry key"
            );
            assert!(
                !metadata.description.is_empty(),
                "Skill {} should have a description",
                name
            );
            assert!(
                !metadata.category.is_empty(),
                "Skill {} should have a category",
                name
            );
        }
    }
}

/// Get skills filtered by categories
pub fn get_skills_by_categories(categories: &[String]) -> Vec<Arc<dyn Skill>> {
    let registry = get_registry();
    let mut result = Vec::new();

    for (name, skill) in registry.iter() {
        let skill_category = skill.category();
        if categories.iter().any(|cat| cat == skill_category) {
            result.push(skill.clone());
        }
    }

    result
}

/// Get skill names filtered by categories
pub fn list_skills_by_categories(categories: &[String]) -> Vec<String> {
    let registry = get_registry();
    let mut result = Vec::new();

    for (name, skill) in registry.iter() {
        let skill_category = skill.category();
        if categories.iter().any(|cat| cat == skill_category) {
            result.push(name.clone());
        }
    }

    result
}

/// Get all available skill categories with their skill counts
pub fn get_skill_categories() -> Vec<(String, usize)> {
    let registry = get_registry();
    let mut category_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for skill in registry.values() {
        let cat = skill.category().to_string();
        *category_counts.entry(cat).or_insert(0) += 1;
    }
    let mut result: Vec<(String, usize)> = category_counts.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}
