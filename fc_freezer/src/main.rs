use std::{thread, time::Duration, io};
use fc_shared::WriteMemoryRequest;
use fc_freezer::{get_process_id, ProcessDriver};

// Импортируем функцию прямого системного вызова из динамической библиотеки Windows
#[link(name = "ntdll")]
unsafe extern "system" { // ИСПРАВЛЕНО: Добавлено ключевое слово unsafe для Rust 2024
    fn NtQuerySystemInformation(
        system_information_class: u32,
        system_information: *mut core::ffi::c_void,
        system_information_length: u32,
        return_length: *mut u32,
    ) -> i32;
}

fn main() {
    unsafe {
        println!("=======================================================");
        println!("         ЗАПУСК ПОЛЬЗОВАТЕЛЬСКОГО СКАНЕРА FC_FREEZER   ");
        println!("=======================================================");
        println!("[*] Инициализация сканера по беспроводному Kernel-хуку...");
        println!("[*] Ожидание запуска игры FC26.exe...");

        let mut pid = 0;
        while pid == 0 {
            pid = get_process_id("FC26.exe");
            thread::sleep(Duration::from_millis(100));
        }
        println!("[+] Процесс игры пойман! PID: {}", pid);

        if let Some(driver) = ProcessDriver::open(pid) {
            let bounds = driver.module_bounds();
            let ai_pattern = [0x48, 0x8B, 0x05, -1, -1, -1, -1, 0x48, 0x8B, 0x88, 0x8B, 0x01];

            println!("[*] Поиск сигнатур в адресном пространстве Frostbite...");
            if let Some(target_address) = driver.aob_scan(bounds.base_address, bounds.size_of_image, &ai_pattern) {
                println!("[+] Цель успешно захвачена: 0x{:X}", target_address);
                println!("[*] Отправка команд через системную шину ядра Windows...");
                println!("[+] Заморозка ИИ активирована. Для остановки закройте это окно.");

                let request = WriteMemoryRequest {
                    process_id: pid,
                    target_address: target_address as u64,
                    value_to_write: 0, // Удерживаем ноль (паралич)
                };

                loop {
                    // Вызываем легальную системную функцию Windows.
                    // Наш работающий в ядре драйвер перехватит этот вызов по магическому коду 0x777FFFFF
                    // и произведет скрытую аппаратную запись в оперативную память.
                    NtQuerySystemInformation(
                        0x777FFFFF, // Магический код команды
                        &request as *const _ as *mut core::ffi::c_void,
                        std::mem::size_of::<WriteMemoryRequest>() as u32,
                        std::ptr::null_mut()
                    );
                    thread::sleep(Duration::from_millis(5));
                }
            } else {
                println!("[!] ОШИБКА: Паттерн сигнатуры ИИ не найден в памяти игры.");
            }
        } else {
            println!("[!] ОШИБКА: Не удалось открыть дескриптор чтения процесса игры.");
        }

        println!("\nНажмите Enter для выхода...");
        let mut buf = String::new();
        let _ = io::stdin().read_line(&mut buf);
    }
}
