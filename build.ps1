Write-Host "[*] Шаг 1: Компиляция воркспейса Cargo..." -ForegroundColor Cyan
cargo build --release

Write-Host "[*] Шаг 2: Линковка Kernel-драйвера через MSVC link.exe..." -ForegroundColor Cyan
& "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\link.exe" /DRIVER /SUBSYSTEM:NATIVE /ENTRY:DriverEntry /NODEFAULTLIB /OUT:C:\Users\artem\.cargo-target\release\fc_driver.sys C:\Users\artem\.cargo-target\release\fc_driver.lib

Write-Host "[*] Шаг 3: Перенос готового софта в папку FreezerLoader..." -ForegroundColor Cyan
Copy-Item C:\Users\artem\.cargo-target\release\fc_driver.sys, C:\Users\artem\.cargo-target\release\fc_freezer.exe -Destination C:\FreezerLoader\ -Force

Write-Host "`n[+] СБОРКА И ЛИНКОВКА ЗАВЕРШЕНЫ УСПЕШНО! КОМПЛЕКС ГОТОВ К ИГРЕ.`n" -ForegroundColor Green
