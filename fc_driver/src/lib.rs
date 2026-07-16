#![no_std]
#![no_main]
#![allow(non_camel_case_types)]

use fc_shared::WriteMemoryRequest;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }

#[unsafe(no_mangle)]
pub extern "system" fn _DllMainCRTStartup() -> i32 { 1 }

type NTSTATUS = i32;
const STATUS_SUCCESS: NTSTATUS = 0;
static mut ORIGINAL_FUNCTION_PTR: *mut core::ffi::c_void = core::ptr::null_mut();

unsafe extern "system" fn hooked_kernel_function(request_ptr: *mut core::ffi::c_void, magic_code: u32) -> NTSTATUS {
    if magic_code == 0x777FFFFF && !request_ptr.is_null() {
        let request = &*(request_ptr as *const WriteMemoryRequest);
        let target_ptr = request.target_address as *mut i32;
        if !target_ptr.is_null() {
            core::ptr::write_volatile(target_ptr, request.value_to_write);
        }
        return STATUS_SUCCESS;
    }
    type OriginalFuncType = unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> NTSTATUS;
    let original: OriginalFuncType = core::mem::transmute(ORIGINAL_FUNCTION_PTR);
    original(request_ptr, magic_code)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DriverEntry(_driver_object: *mut core::ffi::c_void, _registry_path: *mut core::ffi::c_void) -> NTSTATUS {
    let fake_kernel_table_ptr = 0xFFFFF8021FECA000 as *mut *mut core::ffi::c_void;
    if !fake_kernel_table_ptr.is_null() {
        ORIGINAL_FUNCTION_PTR = *fake_kernel_table_ptr;
        *fake_kernel_table_ptr = hooked_kernel_function as *mut core::ffi::c_void;
    }
    STATUS_SUCCESS
}
