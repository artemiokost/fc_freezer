use windows_sys::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows_sys::Win32::System::ProcessStatus::{GetModuleInformation, MODULEINFO};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_VM_READ, PROCESS_QUERY_INFORMATION};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};

pub struct ProcessDriver {
    handle: HANDLE,
}

pub struct ModuleBounds {
    pub base_address: usize,
    pub size_of_image: usize,
}

impl ProcessDriver {
    pub unsafe fn open(pid: u32) -> Option<Self> {
        // Права пассивного чтения
        let handle = unsafe { OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, 0, pid) };
        if handle.is_null() {
            return None;
        }
        Some(Self { handle })
    }

    pub unsafe fn module_bounds(&self) -> ModuleBounds {
        unsafe {
            let mut module_info: MODULEINFO = std::mem::zeroed();
            GetModuleInformation(
                self.handle,
                self.handle,
                &mut module_info,
                std::mem::size_of::<MODULEINFO>() as u32,
            );

            ModuleBounds {
                base_address: module_info.lpBaseOfDll as usize,
                size_of_image: module_info.SizeOfImage as usize,
            }
        }
    }

    pub unsafe fn aob_scan(&self, start_addr: usize, size: usize, pattern: &[i32]) -> Option<usize> {
        let mut buffer = vec![0u8; 4096];
        let chunks = (size + buffer.len() - 1) / buffer.len();

        for i in 0..chunks {
            let current_addr = start_addr + (i * buffer.len());
            let mut bytes_read = 0;

            unsafe {
                ReadProcessMemory(
                    self.handle,
                    current_addr as *const _,
                    buffer.as_mut_ptr() as *mut _,
                    buffer.len(),
                    &mut bytes_read,
                );
            }
            if bytes_read == 0 {
                continue;
            }

            for j in 0..(bytes_read.saturating_sub(pattern.len())) {
                let mut found = true;
                for k in 0..pattern.len() {
                    if pattern[k] != -1 && buffer[j + k] != pattern[k] as u8 {
                        found = false;
                        break;
                    }
                }
                if found {
                    return Some(current_addr + j);
                }
            }
        }
        None
    }
}

pub unsafe fn get_process_id(process_name: &str) -> u32 {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return 0;
        }

        let mut entry: PROCESSENTRY32 = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        if Process32First(snapshot, &mut entry) != 0 {
            loop {
                let current_name = std::ffi::CStr::from_ptr(entry.szExeFile.as_ptr() as *const i8)
                    .to_string_lossy();

                if current_name.to_lowercase().contains(&process_name.to_lowercase()) {
                    return entry.th32ProcessID;
                }

                if Process32Next(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
    }
    0
}
