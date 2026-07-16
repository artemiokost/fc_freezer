#![no_std]
#![no_main]
#![allow(non_camel_case_types)]

use fc_shared::{WriteMemoryRequest, OP_PING, OP_DISABLE_AI, DRIVER_VERSION_CODE};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn _DllMainCRTStartup() -> i32 { 1 }

type NTSTATUS = i32;
const STATUS_SUCCESS: NTSTATUS = 0;

static mut ORIGINAL_FUNCTION_PTR: *mut core::ffi::c_void = core::ptr::null_mut();

unsafe extern "system" fn hooked_handler(request_ptr: *mut core::ffi::c_void, magic_code: u32) -> i32 {
    unsafe {
        if magic_code == 0x777FFFFF && !request_ptr.is_null() {
            let request = &*(request_ptr as *const WriteMemoryRequest);

            // Если пришел пинг — возвращаем жестко зашитую версию драйвера
            if request.operation_id == OP_PING {
                return DRIVER_VERSION_CODE;
            }

            if request.target_address != 0 && request.operation_id == OP_DISABLE_AI {
                let target_ptr = request.target_address as *mut i32;
                if !target_ptr.is_null() {
                    core::ptr::write_volatile(target_ptr, 0);
                }
            }
            return 1;
        }

        type OriginalFunc = unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> i32;
        let original: OriginalFunc = core::mem::transmute(ORIGINAL_FUNCTION_PTR);
        original(request_ptr, magic_code)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DriverEntry(_driver_object: *mut core::ffi::c_void, _registry_path: *mut core::ffi::c_void) -> NTSTATUS {
    let system_dispatch_table_ptr = 0xFFFFF8021FECA000 as *mut *mut core::ffi::c_void;
    unsafe {
        if !system_dispatch_table_ptr.is_null() {
            ORIGINAL_FUNCTION_PTR = *system_dispatch_table_ptr;
            *system_dispatch_table_ptr = hooked_handler as *mut core::ffi::c_void;
        }
    }
    STATUS_SUCCESS
}
