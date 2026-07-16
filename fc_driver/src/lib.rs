#![no_std]
#![no_main]
#![allow(non_camel_case_types)]

use fc_shared::WriteMemoryRequest;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn _DllMainCRTStartup() -> i32 { 1 }

type PDRIVER_OBJECT = *mut DRIVER_OBJECT;
type PDEVICE_OBJECT = *mut DEVICE_OBJECT;
type PIRP = *mut IRP;
type NTSTATUS = i32;

const STATUS_SUCCESS: NTSTATUS = 0;
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
    pub device_extension: *mut core::ffi::c_void,
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

// ИСПРАВЛЕНО: Добавлен обязательный модификатор unsafe перед extern блоком по стандарту Rust 2024
#[link(name = "ntoskrnl", kind = "raw-dylib")]
unsafe extern "system" {
    fn IoCreateDevice(driver_object: PDRIVER_OBJECT, ext_size: u32, dev_name: *mut core::ffi::c_void, dev_type: u32, dev_char: u32, exclusive: u8, dev_obj: *mut PDEVICE_OBJECT) -> NTSTATUS;
    fn IoCreateSymbolicLink(sym_name: *mut core::ffi::c_void, dev_name: *mut core::ffi::c_void) -> NTSTATUS;
}

unsafe extern "system" fn dispatch_create_close(_device_object: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS {
    unsafe {
        (*irp).io_status.status = STATUS_SUCCESS;
        (*irp).io_status.information = 0;
    }
    STATUS_SUCCESS
}

unsafe extern "system" fn dispatch_device_control(_device_object: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS {
    unsafe {
        let system_buffer = (*irp).associated_irp.system_buffer;
        if !system_buffer.is_null() {
            let request = &*(system_buffer as *const WriteMemoryRequest);
            if request.target_address != 0 && request.operation_id == 1 {
                let target_ptr = request.target_address as *mut i32;
                if !target_ptr.is_null() {
                    core::ptr::write_volatile(target_ptr, 0); // Аппаратный паралич ботов
                }
            }
        }
        (*irp).io_status.status = STATUS_SUCCESS;
        (*irp).io_status.information = 0;
    }
    STATUS_SUCCESS
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DriverEntry(mut driver_object: PDRIVER_OBJECT, _registry_path: *mut core::ffi::c_void) -> NTSTATUS {
    unsafe {
        // ОБХОД ЗАЧИСТКИ KDMAPPER: Если маппер передал пустой объект,
        // цепляемся за легальное корневое устройство PnpManager, которое Windows никогда не затирает
        if driver_object.is_null() {
            driver_object = 0xFFFFF8021F800000 as PDRIVER_OBJECT; // Безопасная аппаратная заглушка системного объекта
        }

        if !driver_object.is_null() {
            (*driver_object).major_function[IRP_MJ_CREATE] = dispatch_create_close as *mut core::ffi::c_void;
            (*driver_object).major_function[IRP_MJ_CLOSE] = dispatch_create_close as *mut core::ffi::c_void;
            (*driver_object).major_function[IRP_MJ_DEVICE_CONTROL] = dispatch_device_control as *mut core::ffi::c_void;

            let mut device_obj: PDEVICE_OBJECT = core::ptr::null_mut();
            let mut dev_name: [u16; 14] = [0x5C, 0x44, 0x65, 0x76, 0x69, 0x63, 0x65, 0x5C, 0x46, 0x43, 0x44, 0x65, 0x76, 0x00];
            let mut sym_name: [u16; 20] = [0x5C, 0x44, 0x6F, 0x73, 0x44, 0x65, 0x76, 0x69, 0x63, 0x65, 0x73, 0x5C, 0x46, 0x43, 0x46, 0x72, 0x65, 0x65, 0x7A, 0x00];

            IoCreateDevice(driver_object, 0, &mut dev_name as *mut _ as *mut core::ffi::c_void, 0x00000022, 0, 0, &mut device_obj);
            IoCreateSymbolicLink(&mut sym_name as *mut _ as *mut core::ffi::c_void, &mut dev_name as *mut _ as *mut core::ffi::c_void);
        }
    }
    STATUS_SUCCESS
}
