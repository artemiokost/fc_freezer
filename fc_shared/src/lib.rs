#![no_std]

#[repr(C)]
pub struct WriteMemoryRequest {
    pub process_id: u32,
    pub target_address: u64,
    pub value_to_write: i32,
}
