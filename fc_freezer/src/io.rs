use fc_shared::{WriteMemoryRequest, IOCTL_WRITE_MEMORY, OP_PING};
use windows_sys::Win32::Foundation::{INVALID_HANDLE_VALUE, HANDLE};
use windows_sys::Win32::System::IO::DeviceIoControl;

pub fn get_driver_version_ioctl(handle: HANDLE) -> i32 {
    if handle == INVALID_HANDLE_VALUE { return 0; }
    let r = WriteMemoryRequest { process_id: 0, target_address: 0, operation_id: OP_PING, i32_value: 0 };
    let mut version_returned = 0i32;
    let mut bytes_returned = 0u32;
    unsafe {
        DeviceIoControl(
            handle, IOCTL_WRITE_MEMORY, &r as *const _ as *const _,
            std::mem::size_of::<WriteMemoryRequest>() as u32,
            &mut version_returned as *mut _ as *mut _,
            core::mem::size_of::<i32>() as u32, &mut bytes_returned, std::ptr::null_mut()
        );
    }
    version_returned
}

pub fn send_ioctl_cmd(handle: HANDLE, pid: u32, address: u64, op_id: u32, val: i32) {
    if handle == INVALID_HANDLE_VALUE || address == 0 { return; }
    let r = WriteMemoryRequest { process_id: pid, target_address: address, operation_id: op_id, i32_value: val };
    let mut bytes_ret = 0;
    unsafe {
        DeviceIoControl(
            handle, IOCTL_WRITE_MEMORY, &r as *const _ as *const _,
            std::mem::size_of::<WriteMemoryRequest>() as u32,
            std::ptr::null_mut(), 0, &mut bytes_ret, std::ptr::null_mut()
        );
    }
}
