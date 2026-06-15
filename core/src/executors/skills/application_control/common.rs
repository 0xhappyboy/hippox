// application_control/shared.rs
//! Shared utilities for application control

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
}

/// Helper function to convert OsStr to String
fn os_str_to_string(os_str: &OsStr) -> String {
    os_str.to_string_lossy().to_string()
}

/// Helper function to convert Path to String
fn path_to_string(path: Option<&std::path::Path>) -> Option<String> {
    path.and_then(|p| p.to_str()).map(|s| s.to_string())
}

/// Find process by name - cross-platform using sysinfo
pub fn find_process_by_name(name: &str) -> Result<Vec<ProcessInfo>> {
    use sysinfo::{ProcessesToUpdate, System};

    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut processes = Vec::new();
    let name_lower = name.to_lowercase();

    for (pid, process) in sys.processes() {
        let process_name = os_str_to_string(process.name());
        if process_name.to_lowercase().contains(&name_lower) {
            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: process_name,
                path: path_to_string(process.exe()),
            });
        }
    }

    Ok(processes)
}

/// Launch application
pub fn launch_app(app_path: &str) -> Result<u32> {
    let child = Command::new(app_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(child.id())
}

/// Launch with arguments
pub fn launch_app_with_args(app_path: &str, args: &[String]) -> Result<u32> {
    let child = Command::new(app_path)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(child.id())
}

/// Launch as administrator
#[cfg(target_os = "windows")]
pub fn launch_as_admin(app_path: &str, args: &[String]) -> Result<u32> {
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    // Use PowerShell's Start-Process with -Verb RunAs
    let mut cmd = Command::new("powershell");
    let args_str = args.join(" ").replace("'", "\\'");
    let command = format!(
        "Start-Process -FilePath '{}' -ArgumentList '{}' -Verb RunAs -WindowStyle Hidden",
        app_path, args_str
    );
    cmd.args(["-Command", &command]);

    let child = cmd.spawn()?;
    // Small delay to allow the process to start
    thread::sleep(Duration::from_millis(500));

    // Return the PID of the PowerShell process
    Ok(child.id())
}

#[cfg(not(target_os = "windows"))]
pub fn launch_as_admin(app_path: &str, args: &[String]) -> Result<u32> {
    let mut cmd = Command::new("sudo");
    cmd.arg(app_path);
    cmd.args(args);
    let child = cmd.spawn()?;
    Ok(child.id())
}

/// Kill process by PID
#[cfg(target_os = "windows")]
pub fn kill_process(pid: u32) -> Result<()> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};

    unsafe {
        if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) {
            let _ = TerminateProcess(handle, 1);
            let _ = CloseHandle(handle);
        }
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn kill_process(pid: u32) -> Result<()> {
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }
    Ok(())
}

/// Close window gracefully - try to close the main window
#[cfg(target_os = "windows")]
pub fn close_process_window(pid: u32) -> Result<()> {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE,
    };

    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let target_pid = lparam.0 as u32;
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == target_pid {
            let _ = PostMessageW(hwnd, WM_CLOSE, None, None);
        }
        BOOL::from(true)
    }

    unsafe {
        EnumWindows(Some(enum_callback), LPARAM(pid as isize));
    }

    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn close_process_window(pid: u32) -> Result<()> {
    // On Unix-like systems, send SIGTERM for graceful shutdown
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }
    Ok(())
}

/// Check if process is running using sysinfo
pub fn is_process_running(pid: u32) -> bool {
    use sysinfo::{ProcessesToUpdate, System};

    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    sys.processes().iter().any(|(p, _)| p.as_u32() == pid)
}

/// Wait for process to exit
pub async fn wait_for_exit(pid: u32, timeout_ms: u64) -> Result<bool> {
    let start = std::time::Instant::now();
    while is_process_running(pid) {
        if start.elapsed() > std::time::Duration::from_millis(timeout_ms) {
            return Ok(false);
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    Ok(true)
}

/// Get application path - find executable path
pub fn get_app_path(app_name: &str) -> Result<String> {
    // First check if it's a full path
    if PathBuf::from(app_name).exists() {
        return Ok(app_name.to_string());
    }

    #[cfg(target_os = "windows")]
    {
        // Check System32
        if let Ok(system_root) = std::env::var("SystemRoot") {
            let system32_path = PathBuf::from(&system_root).join("System32").join(app_name);
            if system32_path.exists() {
                return Ok(system32_path.to_string_lossy().to_string());
            }
        }

        // Check Program Files
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            let app_path = PathBuf::from(program_files).join(app_name);
            if app_path.exists() {
                return Ok(app_path.to_string_lossy().to_string());
            }
        }

        // Check Program Files (x86)
        if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
            let app_path = PathBuf::from(program_files_x86).join(app_name);
            if app_path.exists() {
                return Ok(app_path.to_string_lossy().to_string());
            }
        }

        // Use where command
        if let Ok(output) = Command::new("where").arg(app_name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = path.lines().next() {
                    return Ok(first_line.to_string());
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(output) = Command::new("which").arg(app_name).output() {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
    }

    Ok(app_name.to_string())
}

/// List all running processes using sysinfo
pub fn list_running_processes() -> Result<Vec<ProcessInfo>> {
    use sysinfo::{ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut processes = Vec::new();
    for (pid, process) in sys.processes() {
        processes.push(ProcessInfo {
            pid: pid.as_u32(),
            name: os_str_to_string(process.name()),
            path: path_to_string(process.exe()),
        });
    }
    Ok(processes)
}
