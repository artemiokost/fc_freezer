use std::process::Command;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn MoveFileExW(lpExistingFileName: *const u16, lpNewFileName: *const u16, dwFlags: u32) -> i32;
}

#[test]
fn build_and_link_cheat_arm() {
    println!("=======================================================");
    println!("[*] ШАГ 1: Обход блокировок Windows (Смещение)...");
    println!("=======================================================");

    let _ = Command::new("taskkill").args(&["/F", "/IM", "fc_freezer.exe"]).status();
    std::thread::sleep(std::time::Duration::from_millis(300));

    let target_path = "C:\\FreezerLoader\\fc_freezer.exe";
    let backup_path = "C:\\FreezerLoader\\fc_freezer.old";
    let _ = std::fs::remove_file(backup_path);

    if std::fs::rename(target_path, backup_path).is_ok() {
        println!("[+] Блокировка Windows успешно обойдена. Старый файл смещен в .old");
        let target_utf16: Vec<u16> = "C:\\FreezerLoader\\fc_freezer.old\0".encode_utf16().collect();
        unsafe { MoveFileExW(target_utf16.as_ptr(), std::ptr::null(), 4); }
    }

    println!("\n=======================================================");
    println!("[*] ШАГ 2: Принудительная Сборка ФРИЗЕРА (Финальный EXE)...");
    println!("=======================================================");

    let freezer_status = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path", "Cargo.toml"])
        .status()
        .expect("Не удалось запустить команду cargo build");
    assert!(freezer_status.success());

    println!("\n=======================================================");
    println!("[*] ШАГ 3: Изолированная сборка Kernel-драйвера (no_std Ring 0)...");
    println!("=======================================================");

    let driver_status = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path", "../fc_driver/Cargo.toml"])
        .status()
        .expect("Не удалось запустить команду cargo build для fc_driver");
    assert!(driver_status.success());

    println!("\n=======================================================");
    println!("[*] ШАГ 4: Вызов оригинального линковщика Microsoft link.exe...");
    println!("=======================================================");

    // ПРЯМОЙ И СТАБИЛЬНЫЙ ВЫЗОВ: ntoskrnl.lib убрана из аргументов, линковка не упадет!
    let link_output = Command::new("C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\\link.exe")
        .args(&[
            "/DRIVER",
            "/SUBSYSTEM:NATIVE",
            "/ENTRY:DriverEntry",
            "/MACHINE:X64",
            "/NODEFAULTLIB",
            "/LIBPATH:C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Tools\\MSVC\\14.44.35207\\lib\\x64",
            "/OUT:C:\\Users\\artem\\.cargo-target\\release\\fc_driver.sys",
            "C:\\Users\\artem\\.cargo-target\\release\\fc_driver.lib"
        ])
        .output()
        .expect("Не удалось запустить системный link.exe");

    if !link_output.status.success() {
        let err_msg = String::from_utf8_lossy(&link_output.stderr);
        let out_msg = String::from_utf8_lossy(&link_output.stdout);
        println!("[!] КРИТИЧЕСКАЯ ОШИБКА MSVC LINK.EXE:\nSTDOUT:\n{}\nSTDERR:\n{}", out_msg, err_msg);
        panic!("Сбой на этапе линковки драйвера ядра!");
    }
    println!("[+] Системный файл ядра fc_driver.sys успешно слинкован.");

    println!("\n=======================================================");
    println!("[*] ШАГ 5: Принудительный перенос файлов в FreezerLoader...");
    println!("=======================================================");

    let _ = std::fs::copy("C:\\Users\\artem\\.cargo-target\\release\\fc_driver.sys", "C:\\FreezerLoader\\fc_driver.sys");
    let copy_freezer = std::fs::copy("C:\\Users\\artem\\.cargo-target\\release\\fc_freezer.exe", "C:\\FreezerLoader\\fc_freezer.exe");

    if let Err(e) = copy_freezer { panic!("[!] Ошибка переноса бинарника: {:?}", e); }
    println!("\n[+] УСПЕХ: ВЕСЬ ЦИКЛ СБОРКИ, ЛИНКОВКИ И КОПИРОВАНИЯ ВЫПОЛНЕН В ОДИН КЛИК!");
}
