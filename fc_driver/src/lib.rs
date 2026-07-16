#![no_std]
#![no_main]

use fc_shared::WriteMemoryRequest;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "system" fn _DllMainCRTStartup() -> i32 { 1 }

type NTSTATUS = i32;
const STATUS_SUCCESS: NTSTATUS = 0;

// Хранилище для оригинальной функции ядра, чтобы система не упала в BSOD
static mut ORIGINAL_FUNCTION_PTR: *mut core::ffi::c_void = core::ptr::null_mut();

/// Наша кастомная ядерная функция, которая перехватывает управление.
/// Она вызывается каждый раз, когда EXE-приложение отправляет специальный системный запрос.
unsafe extern "system" fn hooked_kernel_function(request_ptr: *mut core::ffi::c_void, magic_code: u32) -> NTSTATUS {
    // Проверяем секретный "магический код", чтобы драйвер реагировал только на НАШ сканер
    if magic_code == 0x777FFFFF && !request_ptr.is_null() {
        let request = &*(request_ptr as *const WriteMemoryRequest);

        // Прямая аппаратная запись в память Frostbite (EA Sports FC 26) из Ring 0
        // Ядро игнорирует любые защиты процессов античитом
        let target_ptr = request.target_address as *mut i32;
        if !target_ptr.is_null() {
            // Атомарно перезаписываем значение ИИ на 0 (заморозка ботов)
            core::ptr::write_volatile(target_ptr, request.value_to_write);
        }
        return STATUS_SUCCESS;
    }

    // Если запрос обычный — передаем управление родной функции Windows, система работает штатно
    type OriginalFuncType = unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> NTSTATUS;
    let original: OriginalFuncType = core::mem::transmute(ORIGINAL_FUNCTION_PTR);
    original(request_ptr, magic_code)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DriverEntry(_driver_object: *mut core::ffi::c_void, _registry_path: *mut core::ffi::c_void) -> NTSTATUS {
    // ТОП-ПРАКТИКА: Подменяем указатель неиспользуемой или редкой функции в ядре ОС.
    // Например, мы можем перехватить обработчик системного вызова NtUserSetWindowLongPtr или аналогичный.
    // Для демонстрации маппер выделяет стабильный вектор перехвата.

    // В реальном маппинге адрес функции ищется по сигнатуре в ntoskrnl.exe:
    let fake_kernel_table_ptr = 0xFFFFF8021FECA000 as *mut *mut core::ffi::c_void; // Системный адрес заглушки

    if !fake_kernel_table_ptr.is_null() {
        ORIGINAL_FUNCTION_PTR = *fake_kernel_table_ptr;
        *fake_kernel_table_ptr = hooked_kernel_function as *mut core::ffi::c_void;
    }

    STATUS_SUCCESS
}
