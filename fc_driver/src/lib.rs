#![no_std]
#![no_main]

use fc_shared::{WriteMemoryRequest, IOCTL_WRITE_MEMORY};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "system" fn _DllMainCRTStartup() -> i32 { 1 }

// Объявляем необходимые типы данных ядра Windows
type PDRIVER_OBJECT = *mut DRIVER_OBJECT;
type PDEVICE_OBJECT = *mut DEVICE_OBJECT;
type PIRP = *mut IRP;
type NTSTATUS = i32;

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_UNSUCCESSFUL: NTSTATUS = -1073741823;

// Коды диспетчеризации Windows IRP
const IRP_MJ_CREATE: usize = 0;
const IRP_MJ_CLOSE: usize = 2;
const IRP_MJ_DEVICE_CONTROL: usize = 14;

#[repr(C)]
pub struct DRIVER_OBJECT {
    pub major_function: [*mut core::ffi::c_void; 28],
    pub driver_unload: *mut core::ffi::c_void,
}

#[repr(C)]
pub struct DEVICE_OBJECT {
    pub next_device: *mut DEVICE_OBJECT,
}

#[repr(C)]
pub struct IRP {
    pub io_status: IO_STATUS_BLOCK,
    pub associated_irp: ASSOCIATED_IRP,
}

#[repr(C)]
pub struct IO_STATUS_BLOCK {
    pub status: NTSTATUS,
    pub information: usize,
}

#[repr(C)]
pub struct ASSOCIATED_IRP {
    pub system_buffer: *mut core::ffi::c_void,
}

// Заглушка для легального открытия и закрытия канала из EXE (CreateFile / CloseHandle)
unsafe extern "system" fn dispatch_create_close(_device_object: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS {
    unsafe {
        (*irp).io_status.status = STATUS_SUCCESS;
        (*irp).io_status.information = 0;
        // В реальном ядре здесь вызывается IoCompleteRequest
    }
    STATUS_SUCCESS
}

// Главный обработчик команд (DeviceIoControl)
unsafe extern "system" fn dispatch_device_control(_device_object: PDEVICE_OBJECT, irp: PIRP, io_stack_location: *mut core::ffi::c_void) -> NTSTATUS {
    unsafe {
        let system_buffer = (*irp).associated_irp.system_buffer;

        // В реальном ядре мы извлекаем Текущий стек ввода-вывода (IoGetCurrentIrpStackLocation)
        // Чтобы узнать код пришедшей команды (Parameters.DeviceIoControl.IoControlCode)
        let ioctl_code = IOCTL_WRITE_MEMORY; // Для простоты привязываемся к нашей команде

        if ioctl_code == IOCTL_WRITE_MEMORY && !system_buffer.is_null() {
            let request = &*(system_buffer as *const WriteMemoryRequest);

            // ТОП-ПРАКТИКА: Прямой вызов функции копирования памяти ядра ОС.
            // Ядро находит структуры процесса по request.process_id, временно
            // отключает аппаратную защиту страниц памяти процессора (флаг WP в регистре CR0),
            // производит запись request.value_to_write по адресу request.target_address
            // и мгновенно возвращает защиту назад. Античит ничего не замечает.

            let _status = kernel_write_memory(request.process_id, request.target_address, request.value_to_write);
        }

        (*irp).io_status.status = STATUS_SUCCESS;
        (*irp).io_status.information = 0;
    }
    STATUS_SUCCESS
}

// Имитация ядерной функции MmCopyVirtualMemory / Прямой аппаратной записи
unsafe fn kernel_write_memory(_pid: u32, _address: u64, _value: i32) -> NTSTATUS {
    // Атомарная запись в физическую страницу памяти RAM
    STATUS_SUCCESS
}

/// Точка входа в ядро системы. Вызывается маппером (kdmapper).
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(driver_object: PDRIVER_OBJECT, _registry_path: *mut core::ffi::c_void) -> NTSTATUS {
    if driver_object.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    unsafe {
        // Регистрируем функции обработки вызовов для Windows
        (*driver_object).major_function[IRP_MJ_CREATE] = dispatch_create_close as *mut core::ffi::c_void;
        (*driver_object).major_function[IRP_MJ_CLOSE] = dispatch_create_close as *mut core::ffi::c_void;
        (*driver_object).major_function[IRP_MJ_DEVICE_CONTROL] = dispatch_device_control as *mut core::ffi::c_void;
    }

    STATUS_SUCCESS
}
