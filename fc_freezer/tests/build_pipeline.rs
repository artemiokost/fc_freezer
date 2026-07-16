use std::process::Command;

#[test]
fn build_and_link_cheat_arm() {
    println!("=======================================================");
    println!("[*] ШАГ 1: Зачистка фоновых процессов Windows...");
    println!("=======================================================");

    // Принудительно закрываем старый зависший процесс трейнера в FreezerLoader
    let _ = Command::new("taskkill")
        .args(&["/F", "/IM", "fc_freezer.exe"])
        .status();

    std::thread::sleep(std::time::Duration::from_millis(200));

    println!("\n=======================================================");
    println!("[*] ШАГ 2: Автоматическая сборка графического воркспейса Cargo...");
    println!("=======================================================");

    // Компилируем воркспейс (fc_shared, fc_freezer) одной командой в релиз
    let cargo_status = Command::new("cargo")
        .args(&["build", "--release"])
        .status()
        .expect("Не удалось запустить команду cargo build");
    assert!(cargo_status.success(), "Ошибка компиляции воркспейса Cargo");

    println!("\n=======================================================");
    println!("[*] ШАГ 3: Изолированная сборка Kernel-драйвера (no_std Ring 0)...");
    println!("=======================================================");

    // Принудительно собираем драйвер, полностью минуя тестовый профиль std
    let driver_status = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path", "../fc_driver/Cargo.toml"])
        .status()
        .expect("Не удалось запустить команду cargo build для fc_driver");
    assert!(driver_status.success(), "Ошибка компиляции драйвера ядра");

    println!("\n=======================================================");
    println!("[*] ШАГ 4: Вызов оригинального линковщика Microsoft link.exe...");
    println!("=======================================================");

    // Вызываем линковщик MSVC для сборки системного файла ядра из кастомной папки Artem
    let link_status = Command::new("C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\\link.exe")
        .args(&[
            "/DRIVER",
            "/SUBSYSTEM:NATIVE",
            "/ENTRY:DriverEntry",
            "/MACHINE:X64",
            "/NODEFAULTLIB",
            "/OUT:C:\\Users\\artem\\.cargo-target\\release\\fc_driver.sys",
            "C:\\Users\\artem\\.cargo-target\\release\\fc_driver.lib"
        ])
        .status()
        .expect("Не удалось запустить системный link.exe");
    assert!(link_status.success(), "Ошибка на этапе линковки драйвера ядра");

    println!("\n=======================================================");
    println!("[*] ШАГ 5: Принудительный перенос файлов в FreezerLoader...");
    println!("=======================================================");

    // Копируем скомпилированные бинарники в пусковую папку
    let copy_driver = std::fs::copy(
        "C:\\Users\\artem\\.cargo-target\\release\\fc_driver.sys",
        "C:\\FreezerLoader\\fc_driver.sys"
    );

    let copy_freezer = std::fs::copy(
        "C:\\Users\\artem\\.cargo-target\\release\\fc_freezer.exe",
        "C:\\FreezerLoader\\fc_freezer.exe"
    );

    if copy_driver.is_err() {
        println!("[!] Предупреждение: Не удалось обновить fc_driver.sys (возможно, старый хук удерживается ядром)");
    }

    if let Err(e) = copy_freezer {
        panic!("[!] КРИТИЧЕСКАЯ ОШИБКА ОС: Не удалось скопировать fc_freezer.exe. Причина: {:?}", e);
    }

    println!("\n[+] УСПЕХ: ВЕСЬ ЦИКЛ СБОРКИ, ЛИНКОВКИ И КОПИРОВАНИЯ ВЫПОЛНЕН В ОДИН КЛИК!");
}
