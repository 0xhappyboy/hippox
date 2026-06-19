//! Shared utilities for process memory operations

use serde::{Deserialize, Serialize};
use sysinfo::{Pid, ProcessRefreshKind, System};

/// Memory read result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryReadResult {
    pub address: usize,
    pub size: usize,
    pub data: Vec<u8>,
    pub data_type: String,
    pub value: Option<String>,
}

/// Memory scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryScanResult {
    pub address: usize,
    pub matched_bytes: Vec<u8>,
    pub module_name: Option<String>,
}

/// Module information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub base_address: usize,
    pub size: usize,
    pub path: Option<String>,
}

/// Memory region information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    pub start_address: usize,
    pub end_address: usize,
    pub size: usize,
    pub permissions: String,
    pub module_name: Option<String>,
}

/// Pattern for memory scanning with wildcards
#[derive(Debug, Clone)]
pub struct Pattern {
    pub bytes: Vec<u8>,
    pub masks: Vec<bool>,
}

impl Pattern {
    /// Create a pattern from hex string with wildcards
    /// Example: "48 8B 05 ? ? ? ?" where '?' is a wildcard
    pub fn from_hex(hex_str: &str) -> Result<Self, String> {
        let parts: Vec<&str> = hex_str.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty pattern".to_string());
        }

        let mut bytes = Vec::new();
        let mut masks = Vec::new();

        for part in parts {
            if part == "?" || part == "??" {
                bytes.push(0);
                masks.push(false);
            } else if part.len() == 2 {
                let byte = u8::from_str_radix(part, 16)
                    .map_err(|_| format!("Invalid hex byte: {}", part))?;
                bytes.push(byte);
                masks.push(true);
            } else {
                return Err(format!("Invalid pattern part: {}", part));
            }
        }

        Ok(Pattern { bytes, masks })
    }

    /// Check if a byte sequence matches this pattern
    pub fn matches(&self, data: &[u8]) -> bool {
        if data.len() < self.bytes.len() {
            return false;
        }

        for i in 0..self.bytes.len() {
            if self.masks[i] && data[i] != self.bytes[i] {
                return false;
            }
        }
        true
    }

    /// Get the length of the pattern
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Check if pattern is empty
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

/// Open a process for memory operations
pub fn open_process(pid: u32, read_only: bool) -> Result<ProcessMemory, String> {
    ProcessMemory::open(pid, read_only)
}

/// Process memory accessor
pub struct ProcessMemory {
    pid: u32,
    #[cfg(target_os = "windows")]
    handle: windows_sys::Win32::Foundation::HANDLE,
}

