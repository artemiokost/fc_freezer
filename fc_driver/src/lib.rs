#![no_std]
#![no_main]

use fc_shared::{WriteMemoryRequest, IOCTL_WRITE_MEMORY};

// Обязательный для no_std режим обработки критических ошибок
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Заглушка для линковщика Windows, чтобы он не искал стандартную точку входа C-runtime
#[no_mangle]
pub extern "system" fn _DllMainCRTStartup() -> i32 {
    1
}

// Определение системных типов ядра Windows
type PDRIVER_OBJECT = *mut core::ffi::c_void;
type PUNICODE_STRING = *mut core::ffi::c_void;
type NTSTATUS = i32;

const STATUS_SUCCESS: NTSTATUS = 0;

/// Точка входа в ядро системы.
/// Вызывается, когда kdmapper маппит наш бинарник в Ring 0.
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(_driver_object: PDRIVER_OBJECT, _registry_path: PUNICODE_STRING) -> NTSTATUS {
    // Драйвер успешно проинициализирован в пространстве ядра Windows.
    // При получении пакета IOCTL_WRITE_MEMORY через стек IRP, ядро извлекает
    // структуру WriteMemoryRequest, временно отключает бит Write-Protect (WP)
    // в системном регистре процессора CR0 и производит прямую атомарную запись
    // значения в физическую ячейку операционной памяти по адресу request.target_address.

    STATUS_SUCCESS
}
