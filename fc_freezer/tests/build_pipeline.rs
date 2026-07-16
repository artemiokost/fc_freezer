use std::process::Command;

#[test]
fn build_and_link_cheat_arm() {
    println!("=======================================================");
    println!("[*] ШАГ 1: Зачистка фоновых процессов Windows...");
    println!("=======================================================");

    // Принудительно закрываем старый зависший процесс трейнера в FreezerLoader, если он запущен
    let _ = Command::new("taskkill")
        .args(&["/F", "/IM", "fc_freezer.exe"])
        .status();

    // Даем операционной системе Windows 200 миллисекунд, чтобы полностью освободить дескрипторы файла на диске
    std::thread::sleep(std::time::Duration::from_millis(200));

    println!("\n=======================================================");
    println!("[*] ШАГ 2: Запуск автоматической сборки воркспейса Cargo...");
    println!("=======================================================");

    // Компилируем весь воркспейс в режиме релиза
    let cargo_status = Command::new("cargo")
        .args(&["build", "--release"])
        .status()
        .expect("Не удалось запустить команду cargo build");
    assert!(cargo_status.success(), "Ошибка компиляции проекта Cargo");

    println!("\n=======================================================");
    println!("[*] ШАГ 3: Вызов оригинального линковщика Microsoft link.exe...");
    println!("=======================================================");

    // Вызываем линковщик MSVC для сборки системного файла ядра из кастомной папки Artem
    // ИСПРАВЛЕНО: Ключ /WHOLEARCHIVE полностью убран. Строка приведена к стандартному синтаксису Windows MSVC link.exe
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
    println!("[*] ШАГ 4: Принудительный перенос файлов в FreezerLoader...");
    println!("=======================================================");

    // Копируем скомпилированные и слинкованные бинарники в пусковую папку
    let copy_driver = std::fs::copy(
        "C:\\Users\\artem\\.cargo-target\\release\\fc_driver.sys",
        "C:\\FreezerLoader\\fc_driver.sys"
    );

    let copy_freezer = std::fs::copy(
        "C:\\Users\\artem\\.cargo-target\\release\\fc_freezer.exe",
        "C:\\FreezerLoader\\fc_freezer.exe"
    );

    if copy_driver.is_err() {
        println!("[!] Предупреждение: Не удалось обновить fc_driver.sys (возможно, старый хук еще удерживается ядром. Требуется перезагрузка ПК)");
    }

    if let Err(e) = copy_freezer {
        panic!("[!] КРИТИЧЕСКАЯ ОШИБКА ОС: Не удалось скопировать fc_freezer.exe. Причина: {:?}", e);
    }

    println!("\n[+] УСПЕХ: ВЕСЬ ЦИКЛ СБОРКИ, ЛИНКОВКИ И КОПИРОВАНИЯ ВЫПОЛНЕН В ОДИН КЛИК!");
}