impl ProcessMemory {
    /// Open a process for memory operations
    pub fn open(pid: u32, _read_only: bool) -> Result<Self, String> {
        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::Foundation::HANDLE;
            use windows_sys::Win32::System::Threading::{
                OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ,
                PROCESS_VM_WRITE,
            };

            let desired_access = if _read_only {
                PROCESS_VM_READ | PROCESS_QUERY_INFORMATION
            } else {
                PROCESS_VM_READ
                    | PROCESS_VM_WRITE
                    | PROCESS_VM_OPERATION
                    | PROCESS_QUERY_INFORMATION
            };

            let handle = unsafe { OpenProcess(desired_access, 0, pid) };
            if handle == std::ptr::null_mut() {
                return Err(format!("Failed to open process with PID: {}", pid));
            }

            Ok(ProcessMemory { pid, handle })
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(ProcessMemory { pid })
        }
    }

    /// Read memory from the process
    pub fn read_memory(&mut self, address: usize, buffer: &mut [u8]) -> Result<usize, String> {
        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;

            let mut bytes_read = 0usize;
            let result = unsafe {
                ReadProcessMemory(
                    self.handle,
                    address as *const std::ffi::c_void,
                    buffer.as_mut_ptr() as *mut std::ffi::c_void,
                    buffer.len(),
                    &mut bytes_read,
                )
            };

            if result == 0 {
                return Err("Failed to read process memory".to_string());
            }

            Ok(bytes_read)
        }

        #[cfg(not(target_os = "windows"))]
        {
            #[cfg(target_os = "linux")]
            {
                use libc::{iovec, pid_t, process_vm_readv};

                let pid_t = self.pid as pid_t;
                let mut local_iov = iovec {
                    iov_base: buffer.as_mut_ptr() as *mut std::ffi::c_void,
                    iov_len: buffer.len(),
                };
                let remote_iov = iovec {
                    iov_base: address as *mut std::ffi::c_void,
                    iov_len: buffer.len(),
                };

                let result = unsafe {
                    process_vm_readv(
                        pid_t,
                        &mut local_iov as *mut iovec,
                        1,
                        &remote_iov as *const iovec,
                        1,
                        0,
                    )
                };

                if result < 0 {
                    return Err(format!(
                        "Failed to read process memory: {}",
                        std::io::Error::last_os_error()
                    ));
                }

                Ok(result as usize)
            }

            #[cfg(not(target_os = "linux"))]
            {
                Err("Memory reading not supported on this platform".to_string())
            }
        }
    }

    /// Read a u8 from memory
    pub fn read_u8(&mut self, address: usize) -> Result<u8, String> {
        let mut buffer = [0u8; 1];
        self.read_memory(address, &mut buffer)?;
        Ok(buffer[0])
    }

    /// Read a u16 from memory
    pub fn read_u16(&mut self, address: usize) -> Result<u16, String> {
        let mut buffer = [0u8; 2];
        self.read_memory(address, &mut buffer)?;
        Ok(u16::from_le_bytes(buffer))
    }

    /// Read a u32 from memory
    pub fn read_u32(&mut self, address: usize) -> Result<u32, String> {
        let mut buffer = [0u8; 4];
        self.read_memory(address, &mut buffer)?;
        Ok(u32::from_le_bytes(buffer))
    }

    /// Read a u64 from memory
    pub fn read_u64(&mut self, address: usize) -> Result<u64, String> {
        let mut buffer = [0u8; 8];
        self.read_memory(address, &mut buffer)?;
        Ok(u64::from_le_bytes(buffer))
    }

    /// Read a f32 from memory
    pub fn read_f32(&mut self, address: usize) -> Result<f32, String> {
        let mut buffer = [0u8; 4];
        self.read_memory(address, &mut buffer)?;
        Ok(f32::from_le_bytes(buffer))
    }

    /// Read a f64 from memory
    pub fn read_f64(&mut self, address: usize) -> Result<f64, String> {
        let mut buffer = [0u8; 8];
        self.read_memory(address, &mut buffer)?;
        Ok(f64::from_le_bytes(buffer))
    }

    /// Read a string from memory
    pub fn read_string(&mut self, address: usize, max_len: usize) -> Result<String, String> {
        let mut buffer = vec![0u8; max_len];
        let bytes_read = self.read_memory(address, &mut buffer)?;
        let end = buffer[..bytes_read]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(bytes_read);
        String::from_utf8(buffer[..end].to_vec()).map_err(|e| format!("Invalid UTF-8: {}", e))
    }

    /// Get module base address using sysinfo
    pub fn get_module_base(&self, module_name: &str) -> Result<usize, String> {
        let modules = self.get_modules()?;
        let module_lower = module_name.to_lowercase();
        for module in modules {
            if module.name.to_lowercase().contains(&module_lower) {
                return Ok(module.base_address);
            }
        }
        Err(format!("Module not found: {}", module_name))
    }

    /// Get all loaded modules using sysinfo
    pub fn get_modules(&self) -> Result<Vec<ModuleInfo>, String> {
        let mut system = System::new();
        system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );

        let pid = Pid::from_u32(self.pid);
        if let Some(process) = system.process(pid) {
            let mut modules = Vec::new();

            #[cfg(target_os = "linux")]
            {
                // On Linux, read from /proc/pid/maps
                let path = format!("/proc/{}/maps", self.pid);
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let mut seen_paths = std::collections::HashSet::new();
                    for line in content.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 6 {
                            let addr_parts: Vec<&str> = parts[0].split('-').collect();
                            if addr_parts.len() == 2 {
                                let base = usize::from_str_radix(addr_parts[0], 16).unwrap_or(0);
                                let end = usize::from_str_radix(addr_parts[1], 16).unwrap_or(0);
                                let path = parts[5];
                                if path != "[vdso]"
                                    && path != "[vsyscall]"
                                    && path != "[heap]"
                                    && path != "[stack]"
                                    && !path.is_empty()
                                {
                                    let name = path.split('/').last().unwrap_or(path);
                                    if !seen_paths.contains(name) && !name.is_empty() {
                                        seen_paths.insert(name.to_string());
                                        modules.push(ModuleInfo {
                                            name: name.to_string(),
                                            base_address: base,
                                            size: end - base,
                                            path: Some(path.to_string()),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
            #[cfg(target_os = "windows")]
            {
                // On Windows, use the process executable path and known DLLs
                if let Some(exe_path) = process.exe() {
                    if let Some(name) = exe_path.file_name() {
                        if let Some(name_str) = name.to_str() {
                            modules.push(ModuleInfo {
                                name: name_str.to_string(),
                                base_address: 0, // Would need proper Windows API to get this
                                size: 0,
                                path: Some(exe_path.to_string_lossy().to_string()),
                            });
                        }
                    }
                }
            }
            Ok(modules)
        } else {
            Err(format!("Process with PID {} not found", self.pid))
        }
    }

    /// List all processes
    pub fn list_processes() -> Result<Vec<ProcessInfo>, String> {
        let mut system = System::new();
        system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );
        let mut processes = Vec::new();
        for (pid, process) in system.processes() {
            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                parent_pid: process.parent().map(|p| p.as_u32()),
            });
        }
        Ok(processes)
    }

    /// Scan memory for a pattern
    pub fn scan(
        &mut self,
        start_address: usize,
        size: usize,
        pattern: &Pattern,
    ) -> Result<Vec<usize>, String> {
        let mut results = Vec::new();
        let chunk_size = 4096;
        let mut buffer = vec![0u8; chunk_size + pattern.len()];

        let mut current_addr = start_address;
        let end_addr = start_address + size;

        while current_addr < end_addr {
            let remaining = end_addr - current_addr;
            let read_size = std::cmp::min(chunk_size + pattern.len(), remaining);
            let read_result = self.read_memory(current_addr, &mut buffer[..read_size]);

            if let Ok(bytes_read) = read_result {
                if bytes_read == 0 {
                    break;
                }

                let search_end = bytes_read - pattern.len() + 1;
                for i in 0..search_end {
                    if pattern.matches(&buffer[i..i + pattern.len()]) {
                        results.push(current_addr + i);
                    }
                }

                current_addr += bytes_read - pattern.len() + 1;
            } else {
                current_addr = (current_addr + 4096) & !4095;
            }

            if results.len() >= 1000 {
                break;
            }
        }

        Ok(results)
    }
}

impl Drop for ProcessMemory {
    #[cfg(target_os = "windows")]
    fn drop(&mut self) {
        use windows_sys::Win32::Foundation::CloseHandle;
        unsafe {
            CloseHandle(self.handle);
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn drop(&mut self) {}
}

/// Process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub parent_pid: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_from_hex() {
        let pattern = Pattern::from_hex("48 8B 05 ? ? ? ?").unwrap();
        assert_eq!(pattern.bytes.len(), 7);
        assert_eq!(pattern.masks.len(), 7);
        assert!(pattern.masks[0]);
        assert!(!pattern.masks[3]);
    }

    #[test]
    fn test_pattern_matches() {
        let pattern = Pattern::from_hex("48 8B ? ? ? ? ?").unwrap();
        let data = vec![0x48, 0x8B, 0x05, 0x00, 0x00, 0x00, 0x00];
        assert!(pattern.matches(&data));
    }

    #[test]
    fn test_process_info_creation() {
        let info = ProcessInfo {
            pid: 1234,
            name: "test".to_string(),
            parent_pid: Some(5678),
        };
        assert_eq!(info.pid, 1234);
        assert_eq!(info.name, "test");
        assert_eq!(info.parent_pid, Some(5678));
    }
}
