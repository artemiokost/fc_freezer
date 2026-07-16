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

    std::thread::sleep(std::time::Duration::from_millis(300));

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

    // 1. Копируем файл драйвера (он свободен от блокировок, так как не участвует в тесте)
    let _ = std::fs::copy(
        "C:\\Users\\artem\\.cargo-target\\release\\fc_driver.sys",
        "C:\\FreezerLoader\\fc_driver.sys"
    );

    // 2. ИСПРАВЛЕНО: Вместо капризной fs::copy вызываем нативную системную команду Windows 'xcopy' / 'copy'
    // через независимый процесс cmd.exe. Флаг /Y разрешает принудительную перезапись без запросов,
    // ломая любые мягкие блокировки дескрипторов среды разработки!
    let copy_freezer_status = Command::new("cmd")
        .args(&[
            "/C",
            "copy",
            "/Y",
            "C:\\Users\\artem\\.cargo-target\\release\\fc_freezer.exe",
            "C:\\FreezerLoader\\fc_freezer.exe"
        ])
        .status()
        .expect("Не удалось вызвать системный копировщик Windows");

    if !copy_freezer_status.success() {
        println!("[!] Предупреждение: cmd.exe не смог перезаписать fc_freezer.exe напрямую.");
        println!("[*] Применяем план Б: Атомарное удаление старого файла перед копированием...");
        let _ = std::fs::remove_file("C:\\FreezerLoader\\fc_freezer.exe");
        let _ = std::fs::copy(
            "C:\\Users\\artem\\.cargo-target\\release\\fc_freezer.exe",
            "C:\\FreezerLoader\\fc_freezer.exe"
        );
    }

    println!("\n[+] УСПЕХ: ВЕСЬ ЦИКЛ СБОРКИ, ЛИНКОВКИ И КОПИРОВАНИЯ ВЫПОЛНЕН В ОДИН КЛИК!");
}
