//! Process memory operations utilities
//!
//! This module provides cross-platform process memory access utilities
//! for reading, writing, and scanning process memory.

use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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
    use anyhow::Result;
    use std::path::PathBuf;
    use std::ptr;
    use winapi::ctypes::c_void;
    use winapi::shared::minwindef::{DWORD, FALSE};
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::psapi::{EnumProcessModules, GetModuleFileNameExW};
    use winapi::um::tlhelp32::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };
    use winapi::um::winnt::{
        HANDLE, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_OPERATION,
        PROCESS_VM_READ, PROCESS_VM_WRITE,
    };

    pub struct ProcessMemory {
        handle: HANDLE,
        pub pid: u32,
    }

    impl ProcessMemory {
        pub fn open(pid: u32, read_only: bool) -> Result<Self> {
            let access = if read_only {
                PROCESS_VM_READ | PROCESS_QUERY_INFORMATION
            } else {
                PROCESS_VM_READ
                    | PROCESS_VM_WRITE
                    | PROCESS_VM_OPERATION
                    | PROCESS_QUERY_INFORMATION
            };
            let handle = unsafe { OpenProcess(access, 0, pid) };
            if handle.is_null() {
                let err = unsafe { GetLastError() };
                anyhow::bail!("Failed to open process with PID {}: error {}", pid, err);
            }
            Ok(Self { handle, pid })
        }

        pub fn read_memory(&self, address: usize, buffer: &mut [u8]) -> Result<usize> {
            let mut bytes_read: usize = 0;
            let success = unsafe {
                ReadProcessMemory(
                    self.handle,
                    address as *const c_void,
                    buffer.as_mut_ptr() as *mut c_void,
                    buffer.len(),
                    &mut bytes_read as *mut _,
                )
            };
            if success == FALSE {
                let err = unsafe { GetLastError() };
                anyhow::bail!(
                    "Failed to read memory at address 0x{:X}: error {}",
                    address,
                    err
                );
            }
            Ok(bytes_read)
        }

        pub fn write_memory(&self, address: usize, data: &[u8]) -> Result<usize> {
            let mut bytes_written: usize = 0;
            let success = unsafe {
                WriteProcessMemory(
                    self.handle,
                    address as *mut c_void,
                    data.as_ptr() as *const c_void,
                    data.len(),
                    &mut bytes_written as *mut _,
                )
            };
            if success == FALSE {
                let err = unsafe { GetLastError() };
                anyhow::bail!(
                    "Failed to write memory at address 0x{:X}: error {}",
                    address,
                    err
                );
            }
            Ok(bytes_written)
        }

        pub fn get_module_base(&self, module_name: &str) -> Result<usize> {
            use winapi::shared::minwindef::HMODULE;
            let mut modules = vec![0 as HMODULE; 1024];
            let mut cb_needed: DWORD = 0;
            let success = unsafe {
                EnumProcessModules(
                    self.handle,
                    modules.as_mut_ptr(),
                    (modules.len() * std::mem::size_of::<u64>()) as u32,
                    &mut cb_needed,
                )
            };
            if success == FALSE {
                anyhow::bail!("Failed to enumerate modules");
            }
            let module_count = cb_needed as usize / std::mem::size_of::<HMODULE>();
            for i in 0..module_count {
                let module_handle = modules[i];
                let mut module_path = [0u16; 260];
                let len = unsafe {
                    GetModuleFileNameExW(
                        self.handle,
                        module_handle,
                        module_path.as_mut_ptr(),
                        module_path.len() as u32,
                    )
                };
                if len > 0 {
                    let path = String::from_utf16_lossy(&module_path[..len as usize]);
                    if let Some(name) = PathBuf::from(&path).file_name() {
                        if name.to_string_lossy().to_lowercase() == module_name.to_lowercase() {
                            return Ok(module_handle as usize);
                        }
                    }
                }
            }
            anyhow::bail!("Module not found: {}", module_name)
        }
    }

    impl Drop for ProcessMemory {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.handle);
            }
        }
    }

    pub fn list_processes() -> Result<Vec<super::ProcessInfo>> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot.is_null() {
            anyhow::bail!("Failed to create process snapshot");
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
        Ok(processes)
    }
}

#[cfg(target_os = "linux")]
pub mod platform {
    use anyhow::Result;
    use std::fs::{self, File};
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::path::Path;

    pub struct ProcessMemory {
        mem_file: File,
        pub pid: u32,
    }

    impl ProcessMemory {
        pub fn open(pid: u32, _read_only: bool) -> Result<Self> {
            let mem_path = format!("/proc/{}/mem", pid);
            let mem_file = File::open(&mem_path)?;
            Ok(Self { mem_file, pid })
        }

