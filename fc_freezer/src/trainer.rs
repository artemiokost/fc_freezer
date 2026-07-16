use std::{thread, time::Duration, fs, sync::{Arc, Mutex}, process::Command, env};
use fc_shared::{WriteMemoryRequest, OP_DISABLE_AI};
use crate::ProcessDriver;

#[link(name = "user32")]
unsafe extern "system" {
    fn SetWindowLongPtrW(hwnd: *mut core::ffi::c_void, index: i32, new_long: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct TrainerConfig {
    pub pattern_ai_freeze: Vec<i32>,
    pub pattern_network_data: Vec<i32>,
    pub spoofed_division: i32,
    pub spoofed_draft_round: i32,
    pub spoofed_wl_wins: i32,
    pub server_location_id: i32,
}

pub struct TrainerState {
    pub game_pid: u32,
    pub addr_ai: u64,
    pub addr_net: u64,
    pub mod_disable_ai: bool,
    pub mod_div_spoof: bool,
    pub mod_draft_round: bool,
    pub mod_wl_spoof: bool,
    pub mod_server_change: bool,
    pub mod_alttab_bypass: bool,
    pub log_message: String,
}

pub fn load_or_create_config() -> TrainerConfig {
    let config_path = "config.json";
    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(config) = serde_json::from_str::<TrainerConfig>(&content) { return config; }
    }
    let new_config = TrainerConfig {
        pattern_ai_freeze: vec![0x48, 0x8B, 0x05, -1, -1, -1, -1, 0x48, 0x8B, 0x88, 0x8B, 0x01],
        pattern_network_data: vec![0x8B, 0x05, -1, -1, -1, -1, 0x89, 0x88, -1, -1, 0x00, 0x00],
        spoofed_division: 1,
        spoofed_draft_round: 3,
        spoofed_wl_wins: 15,
        server_location_id: 14,
    };
    if let Ok(json) = serde_json::to_string_pretty(&new_config) { let _ = fs::write(config_path, json); }
    new_config
}

pub fn spawn_workers(state: Arc<Mutex<TrainerState>>, config: TrainerConfig) {
    // ПОЛНЫЙ АВТОМАТ: Динамически определяем текущую папку и запускаем маппер "рядом с собой"
    thread::spawn(move || {
        // Получаем полный путь к текущему fc_freezer.exe
        if let Ok(mut current_dir) = env::current_exe() {
            // Отрезаем имя fc_freezer.exe, оставляя только путь к папке (FreezerLoader)
            if current_dir.pop() {
                let mapper_path = current_dir.join("kdmapper.exe");
                let driver_path = current_dir.join("fc_driver.sys");

                // Проверяем, лежат ли файлы маппера и драйвера в этой же папке рядом
                if fs::metadata(&mapper_path).is_ok() && fs::metadata(&driver_path).is_ok() {
                    // Запускаем маппер из локальной директории
                    let _ = Command::new(mapper_path)
                        .arg(driver_path)
                        .status();
                }
            }
        }
    });

    // Поток 1: Глобальный асинхронный перехват Hotkeys прямо в игре
    let hotkey_state = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
            unsafe {
                use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
                use windows_sys::Win32::System::Diagnostics::Debug::Beep;

                if (GetAsyncKeyState(0x74) as u16 & 0x8000) != 0 {
                    let mut s = hotkey_state.lock().unwrap(); s.mod_disable_ai = !s.mod_disable_ai;
                    Beep(if s.mod_disable_ai { 800 } else { 400 }, 100);
                }
                if (GetAsyncKeyState(0x75) as u16 & 0x8000) != 0 {
                    let mut s = hotkey_state.lock().unwrap(); s.mod_div_spoof = !s.mod_div_spoof;
                    Beep(if s.mod_div_spoof { 850 } else { 425 }, 100);
                }
                if (GetAsyncKeyState(0x76) as u16 & 0x8000) != 0 {
                    let mut s = hotkey_state.lock().unwrap(); s.mod_draft_round = !s.mod_draft_round;
                    Beep(if s.mod_draft_round { 900 } else { 450 }, 100);
                }
                if (GetAsyncKeyState(0x77) as u16 & 0x8000) != 0 {
                    let mut s = hotkey_state.lock().unwrap(); s.mod_wl_spoof = !s.mod_wl_spoof;
                    Beep(if s.mod_wl_spoof { 950 } else { 475 }, 100);
                }
                if (GetAsyncKeyState(0x78) as u16 & 0x8000) != 0 {
                    let mut s = hotkey_state.lock().unwrap(); s.mod_server_change = !s.mod_server_change;
                    Beep(if s.mod_server_change { 1000 } else { 500 }, 100);
                }
                if (GetAsyncKeyState(0x79) as u16 & 0x8000) != 0 {
                    let mut s = hotkey_state.lock().unwrap(); s.mod_alttab_bypass = !s.mod_alttab_bypass;
                    Beep(if s.mod_alttab_bypass { 1100 } else { 550 }, 100);
                }
            }
        }
    });

    // Поток 2: Мониторинг памяти Frostbite и беспроводная запись Ring 0
    let worker_state = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(30));
            let mut s = worker_state.lock().unwrap();

            if s.game_pid == 0 {
                s.game_pid = unsafe { crate::get_process_id("FC26.exe") };
                continue;
            }
            if unsafe { crate::get_process_id("FC26.exe") } == 0 {
                s.game_pid = 0; s.addr_ai = 0; s.addr_net = 0;
                s.mod_disable_ai = false; s.mod_div_spoof = false; s.mod_draft_round = false;
                s.mod_wl_spoof = false; s.mod_server_change = false; s.mod_alttab_bypass = false;
                continue;
            }

            if s.addr_ai == 0 || s.addr_net == 0 {
                if let Some(driver) = unsafe { ProcessDriver::open(s.game_pid) } {
                    let bounds = unsafe { driver.module_bounds() };
                    if s.addr_ai == 0 { if let Some(a) = unsafe { driver.aob_scan(bounds.base_address, bounds.size_of_image, &config.pattern_ai_freeze) } { s.addr_ai = a as u64; } }
                    if s.addr_net == 0 { if let Some(a) = unsafe { driver.aob_scan(bounds.base_address, bounds.size_of_image, &config.pattern_network_data) } { s.addr_net = a as u64; } }
                    if s.addr_ai != 0 && s.addr_net != 0 { s.log_message = String::from("[+] АВТО-МАППИНГ ЗАВЕРШЕН. СЕТЕВЫЕ ИГРОВЫЕ СЕССИИ CHEATARMY АКТИВНЫ!"); }
                }
                continue;
            }

            unsafe {
                if s.mod_disable_ai {
                    let r = WriteMemoryRequest { process_id: s.game_pid, target_address: s.addr_ai, operation_id: OP_DISABLE_AI, i32_value: 0 };
                    SetWindowLongPtrW(&r as *const _ as *mut core::ffi::c_void, 0x777FFFFF, std::ptr::null_mut());
                }
            }
        }
    });
}
