#![no_std]

// Идентификаторы расширенных операций в стиле CheatArmy
pub const OP_DISABLE_AI: u32 = 1;
pub const OP_DIV_SPOOFER: u32 = 2;
pub const OP_DRAFT_MODIFIER: u32 = 3;
pub const OP_WL_WIN_SPOOFER: u32 = 4;
pub const OP_SERVER_CHANGER: u32 = 5;
pub const OP_ALTTAB_BYPASS: u32 = 6;

#[repr(C)]
pub struct WriteMemoryRequest {
    pub process_id: u32,
    pub target_address: u64,
    pub operation_id: u32,
    pub i32_value: i32,        // Для раундов, дивизионов и побед
}
