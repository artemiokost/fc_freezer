use std::{thread, time::Duration, fs, sync::{Arc, Mutex}, process::{Command, Stdio}, env};
use fc_shared::{
    DRIVER_VERSION_CODE, OP_PING, OP_DISABLE_AI, OP_DIV_SPOOFER,
    OP_DRAFT_MODIFIER, OP_WL_WIN_SPOOFER, OP_SERVER_CHANGER, OP_ALTTAB_BYPASS, WriteMemoryRequest
};
use crate::{ProcessDriver, config::TrainerConfig};

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtSetInformationProcess(process_handle: *mut core::ffi::c_void, process_information_class: u32, process_information: *mut core::ffi::c_void, process_information_length: u32) -> i32;
}

pub struct TrainerState {
    pub driver_status_str: String,
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

fn send_kernel_request(req: &WriteMemoryRequest) -> i32 {
    unsafe {
        NtSetInformationProcess(
            core::ptr::null_mut(),
            0x777FFFFF,
            req as *const _ as *mut core::ffi::c_void,
            std::mem::size_of::<WriteMemoryRequest>() as u32
        )
    }
}

pub fn spawn_workers(state: Arc<Mutex<TrainerState>>, config: TrainerConfig) {
    let init_state = Arc::clone(&state);
    let cfg = config.clone();

    thread::spawn(move || {
        let req_ping = WriteMemoryRequest { process_id: 0, target_address: 0, operation_id: OP_PING, i32_value: 0 };
        let mut current_version = send_kernel_request(&req_ping);

        if current_version != DRIVER_VERSION_CODE {
            {
                let mut s = init_state.lock().unwrap();
                s.driver_status_str = String::from("Ядро не отвечает. Авто-маппинг...");
            }

            if let Ok(mut c_dir) = env::current_exe() {
                if c_dir.pop() {
                    let m_path = c_dir.join("kdmapper.exe");
                    let d_path = c_dir.join("fc_driver.sys");

                    if fs::metadata(&m_path).is_ok() && fs::metadata(&d_path).is_ok() {
                        // ИСПРАВЛЕНО: Полностью глушим логи kdmapper через Stdio::null(), чтобы не ломать TUI
                        let _ = Command::new(m_path)
                            .arg(d_path)
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status();
                        thread::sleep(Duration::from_millis(1000));
                    }
                }
            }
            current_version = send_kernel_request(&req_ping);
        }

        {
            let mut s = init_state.lock().unwrap();
            if current_version == DRIVER_VERSION_CODE {
                s.driver_status_str = format!("v{:.2} [ СВЯЗЬ ЯДРА ОК ]", (current_version as f32) / 100.0);
            } else if current_version > 0 {
                s.driver_status_str = format!("v{:.2} [ТРЕБУЕТСЯ ПЕРЕЗАГРУЗКА ПК]", (current_version as f32) / 100.0);
            } else {
                s.driver_status_str = String::from("ОШИБКА ЗАГРУЗКИ. Проверьте BIOS / HVCI");
            }
        }

        loop {
            thread::sleep(Duration::from_millis(30));
            let mut s = init_state.lock().unwrap();

            unsafe {
                use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
                use windows_sys::Win32::System::Diagnostics::Debug::Beep;
                if (GetAsyncKeyState(0x74) as u16 & 0x8000) != 0 { s.mod_disable_ai = !s.mod_disable_ai; Beep(if s.mod_disable_ai { 800 } else { 400 }, 100); }
                if (GetAsyncKeyState(0x75) as u16 & 0x8000) != 0 { s.mod_div_spoof = !s.mod_div_spoof; Beep(if s.mod_div_spoof { 850 } else { 425 }, 100); }
                if (GetAsyncKeyState(0x76) as u16 & 0x8000) != 0 { s.mod_draft_round = !s.mod_draft_round; Beep(if s.mod_draft_round { 900 } else { 450 }, 100); }
                if (GetAsyncKeyState(0x77) as u16 & 0x8000) != 0 { s.mod_wl_spoof = !s.mod_wl_spoof; Beep(if s.mod_wl_spoof { 950 } else { 475 }, 100); }
                if (GetAsyncKeyState(0x78) as u16 & 0x8000) != 0 { s.mod_server_change = !s.mod_server_change; Beep(if s.mod_server_change { 1000 } else { 500 }, 100); }
                if (GetAsyncKeyState(0x79) as u16 & 0x8000) != 0 { s.mod_alttab_bypass = !s.mod_alttab_bypass; Beep(if s.mod_alttab_bypass { 1100 } else { 550 }, 100); }
            }

            if s.game_pid == 0 { s.game_pid = unsafe { crate::get_process_id("FC26.exe") }; continue; }
            if unsafe { crate::get_process_id("FC26.exe") } == 0 { s.game_pid = 0; s.addr_ai = 0; s.addr_net = 0; s.mod_disable_ai = false; s.mod_div_spoof = false; s.mod_draft_round = false; s.mod_wl_spoof = false; s.mod_server_change = false; s.mod_alttab_bypass = false; continue; }

            if s.addr_ai == 0 || s.addr_net == 0 {
                if let Some(driver) = unsafe { ProcessDriver::open(s.game_pid) } {
                    let bounds = unsafe { driver.module_bounds() };
                    if s.addr_ai == 0 { if let Some(a) = unsafe { driver.aob_scan(bounds.base_address, bounds.size_of_image, &cfg.pattern_ai_freeze) } { s.addr_ai = a as u64; } }
                    if s.addr_net == 0 { if let Some(a) = unsafe { driver.aob_scan(bounds.base_address, bounds.size_of_image, &cfg.pattern_network_data) } { s.addr_net = a as u64; } }
                    if s.addr_ai != 0 && s.addr_net != 0 { s.log_message = String::from("[+] СЕТЕВЫЕ И ИГРОВЫЕ ДЕСКРИПТОРЫ CHEATARMY УСПЕШНО ИНИЦИАЛИЗИРОВАНЫ!"); }
                }
                continue;
            }

            if s.mod_disable_ai { let r = WriteMemoryRequest { process_id: s.game_pid, target_address: s.addr_ai, operation_id: OP_DISABLE_AI, i32_value: 0 }; send_kernel_request(&r); }
            if s.mod_div_spoof { let r = WriteMemoryRequest { process_id: s.game_pid, target_address: s.addr_net + 0x10, operation_id: OP_DIV_SPOOFER, i32_value: cfg.spoofed_division }; send_kernel_request(&r); }
            if s.mod_draft_round { let r = WriteMemoryRequest { process_id: s.game_pid, target_address: s.addr_net + 0x20, operation_id: OP_DRAFT_MODIFIER, i32_value: cfg.spoofed_draft_round }; send_kernel_request(&r); }
            if s.mod_wl_spoof { let r = WriteMemoryRequest { process_id: s.game_pid, target_address: s.addr_net + 0x30, operation_id: OP_WL_WIN_SPOOFER, i32_value: cfg.spoofed_wl_wins }; send_kernel_request(&r); }
            if s.mod_server_change { let r = WriteMemoryRequest { process_id: s.game_pid, target_address: s.addr_net + 0x40, operation_id: OP_SERVER_CHANGER, i32_value: cfg.server_location_id }; send_kernel_request(&r); }
            if s.mod_alttab_bypass { let r = WriteMemoryRequest { process_id: s.game_pid, target_address: s.addr_net + 0x50, operation_id: OP_ALTTAB_BYPASS, i32_value: 1 }; send_kernel_request(&r); }
        }
    });
}