        pub fn read_memory(&mut self, address: usize, buffer: &mut [u8]) -> Result<usize> {
            self.mem_file.seek(SeekFrom::Start(address as u64))?;
            let bytes_read = self.mem_file.read(buffer)?;
            Ok(bytes_read)
        }

        pub fn write_memory(&mut self, address: usize, data: &[u8]) -> Result<usize> {
            self.mem_file.seek(SeekFrom::Start(address as u64))?;
            let bytes_written = self.mem_file.write(data)?;
            Ok(bytes_written)
        }

        pub fn get_module_base(&self, module_name: &str) -> Result<usize> {
            let maps_path = format!("/proc/{}/maps", self.pid);
            let content = fs::read_to_string(&maps_path)?;
            for line in content.lines() {
                if line.contains(module_name) {
                    if let Some(addr_str) = line.split_whitespace().next() {
                        if let Some(addr_start) = addr_str.split('-').next() {
                            if let Ok(addr) = usize::from_str_radix(addr_start, 16) {
                                return Ok(addr);
                            }
                        }
                    }
                }
            }
            anyhow::bail!("Module not found: {}", module_name)
        }
    }

    pub fn list_processes() -> Result<Vec<super::ProcessInfo>> {
        let mut processes = Vec::new();
        for entry in fs::read_dir("/proc")? {
            let entry = entry?;
            let name = entry.file_name();
            if let Some(name_str) = name.to_str() {
                if let Ok(pid) = name_str.parse::<u32>() {
                    let status_path = format!("/proc/{}/status", pid);
                    let mut name = String::new();
                    let mut parent_pid = None;
                    if let Ok(content) = fs::read_to_string(&status_path) {
                        for line in content.lines() {
                            if line.starts_with("Name:") {
                                name = line.trim_start_matches("Name:").trim().to_string();
                            } else if line.starts_with("PPid:") {
                                if let Ok(ppid) =
                                    line.trim_start_matches("PPid:").trim().parse::<u32>()
                                {
                                    parent_pid = Some(ppid);
                                }
                            }
                        }
                    }
                    processes.push(super::ProcessInfo {
                        pid,
                        name,
                        parent_pid,
                    });
                }
            }
        }
        Ok(processes)
    }
}

#[cfg(target_os = "macos")]
pub mod platform {
    use anyhow::Result;
    use libc::{mach_task_self, mach_vm_read_overwrite, mach_vm_write, task_t, vm_deallocate};
    use std::ptr;

    pub struct ProcessMemory {
        pub pid: u32,
        task: task_t,
    }

    impl ProcessMemory {
        pub fn open(pid: u32, _read_only: bool) -> Result<Self> {
            use libc::task_for_pid;
            let mut task: task_t = 0;
            let result = unsafe { task_for_pid(mach_task_self(), pid, &mut task) };
            if result != 0 {
                anyhow::bail!("Failed to get task for PID {}", pid);
            }
            Ok(Self { pid, task })
        }

        pub fn read_memory(&self, address: usize, buffer: &mut [u8]) -> Result<usize> {
            let mut bytes_read = 0u64;
            let result = unsafe {
                mach_vm_read_overwrite(
                    self.task,
                    address as u64,
                    buffer.len() as u64,
                    buffer.as_mut_ptr() as u64,
                    &mut bytes_read,
                )
            };
            if result != 0 {
                anyhow::bail!("Failed to read memory at address 0x{:X}", address);
            }
            Ok(bytes_read as usize)
        }

        pub fn write_memory(&self, address: usize, data: &[u8]) -> Result<usize> {
            let result = unsafe {
                mach_vm_write(
                    self.task,
                    address as u64,
                    data.as_ptr() as u64,
                    data.len() as u64,
                )
            };
            if result != 0 {
                anyhow::bail!("Failed to write memory at address 0x{:X}", address);
            }
            Ok(data.len())
        }

        pub fn get_module_base(&self, _module_name: &str) -> Result<usize> {
            anyhow::bail!("Module base lookup not implemented for macOS")
        }
    }

    impl Drop for ProcessMemory {
        fn drop(&mut self) {
            // Task reference cleanup
        }
    }

    pub fn list_processes() -> Result<Vec<super::ProcessInfo>> {
        use libproc::libproc::bsd_info::BSDInfo;
        use libproc::libproc::proc_pid::{PidInfo, pidinfo};
        use libproc::libproc::processes::pids;
        let pids = pids();
        let mut processes = Vec::new();
        for pid in pids {
            if let Ok(bsd_info) = pidinfo::<BSDInfo>(pid as i32, 0) {
                let name = String::from_utf8_lossy(&bsd_info.pbi_name).to_string();
                processes.push(super::ProcessInfo {
                    pid,
                    name: name.trim_end_matches('\0').to_string(),
                    parent_pid: Some(bsd_info.pbi_ppid as u32),
                });
            }
        }
        Ok(processes)
    }
}

