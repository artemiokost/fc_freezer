#![no_std]
#![no_main]
#![allow(non_camel_case_types)]

use fc_shared::{
    WriteMemoryRequest, DRIVER_VERSION_CODE, OP_PING, OP_DISABLE_AI,
    OP_DIV_SPOOFER, OP_DRAFT_MODIFIER, OP_WL_WIN_SPOOFER, OP_SERVER_CHANGER, OP_ALTTAB_BYPASS
};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn _DllMainCRTStartup() -> i32 { 1 }

type NTSTATUS = i32;
const STATUS_SUCCESS: NTSTATUS = 0;

static mut ORIGINAL_FUNCTION_PTR: *mut core::ffi::c_void = core::ptr::null_mut();

#[link(name = "ntoskrnl", kind = "raw-dylib")]
unsafe extern "system" {
    fn ObReferenceObjectByName(obj_name: *mut core::ffi::c_void, attrs: u32, access: *mut core::ffi::c_void, desired: u32, obj_type: *mut core::ffi::c_void, mode: i8, context: *mut core::ffi::c_void, object: *mut *mut core::ffi::c_void) -> NTSTATUS;
}

unsafe extern "system" fn hooked_handler(process_handle: *mut core::ffi::c_void, process_information_class: u32, process_information: *mut core::ffi::c_void, process_information_length: u32) -> NTSTATUS {
    unsafe {
        if process_information_class == 0x777FFFFF && !process_information.is_null() {
            let request = &*(process_information as *const WriteMemoryRequest);

            if request.operation_id == OP_PING {
                return DRIVER_VERSION_CODE;
            } else if request.target_address != 0 {
                if request.operation_id == OP_DISABLE_AI {
                    let ptr = request.target_address as *mut i32;
                    core::ptr::write_volatile(ptr, 0);
                } else if request.operation_id == OP_DIV_SPOOFER || request.operation_id == OP_DRAFT_MODIFIER || request.operation_id == OP_WL_WIN_SPOOFER || request.operation_id == OP_ALTTAB_BYPASS {
                    let ptr = request.target_address as *mut i32;
                    core::ptr::write_volatile(ptr, request.i32_value);
                } else if request.operation_id == OP_SERVER_CHANGER {
                    let ptr = request.target_address as *mut u32;
                    core::ptr::write_volatile(ptr, request.i32_value as u32);
                }
            }
            return STATUS_SUCCESS;
        }

        type OriginalFunc = unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut core::ffi::c_void, u32) -> NTSTATUS;
        let original: OriginalFunc = core::mem::transmute(ORIGINAL_FUNCTION_PTR);
        original(process_handle, process_information_class, process_information, process_information_length)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DriverEntry(mut _driver_object: *mut core::ffi::c_void, _registry_path: *mut core::ffi::c_void) -> NTSTATUS {
    unsafe {
        if _driver_object.is_null() {
            let mut base_driver: *mut core::ffi::c_void = core::ptr::null_mut();
            let mut name_bytes: [u16; 16] = [0x5C, 0x44, 0x72, 0x69, 0x76, 0x65, 0x5C, 0x50, 0x6E, 0x70, 0x4D, 0x61, 0x6E, 0x61, 0x67, 0x65];
            ObReferenceObjectByName(&mut name_bytes as *mut _ as *mut core::ffi::c_void, 0, core::ptr::null_mut(), 0, core::ptr::null_mut(), 0, core::ptr::null_mut(), &mut base_driver);
            if !base_driver.is_null() { _driver_object = base_driver; }
        }

        let system_dispatch_table_ptr = 0xFFFFF8021FECA000 as *mut *mut core::ffi::c_void;
        if !system_dispatch_table_ptr.is_null() {
            ORIGINAL_FUNCTION_PTR = *system_dispatch_table_ptr;
            *system_dispatch_table_ptr = hooked_handler as *mut core::ffi::c_void;
        }
    }
    STATUS_SUCCESS
}
