//! Process memory operations utilities
//!
//! This module provides cross-platform process memory access utilities
//! for reading, writing, and scanning process memory.
use crate::DriverError;
use crate::result::DriverResult;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};
/// Process information structure
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub parent_pid: Option<u32>,
}
/// Memory region information
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start: usize,
    pub end: usize,
    pub size: usize,
    pub permissions: String,
    pub path: Option<String>,
}
/// Process memory accessor (platform-specific implementation)
#[cfg(target_os = "windows")]
pub mod platform {
    use crate::DriverError;
    use crate::result::DriverResult;
    use std::path::PathBuf;
    use std::ptr;
    use tracing::{debug, info, warn};
    use winapi::ctypes::c_void;
    use winapi::shared::minwindef::{DWORD, FALSE};
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::psapi::{EnumProcessModules, GetModuleFileNameExW};
    use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS};
    use winapi::um::winnt::{
        HANDLE, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
    };
    pub struct ProcessMemory {
        handle: HANDLE,
        pub pid: u32,
    }
    impl ProcessMemory {
        pub fn open(pid: u32, read_only: bool) -> DriverResult<Self> {
            debug!("Opening process memory for PID: {}, read_only: {}", pid, read_only);
            let access = if read_only {
                PROCESS_VM_READ | PROCESS_QUERY_INFORMATION
            } else {
                PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_QUERY_INFORMATION
            };
            let handle = unsafe { OpenProcess(access, 0, pid) };
            if handle.is_null() {
                let err = unsafe { GetLastError() };
                let err_msg = format!("Failed to open process with PID {}: error {}", pid, err);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
            info!("Process memory opened for PID: {}", pid);
            return Ok(Self { handle, pid });
        }
        pub fn read_memory(&self, address: usize, buffer: &mut [u8]) -> DriverResult<usize> {
            let mut bytes_read: usize = 0;
            let success = unsafe {
                ReadProcessMemory(self.handle, address as *const c_void, buffer.as_mut_ptr() as *mut c_void, buffer.len(), &mut bytes_read as *mut _)
            };
            if success == FALSE {
                let err = unsafe { GetLastError() };
                let err_msg = format!("Failed to read memory at address 0x{:X}: error {}", address, err);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
            debug!("Read {} bytes from address 0x{:X}", bytes_read, address);
            return Ok(bytes_read);
        }
        pub fn write_memory(&self, address: usize, data: &[u8]) -> DriverResult<usize> {
            let mut bytes_written: usize = 0;
            let success = unsafe {
                WriteProcessMemory(self.handle, address as *mut c_void, data.as_ptr() as *const c_void, data.len(), &mut bytes_written as *mut _)
            };
            if success == FALSE {
                let err = unsafe { GetLastError() };
                let err_msg = format!("Failed to write memory at address 0x{:X}: error {}", address, err);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
            debug!("Wrote {} bytes to address 0x{:X}", bytes_written, address);
            return Ok(bytes_written);
        }
        pub fn get_module_base(&self, module_name: &str) -> DriverResult<usize> {
            use winapi::shared::minwindef::HMODULE;
            let mut modules = vec![0 as HMODULE; 1024];
            let mut cb_needed: DWORD = 0;
            let success =
                unsafe { EnumProcessModules(self.handle, modules.as_mut_ptr(), (modules.len() * std::mem::size_of::<u64>()) as u32, &mut cb_needed) };
            if success == FALSE {
                let err_msg = "Failed to enumerate modules".to_string();
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
            let module_count = cb_needed as usize / std::mem::size_of::<HMODULE>();
            for i in 0..module_count {
                let module_handle = modules[i];
                let mut module_path = [0u16; 260];
                let len = unsafe { GetModuleFileNameExW(self.handle, module_handle, module_path.as_mut_ptr(), module_path.len() as u32) };
                if len > 0 {
                    let path = String::from_utf16_lossy(&module_path[..len as usize]);
                    if let Some(name) = PathBuf::from(&path).file_name() {
                        if name.to_string_lossy().to_lowercase() == module_name.to_lowercase() {
                            info!("Module base found for {}: 0x{:X}", module_name, module_handle as usize);
                            return Ok(module_handle as usize);
                        }
                    }
                }
            }
            let err_msg = format!("Module not found: {}", module_name);
            warn!("{}", err_msg);
            return Err(DriverError::not_found(module_name));
        }
    }
    impl Drop for ProcessMemory {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.handle);
            }
        }
    }
    pub fn list_processes() -> DriverResult<Vec<super::ProcessInfo>> {
        debug!("Listing processes on Windows");
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot.is_null() {
            let err_msg = "Failed to create process snapshot".to_string();
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
        let mut processes = Vec::new();
        let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if unsafe { Process32FirstW(snapshot, &mut entry) } == 1 {
            loop {
                processes.push(super::ProcessInfo {
                    pid: entry.th32ProcessID,
                    name: String::from_utf16_lossy(&entry.szExeFile).to_string(),
                    parent_pid: Some(entry.th32ParentProcessID),
                });
                if unsafe { Process32NextW(snapshot, &mut entry) } != 1 {
                    break;
                }
            }
        }
        unsafe {
            CloseHandle(snapshot);
        }
        info!("Listed {} processes", processes.len());
        return Ok(processes);
    }
}
#[cfg(target_os = "linux")]
pub mod platform {
    use crate::DriverError;
    use crate::result::DriverResult;
    use std::fs::{self, File};
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::path::Path;
    use tracing::{debug, info, warn};
    pub struct ProcessMemory {
        mem_file: File,
        pub pid: u32,
    }
    impl ProcessMemory {
        pub fn open(pid: u32, _read_only: bool) -> DriverResult<Self> {
            debug!("Opening process memory for PID: {}", pid);
            let mem_path = format!("/proc/{}/mem", pid);
            match File::open(&mem_path) {
                Ok(mem_file) => {
                    info!("Process memory opened for PID: {}", pid);
                    return Ok(Self { mem_file, pid });
                }
                Err(e) => {
                    let err_msg = format!("Failed to open memory file for PID {}: {}", pid, e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            }
        }
        pub fn read_memory(&mut self, address: usize, buffer: &mut [u8]) -> DriverResult<usize> {
            match self.mem_file.seek(SeekFrom::Start(address as u64)) {
                Ok(_) => {}
                Err(e) => {
                    let err_msg = format!("Failed to seek to address 0x{:X}: {}", address, e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            }
            match self.mem_file.read(buffer) {
                Ok(bytes_read) => {
                    debug!("Read {} bytes from address 0x{:X}", bytes_read, address);
                    return Ok(bytes_read);
                }
                Err(e) => {
                    let err_msg = format!("Failed to read memory at address 0x{:X}: {}", address, e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            }
        }
        pub fn write_memory(&mut self, address: usize, data: &[u8]) -> DriverResult<usize> {
            match self.mem_file.seek(SeekFrom::Start(address as u64)) {
                Ok(_) => {}
                Err(e) => {
                    let err_msg = format!("Failed to seek to address 0x{:X}: {}", address, e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            }
            match self.mem_file.write(data) {
                Ok(bytes_written) => {
                    debug!("Wrote {} bytes to address 0x{:X}", bytes_written, address);
                    return Ok(bytes_written);
                }
                Err(e) => {
                    let err_msg = format!("Failed to write memory at address 0x{:X}: {}", address, e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            }
        }
        pub fn get_module_base(&self, module_name: &str) -> DriverResult<usize> {
            let maps_path = format!("/proc/{}/maps", self.pid);
            let content = match fs::read_to_string(&maps_path) {
                Ok(c) => c,
                Err(e) => {
                    let err_msg = format!("Failed to read maps file: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            };
            for line in content.lines() {
                if line.contains(module_name) {
                    if let Some(addr_str) = line.split_whitespace().next() {
                        if let Some(addr_start) = addr_str.split('-').next() {
                            if let Ok(addr) = usize::from_str_radix(addr_start, 16) {
                                info!("Module base found for {}: 0x{:X}", module_name, addr);
                                return Ok(addr);
                            }
                        }
                    }
                }
            }
            let err_msg = format!("Module not found: {}", module_name);
            warn!("{}", err_msg);
            return Err(DriverError::not_found(module_name));
        }
    }
    pub fn list_processes() -> DriverResult<Vec<super::ProcessInfo>> {
        debug!("Listing processes on Linux");
        let mut processes = Vec::new();
        for entry in match fs::read_dir("/proc") {
            Ok(e) => e,
            Err(e) => {
                let err_msg = format!("Failed to read /proc directory: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        } {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Failed to read directory entry: {}", e);
                    continue;
                }
            };
            let name = entry.file_name();
            if let Some(name_str) = name.to_str() {
                if let Ok(pid) = name_str.parse::<u32>() {
                    let status_path = format!("/proc/{}/status", pid);
                    let mut process_name = String::new();
                    let mut parent_pid = None;
                    if let Ok(content) = fs::read_to_string(&status_path) {
                        for line in content.lines() {
                            if line.starts_with("Name:") {
                                process_name = line.trim_start_matches("Name:").trim().to_string();
                            } else if line.starts_with("PPid:") {
                                if let Ok(ppid) = line.trim_start_matches("PPid:").trim().parse::<u32>() {
                                    parent_pid = Some(ppid);
                                }
                            }
                        }
                    }
                    processes.push(super::ProcessInfo { pid, name: process_name, parent_pid });
                }
            }
        }
        info!("Listed {} processes", processes.len());
        return Ok(processes);
    }
}
#[cfg(target_os = "macos")]
pub mod platform {
    use crate::DriverError;
    use crate::result::DriverResult;
    use libc::{mach_task_self, mach_vm_read_overwrite, mach_vm_write, task_t, vm_deallocate};
    use std::ptr;
    use tracing::{debug, info, warn};
    pub struct ProcessMemory {
        pub pid: u32,
        task: task_t,
    }
    impl ProcessMemory {
        pub fn open(pid: u32, _read_only: bool) -> DriverResult<Self> {
            use libc::task_for_pid;
            debug!("Opening process memory for PID: {}", pid);
            let mut task: task_t = 0;
            let result = unsafe { task_for_pid(mach_task_self(), pid, &mut task) };
            if result != 0 {
                let err_msg = format!("Failed to get task for PID {}", pid);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
            info!("Process memory opened for PID: {}", pid);
            return Ok(Self { pid, task });
        }
        pub fn read_memory(&self, address: usize, buffer: &mut [u8]) -> DriverResult<usize> {
            let mut bytes_read = 0u64;
            let result =
                unsafe { mach_vm_read_overwrite(self.task, address as u64, buffer.len() as u64, buffer.as_mut_ptr() as u64, &mut bytes_read) };
            if result != 0 {
                let err_msg = format!("Failed to read memory at address 0x{:X}", address);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
            debug!("Read {} bytes from address 0x{:X}", bytes_read, address);
            return Ok(bytes_read as usize);
        }
        pub fn write_memory(&self, address: usize, data: &[u8]) -> DriverResult<usize> {
            let result = unsafe { mach_vm_write(self.task, address as u64, data.as_ptr() as u64, data.len() as u64) };
            if result != 0 {
                let err_msg = format!("Failed to write memory at address 0x{:X}", address);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
            debug!("Wrote {} bytes to address 0x{:X}", data.len(), address);
            return Ok(data.len());
        }
        pub fn get_module_base(&self, _module_name: &str) -> DriverResult<usize> {
            let err_msg = "Module base lookup not implemented for macOS".to_string();
            warn!("{}", err_msg);
            return Err(DriverError::internal(err_msg));
        }
    }
    impl Drop for ProcessMemory {
        fn drop(&mut self) {
            // Task reference cleanup
        }
    }
    pub fn list_processes() -> DriverResult<Vec<super::ProcessInfo>> {
        use libproc::libproc::bsd_info::BSDInfo;
        use libproc::libproc::proc_pid::{PidInfo, pidinfo};
        use libproc::libproc::processes::pids;
        debug!("Listing processes on macOS");
        let pids = pids();
        let mut processes = Vec::new();
        for pid in pids {
            if let Ok(bsd_info) = pidinfo::<BSDInfo>(pid as i32, 0) {
                let name = String::from_utf8_lossy(&bsd_info.pbi_name).to_string();
                processes.push(super::ProcessInfo { pid, name: name.trim_end_matches('\0').to_string(), parent_pid: Some(bsd_info.pbi_ppid as u32) });
            }
        }
        info!("Listed {} processes", processes.len());
        return Ok(processes);
    }
}
/// Process memory accessor (unified interface)
pub struct ProcessMemory {
    inner: platform::ProcessMemory,
    pub pid: u32,
}
impl ProcessMemory {
    pub fn open(pid: u32, read_only: bool) -> DriverResult<Self> {
        let inner = platform::ProcessMemory::open(pid, read_only)?;
        return Ok(Self { inner, pid });
    }
    pub fn read_memory(&mut self, address: usize, buffer: &mut [u8]) -> DriverResult<usize> {
        return self.inner.read_memory(address, buffer);
    }
    pub fn read_u8(&mut self, address: usize) -> DriverResult<u8> {
        let mut buf = [0u8; 1];
        self.read_memory(address, &mut buf)?;
        return Ok(buf[0]);
    }
    pub fn read_u16(&mut self, address: usize) -> DriverResult<u16> {
        let mut buf = [0u8; 2];
        self.read_memory(address, &mut buf)?;
        return Ok(u16::from_le_bytes(buf));
    }
    pub fn read_u32(&mut self, address: usize) -> DriverResult<u32> {
        let mut buf = [0u8; 4];
        self.read_memory(address, &mut buf)?;
        return Ok(u32::from_le_bytes(buf));
    }
    pub fn read_u64(&mut self, address: usize) -> DriverResult<u64> {
        let mut buf = [0u8; 8];
        self.read_memory(address, &mut buf)?;
        return Ok(u64::from_le_bytes(buf));
    }
    pub fn read_f32(&mut self, address: usize) -> DriverResult<f32> {
        let value = self.read_u32(address)?;
        return Ok(f32::from_bits(value));
    }
    pub fn read_f64(&mut self, address: usize) -> DriverResult<f64> {
        let value = self.read_u64(address)?;
        return Ok(f64::from_bits(value));
    }
    pub fn read_string(&mut self, address: usize, max_len: usize) -> DriverResult<String> {
        let mut buf = vec![0u8; max_len];
        let bytes_read = self.read_memory(address, &mut buf)?;
        if let Some(null_pos) = buf[..bytes_read].iter().position(|&b| b == 0) {
            return Ok(String::from_utf8_lossy(&buf[..null_pos]).to_string());
        } else {
            return Ok(String::from_utf8_lossy(&buf[..bytes_read]).to_string());
        }
    }
    pub fn write_memory(&mut self, address: usize, data: &[u8]) -> DriverResult<usize> {
        return self.inner.write_memory(address, data);
    }
    pub fn write_u8(&mut self, address: usize, value: u8) -> DriverResult<()> {
        self.write_memory(address, &[value])?;
        return Ok(());
    }
    pub fn write_u16(&mut self, address: usize, value: u16) -> DriverResult<()> {
        self.write_memory(address, &value.to_le_bytes())?;
        return Ok(());
    }
    pub fn write_u32(&mut self, address: usize, value: u32) -> DriverResult<()> {
        self.write_memory(address, &value.to_le_bytes())?;
        return Ok(());
    }
    pub fn write_u64(&mut self, address: usize, value: u64) -> DriverResult<()> {
        self.write_memory(address, &value.to_le_bytes())?;
        return Ok(());
    }
    pub fn write_f32(&mut self, address: usize, value: f32) -> DriverResult<()> {
        self.write_u32(address, value.to_bits())
    }
    pub fn write_f64(&mut self, address: usize, value: f64) -> DriverResult<()> {
        self.write_u64(address, value.to_bits())
    }
    pub fn get_module_base(&self, module_name: &str) -> DriverResult<usize> {
        return self.inner.get_module_base(module_name);
    }
}
/// List all running processes
pub fn list_processes() -> DriverResult<Vec<ProcessInfo>> {
    return platform::list_processes();
}
/// Find process by name
pub fn find_process_by_name(name: &str) -> DriverResult<Vec<ProcessInfo>> {
    debug!("Finding process by name: {}", name);
    let name_lower = name.to_lowercase();
    let processes = list_processes()?;
    let result: Vec<ProcessInfo> = processes.into_iter().filter(|p| p.name.to_lowercase().contains(&name_lower)).collect();
    info!("Found {} processes matching name: {}", result.len(), name);
    return Ok(result);
}
/// Pattern scanner for memory
#[derive(Debug, Clone)]
pub struct Pattern {
    pub bytes: Vec<Option<u8>>,
    pub mask: Vec<bool>,
}
impl Pattern {
    pub fn from_hex(hex_pattern: &str) -> DriverResult<Self> {
        debug!("Creating pattern from hex: {}", hex_pattern);
        let parts: Vec<&str> = hex_pattern.split_whitespace().collect();
        let mut bytes = Vec::new();
        let mut mask = Vec::new();
        for part in parts {
            if part == "?" || part == "??" {
                bytes.push(None);
                mask.push(false);
            } else if part.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(part, 16) {
                    bytes.push(Some(byte));
                    mask.push(true);
                } else {
                    let err_msg = format!("Invalid hex byte: {}", part);
                    warn!("{}", err_msg);
                    return Err(DriverError::validation("hex_pattern", err_msg));
                }
            } else {
                let err_msg = format!("Invalid pattern part: {}", part);
                warn!("{}", err_msg);
                return Err(DriverError::validation("hex_pattern", err_msg));
            }
        }
        info!("Created pattern with {} bytes", bytes.len());
        return Ok(Self { bytes, mask });
    }
    pub fn matches(&self, data: &[u8]) -> bool {
        if data.len() < self.bytes.len() {
            return false;
        }
        for (i, byte_opt) in self.bytes.iter().enumerate() {
            if let Some(byte) = byte_opt {
                if data[i] != *byte {
                    return false;
                }
            }
        }
        return true;
    }
    pub fn len(&self) -> usize {
        return self.bytes.len();
    }
    pub fn is_empty(&self) -> bool {
        return self.bytes.is_empty();
    }
}
/// Scan memory region for pattern
pub fn scan_region(memory: &mut ProcessMemory, start: usize, size: usize, pattern: &Pattern) -> DriverResult<Vec<usize>> {
    debug!("Scanning memory region: start=0x{:X}, size={}", start, size);
    let mut results = Vec::new();
    let buffer_size = 4096;
    let mut buffer = vec![0u8; buffer_size];
    let pattern_len = pattern.len();
    if pattern_len == 0 {
        let err_msg = "Pattern is empty".to_string();
        warn!("{}", err_msg);
        return Err(DriverError::validation("pattern", err_msg));
    }
    for offset in (0..size).step_by(buffer_size - pattern_len + 1) {
        let read_size = std::cmp::min(buffer_size, size - offset);
        let bytes_read = memory.read_memory(start + offset, &mut buffer[..read_size])?;
        for i in 0..bytes_read.saturating_sub(pattern_len) + 1 {
            if pattern.matches(&buffer[i..i + pattern_len]) {
                let found_addr = start + offset + i;
                debug!("Pattern found at address: 0x{:X}", found_addr);
                results.push(found_addr);
            }
        }
    }
    info!("Found {} pattern matches in memory region", results.len());
    return Ok(results);
}