/// Process memory accessor (unified interface)
pub struct ProcessMemory {
    inner: platform::ProcessMemory,
    pub pid: u32,
}

impl ProcessMemory {
    pub fn open(pid: u32, read_only: bool) -> Result<Self> {
        let inner = platform::ProcessMemory::open(pid, read_only)?;
        Ok(Self { inner, pid })
    }

    pub fn read_memory(&mut self, address: usize, buffer: &mut [u8]) -> Result<usize> {
        self.inner.read_memory(address, buffer)
    }

    pub fn read_u8(&mut self, address: usize) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_memory(address, &mut buf)?;
        Ok(buf[0])
    }

    pub fn read_u16(&mut self, address: usize) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_memory(address, &mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub fn read_u32(&mut self, address: usize) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_memory(address, &mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_u64(&mut self, address: usize) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.read_memory(address, &mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    pub fn read_f32(&mut self, address: usize) -> Result<f32> {
        let value = self.read_u32(address)?;
        Ok(f32::from_bits(value))
    }

    pub fn read_f64(&mut self, address: usize) -> Result<f64> {
        let value = self.read_u64(address)?;
        Ok(f64::from_bits(value))
    }

    pub fn read_string(&mut self, address: usize, max_len: usize) -> Result<String> {
        let mut buf = vec![0u8; max_len];
        let bytes_read = self.read_memory(address, &mut buf)?;
        if let Some(null_pos) = buf[..bytes_read].iter().position(|&b| b == 0) {
            Ok(String::from_utf8_lossy(&buf[..null_pos]).to_string())
        } else {
            Ok(String::from_utf8_lossy(&buf[..bytes_read]).to_string())
        }
    }

    pub fn write_memory(&mut self, address: usize, data: &[u8]) -> Result<usize> {
        self.inner.write_memory(address, data)
    }

    pub fn write_u8(&mut self, address: usize, value: u8) -> Result<()> {
        self.write_memory(address, &[value])?;
        Ok(())
    }

    pub fn write_u16(&mut self, address: usize, value: u16) -> Result<()> {
        self.write_memory(address, &value.to_le_bytes())?;
        Ok(())
    }

    pub fn write_u32(&mut self, address: usize, value: u32) -> Result<()> {
        self.write_memory(address, &value.to_le_bytes())?;
        Ok(())
    }

    pub fn write_u64(&mut self, address: usize, value: u64) -> Result<()> {
        self.write_memory(address, &value.to_le_bytes())?;
        Ok(())
    }

    pub fn write_f32(&mut self, address: usize, value: f32) -> Result<()> {
        self.write_u32(address, value.to_bits())
    }

    pub fn write_f64(&mut self, address: usize, value: f64) -> Result<()> {
        self.write_u64(address, value.to_bits())
    }

    pub fn get_module_base(&self, module_name: &str) -> Result<usize> {
        self.inner.get_module_base(module_name)
    }
}

/// List all running processes
pub fn list_processes() -> Result<Vec<ProcessInfo>> {
    platform::list_processes()
}

/// Find process by name
pub fn find_process_by_name(name: &str) -> Result<Vec<ProcessInfo>> {
    let name_lower = name.to_lowercase();
    Ok(list_processes()?
        .into_iter()
        .filter(|p| p.name.to_lowercase().contains(&name_lower))
        .collect())
}

/// Pattern scanner for memory
#[derive(Debug, Clone)]
pub struct Pattern {
    pub bytes: Vec<Option<u8>>,
    pub mask: Vec<bool>,
}

impl Pattern {
    pub fn from_hex(hex_pattern: &str) -> Result<Self> {
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
                    anyhow::bail!("Invalid hex byte: {}", part);
                }
            } else {
                anyhow::bail!("Invalid pattern part: {}", part);
            }
        }

        Ok(Self { bytes, mask })
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
        true
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

/// Scan memory region for pattern
pub fn scan_region(
    memory: &mut ProcessMemory,
    start: usize,
    size: usize,
    pattern: &Pattern,
) -> Result<Vec<usize>> {
    let mut results = Vec::new();
    let buffer_size = 4096;
    let mut buffer = vec![0u8; buffer_size];
    let pattern_len = pattern.len();
    for offset in (0..size).step_by(buffer_size - pattern_len + 1) {
        let read_size = std::cmp::min(buffer_size, size - offset);
        let bytes_read = memory.read_memory(start + offset, &mut buffer[..read_size])?;
        for i in 0..bytes_read.saturating_sub(pattern_len) + 1 {
            if pattern.matches(&buffer[i..i + pattern_len]) {
                results.push(start + offset + i);
            }
        }
    }
    Ok(results)
}
