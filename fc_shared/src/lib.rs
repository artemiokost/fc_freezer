#![no_std]

pub const IOCTL_WRITE_MEMORY: u32 = 0x80002000;

// Имя Kernel-устройства в формате сырого слайса UTF-16, завершающегося нулем (\0)
pub const DEVICE_NAME_UTF16: &[u16] = &[
    0x005C, 0x005C, 0x002E, 0x005C, // \\.\
    0x0046, 0x0043, 0x0046, 0x0072, 0x0065, 0x0065, 0x007A, 0x0065, 0x0072, // FCFreezer
    0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0000 // Device\0
];

#[repr(C)]
pub struct WriteMemoryRequest {
    pub process_id: u32,
    pub target_address: u64,
    pub value_to_write: i32,
}
