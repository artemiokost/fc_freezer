use std::{thread, time::Duration};
use fc_shared::{WriteMemoryRequest, IOCTL_WRITE_MEMORY, DEVICE_NAME_UTF16};
use fc_freezer::{get_process_id, ProcessDriver};
use windows_sys::Win32::Foundation::{INVALID_HANDLE_VALUE, GENERIC_READ, GENERIC_WRITE};
use windows_sys::Win32::System::IO::DeviceIoControl;

// ИСПРАВЛЕНО: Импортируем строго CreateFileW, соответствующую спецификации 0.61.2
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE
};

fn main() {
    unsafe {
        println!("[*] Инициализация fc_freezer [windows-sys 0.61.2]...");

        // Передаем указатель на подготовленный UTF-16 массив имени нашего драйвера
        let driver_handle = CreateFileW(
            DEVICE_NAME_UTF16.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut()
        );

        if driver_handle == INVALID_HANDLE_VALUE {
            println!("[!] Ошибка: Kernel-драйвер не запущен! Загрузите fc_driver.sys.");
            return;
        }
        println!("[+] Успешный доступ к Kernel-устройству получен.");

        println!("[*] Ожидание старта игры FC26.exe...");
        let mut pid = 0;
        while pid == 0 {
            pid = get_process_id("FC26.exe");
            thread::sleep(Duration::from_millis(50));
        }
        println!("[+] Процесс игры пойман! PID: {}", pid);

        if let Some(driver) = ProcessDriver::open(pid) {
            let bounds = driver.module_bounds();
            let ai_pattern = [0x48, 0x8B, 0x05, -1, -1, -1, -1, 0x48, 0x8B, 0x88, 0x8B, 0x01];

            println!("[*] Поиск сигнатур в памяти Frostbite...");
            if let Some(target_address) = driver.aob_scan(bounds.base_address, bounds.size_of_image, &ai_pattern) {
                println!("[+] Цель захвачена: 0x{:X}", target_address);
                println!("[*] Активирован защищенный цикл заморозки...");

                let request = WriteMemoryRequest {
                    process_id: pid,
                    target_address: target_address as u64,
                    value_to_write: 0,
                };

                let mut bytes_returned = 0;
                loop {
                    DeviceIoControl(
                        driver_handle,
                        IOCTL_WRITE_MEMORY,
                        &request as *const _ as *const _,
                        std::mem::size_of::<WriteMemoryRequest>() as u32,
                        std::ptr::null_mut(),
                        0,
                        &mut bytes_returned,
                        std::ptr::null_mut()
                    );
                    thread::sleep(Duration::from_millis(5));
                }
            } else {
                println!("[!] Ошибка: Сигнатура не обнаружена.");
            }
        }
    }
}
