//! Shared utilities for application control
//!
//! This module provides cross-platform utilities for process and application management,
//! including finding, launching, and controlling applications.
use crate::DriverError;
use crate::result::DriverResult;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::{debug, info, warn};
/// Process information structure containing process metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// Full path to the executable (if available)
    pub path: Option<String>,
}
/// Helper function to convert OsStr to String
fn os_str_to_string(os_str: &OsStr) -> String {
    return os_str.to_string_lossy().to_string();
}
/// Helper function to convert Path to String
fn path_to_string(path: Option<&std::path::Path>) -> Option<String> {
    return path.and_then(|p| p.to_str()).map(|s| s.to_string());
}
/// Find process by name - cross-platform using sysinfo
///
/// # Arguments
/// * `name` - Process name to search for (case-insensitive partial match)
///
/// # Returns
/// Vector of ProcessInfo for all matching processes
pub fn find_process_by_name(name: &str) -> DriverResult<Vec<ProcessInfo>> {
    debug!("Searching for processes matching: {}", name);
    use sysinfo::{ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut processes = Vec::new();
    let name_lower = name.to_lowercase();
    for (pid, process) in sys.processes() {
        let process_name = os_str_to_string(process.name());
        if process_name.to_lowercase().contains(&name_lower) {
            let info = ProcessInfo { pid: pid.as_u32(), name: process_name, path: path_to_string(process.exe()) };
            debug!("Found process: {} (PID: {})", info.name, info.pid);
            processes.push(info);
        }
    }
    info!("Found {} processes matching '{}'", processes.len(), name);
    return Ok(processes);
}
/// Launch an application
///
/// # Arguments
/// * `app_path` - Path to the application executable
///
/// # Returns
/// The PID of the launched process
pub fn launch_app(app_path: &str) -> DriverResult<u32> {
    debug!("Launching application: {}", app_path);
    let child = match Command::new(app_path).stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to launch application {}: {}", app_path, e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    let pid = child.id();
    info!("Application launched: {} (PID: {})", app_path, pid);
    return Ok(pid);
}
/// Launch an application with command-line arguments
///
/// # Arguments
/// * `app_path` - Path to the application executable
/// * `args` - Vector of command-line arguments
///
/// # Returns
/// The PID of the launched process
pub fn launch_app_with_args(app_path: &str, args: &[String]) -> DriverResult<u32> {
    debug!("Launching application with args: {} {:?}", app_path, args);
    let child = match Command::new(app_path).args(args).stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to launch application with args {}: {}", app_path, e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    let pid = child.id();
    info!("Application launched with args: {} (PID: {})", app_path, pid);
    return Ok(pid);
}
/// Launch an application with administrator privileges
///
/// # Arguments
/// * `app_path` - Path to the application executable
/// * `args` - Vector of command-line arguments
///
/// # Returns
/// The PID of the launched process
#[cfg(target_os = "windows")]
pub fn launch_as_admin(app_path: &str, args: &[String]) -> DriverResult<u32> {
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    debug!("Launching as admin: {} {:?}", app_path, args);
    // Use PowerShell's Start-Process with -Verb RunAs
    let mut cmd = Command::new("powershell");
    let args_str = args.join(" ").replace("'", "\\'");
    let command = format!("Start-Process -FilePath '{}' -ArgumentList '{}' -Verb RunAs -WindowStyle Hidden", app_path, args_str);
    cmd.args(["-Command", &command]);
    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to launch as admin {}: {}", app_path, e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    // Small delay to allow the process to start
    thread::sleep(Duration::from_millis(500));
    let pid = child.id();
    info!("Application launched as admin: {} (PID: {})", app_path, pid);
    return Ok(pid);
}
#[cfg(not(target_os = "windows"))]
pub fn launch_as_admin(app_path: &str, args: &[String]) -> DriverResult<u32> {
    debug!("Launching as admin (Unix): {} {:?}", app_path, args);
    let mut cmd = Command::new("sudo");
    cmd.arg(app_path);
    cmd.args(args);
    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to launch as admin {}: {}", app_path, e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    let pid = child.id();
    info!("Application launched as admin: {} (PID: {})", app_path, pid);
    return Ok(pid);
}
/// Kill a process by PID
///
/// # Arguments
/// * `pid` - Process ID to terminate
///
/// # Returns
/// Ok(()) on success
#[cfg(target_os = "windows")]
pub fn kill_process(pid: u32) -> DriverResult<()> {
    debug!("Killing process: {}", pid);
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};
    unsafe {
        if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) {
            let _ = TerminateProcess(handle, 1);
            let _ = CloseHandle(handle);
            info!("Process killed: {}", pid);
        } else {
            let err_msg = format!("Failed to open process {} for termination", pid);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    }
    return Ok(());
}
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn kill_process(pid: u32) -> DriverResult<()> {
    debug!("Killing process: {}", pid);
    unsafe {
        let result = libc::kill(pid as i32, libc::SIGTERM);
        if result != 0 {
            let err_msg = format!("Failed to kill process {}: errno {}", pid, result);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    }
    info!("Process killed: {}", pid);
    return Ok(());
}
/// Close a process window gracefully (send close message)
///
/// # Arguments
/// * `pid` - Process ID
///
/// # Returns
/// Ok(()) on success
#[cfg(target_os = "windows")]
pub fn close_process_window(pid: u32) -> DriverResult<()> {
    debug!("Closing process window: {}", pid);
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE};
    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let target_pid = lparam.0 as u32;
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == target_pid {
            let _ = PostMessageW(hwnd, WM_CLOSE, None, None);
        }
        return BOOL::from(true);
    }
    unsafe {
        EnumWindows(Some(enum_callback), LPARAM(pid as isize));
    }
    info!("Close message sent to process: {}", pid);
    return Ok(());
}
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn close_process_window(pid: u32) -> DriverResult<()> {
    // On Unix-like systems, send SIGTERM for graceful shutdown
    debug!("Closing process window (Unix): {}", pid);
    unsafe {
        let result = libc::kill(pid as i32, libc::SIGTERM);
        if result != 0 {
            let err_msg = format!("Failed to send SIGTERM to process {}: errno {}", pid, result);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    }
    info!("SIGTERM sent to process: {}", pid);
    return Ok(());
}
/// Check if a process is currently running
///
/// # Arguments
/// * `pid` - Process ID to check
///
/// # Returns
/// true if the process is running
pub fn is_process_running(pid: u32) -> bool {
    use sysinfo::{ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let running = sys.processes().iter().any(|(p, _)| p.as_u32() == pid);
    debug!("Process {} running: {}", pid, running);
    return running;
}
/// Wait for a process to exit
///
/// # Arguments
/// * `pid` - Process ID to wait for
/// * `timeout_ms` - Maximum wait time in milliseconds
///
/// # Returns
/// Ok(true) if process exited, Ok(false) if timeout
pub async fn wait_for_exit(pid: u32, timeout_ms: u64) -> DriverResult<bool> {
    debug!("Waiting for process {} to exit (timeout: {}ms)", pid, timeout_ms);
    let start = std::time::Instant::now();
    while is_process_running(pid) {
        if start.elapsed() > std::time::Duration::from_millis(timeout_ms) {
            warn!("Timeout waiting for process {} to exit", pid);
            return Ok(false);
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    info!("Process {} exited", pid);
    return Ok(true);
}
/// Get the full path of an application executable
///
/// # Arguments
/// * `app_name` - Application name or path
///
/// # Returns
/// The full path to the executable
pub fn get_app_path(app_name: &str) -> DriverResult<String> {
    debug!("Getting application path for: {}", app_name);
    // First check if it's a full path
    if PathBuf::from(app_name).exists() {
        info!("Found application at: {}", app_name);
        return Ok(app_name.to_string());
    }
    #[cfg(target_os = "windows")]
    {
        // Check System32
        if let Ok(system_root) = std::env::var("SystemRoot") {
            let system32_path = PathBuf::from(&system_root).join("System32").join(app_name);
            if system32_path.exists() {
                let path = system32_path.to_string_lossy().to_string();
                info!("Found application in System32: {}", path);
                return Ok(path);
            }
        }
        // Check Program Files
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            let app_path = PathBuf::from(program_files).join(app_name);
            if app_path.exists() {
                let path = app_path.to_string_lossy().to_string();
                info!("Found application in Program Files: {}", path);
                return Ok(path);
            }
        }
        // Check Program Files (x86)
        if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
            let app_path = PathBuf::from(program_files_x86).join(app_name);
            if app_path.exists() {
                let path = app_path.to_string_lossy().to_string();
                info!("Found application in Program Files (x86): {}", path);
                return Ok(path);
            }
        }
        // Use where command
        if let Ok(output) = Command::new("where").arg(app_name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = path.lines().next() {
                    info!("Found application via 'where': {}", first_line);
                    return Ok(first_line.to_string());
                }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(output) = Command::new("which").arg(app_name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                info!("Found application via 'which': {}", path);
                return Ok(path);
            }
        }
    }
    warn!("Application '{}' not found, returning as-is", app_name);
    return Ok(app_name.to_string());
}
/// List all running processes using sysinfo
///
/// # Returns
/// Vector of ProcessInfo for all running processes
pub fn list_running_processes() -> DriverResult<Vec<ProcessInfo>> {
    debug!("Listing all running processes");
    use sysinfo::{ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut processes = Vec::new();
    for (pid, process) in sys.processes() {
        processes.push(ProcessInfo { pid: pid.as_u32(), name: os_str_to_string(process.name()), path: path_to_string(process.exe()) });
    }
    info!("Found {} running processes", processes.len());
    return Ok(processes);
}
