pub mod trainer;
mod build;

use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS
};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_VM_READ, PROCESS_QUERY_INFORMATION};
use windows_sys::Win32::System::ProcessStatus::{EnumProcessModules, GetModuleInformation, MODULEINFO};
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};

pub struct ModuleBounds {
    pub base_address: u64,
    pub size_of_image: u32,
}

pub struct ProcessDriver {
    process_handle: HANDLE,
}

impl ProcessDriver {
    pub unsafe fn open(process_id: u32) -> Option<Self> {
        // Явно изолируем системные вызовы Windows в блоки unsafe для спецификации Rust 2024
        unsafe {
            let handle = OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, 0, process_id);
            if handle == core::ptr::null_mut() {
                None
            } else {
                Some(Self { process_handle: handle })
            }
        }
    }

    pub unsafe fn module_bounds(&self) -> ModuleBounds {
        let mut modules = [core::ptr::null_mut(); 1024];
        let mut cb_needed = 0u32;

        unsafe {
            if EnumProcessModules(
                self.process_handle,
                modules.as_mut_ptr(),
                std::mem::size_of_val(&modules) as u32,
                &mut cb_needed,
            ) != 0 {
                let mut mod_info = MODULEINFO {
                    lpBaseOfDll: std::ptr::null_mut(),
                    SizeOfImage: 0,
                    EntryPoint: std::ptr::null_mut(),
                };

                if GetModuleInformation(
                    self.process_handle,
                    modules[0],
                    &mut mod_info,
                    std::mem::size_of::<MODULEINFO>() as u32,
                ) != 0 {
                    return ModuleBounds {
                        base_address: mod_info.lpBaseOfDll as u64,
                        size_of_image: mod_info.SizeOfImage,
                    };
                }
            }
        }

        ModuleBounds { base_address: 0, size_of_image: 0 }
    }

    pub unsafe fn read_memory(&self, address: u64, buffer: *mut u8, size: usize) -> bool {
        use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
        let mut bytes_read = 0;
        unsafe {
            ReadProcessMemory(
                self.process_handle,
                address as *const core::ffi::c_void,
                buffer as *mut core::ffi::c_void,
                size,
                &mut bytes_read,
            ) != 0
        }
    }

    pub unsafe fn aob_scan(&self, base_address: u64, size: u32, pattern: &[i32]) -> Option<u64> {
        let mut chunk = vec![0u8; 4096];
        let mut offset = 0u32;

        while offset < size {
            let current_address = base_address + offset as u64;
            unsafe {
                if self.read_memory(current_address, chunk.as_mut_ptr(), chunk.len()) {
                    for i in 0..chunk.len() - pattern.len() {
                        let mut found = true;
                        for j in 0..pattern.len() {
                            if pattern[j] != -1 && pattern[j] != chunk[i + j] as i32 {
                                found = false;
                                break;
                            }
                        }
                        if found {
                            return Some(current_address + i as u64);
                        }
                    }
                }
            }
            offset += 4096 - pattern.len() as u32;
        }
        None
    }
}

impl Drop for ProcessDriver {
    fn drop(&mut self) {
        if self.process_handle != core::ptr::null_mut() {
            unsafe { CloseHandle(self.process_handle); }
        }
    }
}

pub unsafe fn get_process_id(process_name: &str) -> u32 {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return 0;
        }

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            cntUsage: 0,
            th32ProcessID: 0,
            th32DefaultHeapID: 0,
            th32ModuleID: 0,
            cntThreads: 0,
            th32ParentProcessID: 0,
            pcPriClassBase: 0,
            dwFlags: 0,
            szExeFile: [0u16; 260],
        };

        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let exe_name = String::from_utf16_lossy(&entry.szExeFile);
                if exe_name.contains(process_name) {
                    CloseHandle(snapshot);
                    return entry.th32ProcessID;
                }
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
        0
    }
}
