use std::{thread, time::Duration, io, fs, sync::{Arc, Mutex}};
use fc_shared::WriteMemoryRequest;
use fc_freezer::{get_process_id, ProcessDriver};
use serde::{Serialize, Deserialize};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtQuerySystemInformation(class: u32, info: *mut core::ffi::c_void, len: u32, ret_len: *mut u32) -> i32;
}

// Изменено на i32, чтобы полностью соответствовать требованиям функции aob_scan
#[derive(Serialize, Deserialize, Clone)]
struct TrainerConfig {
    pattern_bytes: Vec<i32>,
}

struct TrainerState {
    is_active: bool,
    game_pid: u32,
    target_address: u64,
    log_message: String,
    current_pattern_str: String,
}

fn load_or_create_config() -> TrainerConfig {
    let config_path = "config.json";
    let default_pattern = vec![0x48, 0x8B, 0x05, -1, -1, -1, -1, 0x48, 0x8B, 0x88, 0x8B, 0x01];

    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(config) = serde_json::from_str::<TrainerConfig>(&content) {
            return config;
        }
    }

    let new_config = TrainerConfig { pattern_bytes: default_pattern };
    if let Ok(json) = serde_json::to_string_pretty(&new_config) {
        let _ = fs::write(config_path, json);
    }
    new_config
}

fn main() -> Result<(), io::Error> {
    let config = load_or_create_config();

    let pattern_preview: String = config.pattern_bytes.iter()
        .map(|&b| if b == -1 { "??".to_string() } else { format!("{:02X}", b) })
        .collect::<Vec<String>>().join(" ");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let state = Arc::new(Mutex::new(TrainerState {
        is_active: false,
        game_pid: 0,
        target_address: 0,
        log_message: String::from("[*] Трейнер запущен. Ожидание процесса игры..."),
        current_pattern_str: pattern_preview,
    }));

    let hotkey_state = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
            unsafe {
                let key_state = windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(0x74);
                if (key_state as u16 & 0x8000) != 0 {
                    let mut s = hotkey_state.lock().unwrap();
                    if s.target_address != 0 {
                        s.is_active = !s.is_active;
                        let freq = if s.is_active { 800 } else { 400 };
                        windows_sys::Win32::System::Diagnostics::Debug::Beep(freq, 150);

                        s.log_message = if s.is_active {
                            String::from("[*] Горячая клавиша F5: Заморозка ИИ АКТИВИРОВАНА.")
                        } else {
                            String::from("[*] Горячая клавиша F5: Заморозка ИИ ОТКЛЮЧЕНА.")
                        };
                    }
                }
            }
        }
    });

    let worker_state = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(50));
            let mut s = worker_state.lock().unwrap();

            if s.game_pid == 0 {
                s.game_pid = unsafe { get_process_id("FC26.exe") };
                if s.game_pid != 0 {
                    s.log_message = format!("[+] Игра поймана! PID: {}. Ищем ИИ на поле...", s.game_pid);
                }
                continue;
            }

            if unsafe { get_process_id("FC26.exe") } == 0 {
                s.game_pid = 0;
                s.target_address = 0;
                s.is_active = false;
                s.log_message = String::from("[*] Игра закрыта. Переход в режим ожидания...");
                continue;
            }

            if s.target_address == 0 {
                if let Some(driver) = unsafe { ProcessDriver::open(s.game_pid) } {
                    let bounds = unsafe { driver.module_bounds() };

                    // ИСПРАВЛЕНО: Передаем чистый срез &[i32] напрямую из нашего вектора конфигурации
                    if let Some(addr) = unsafe { driver.aob_scan(bounds.base_address, bounds.size_of_image, &config.pattern_bytes) } {
                        s.target_address = addr as u64;
                        s.log_message = format!("[+] Матч начался! Сигнатура зафиксирована: 0x{:X}", s.target_address);
                    }
                }
                continue;
            }

            if s.is_active && s.target_address != 0 {
                let request = WriteMemoryRequest {
                    process_id: s.game_pid,
                    target_address: s.target_address,
                    value_to_write: 0,
                };
                unsafe {
                    NtQuerySystemInformation(
                        0x777FFFFF,
                        &request as *const _ as *mut core::ffi::c_void,
                        std::mem::size_of::<WriteMemoryRequest>() as u32,
                        std::ptr::null_mut()
                    );
                }
            }
        }
    });

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(3)].as_ref())
                .split(size);

            let s = state.lock().unwrap();

            let header = Paragraph::new(" EA SPORTS FC 26 - ADVANCED RATATUI TRAINER v4.0 ")
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));

            // ИСПРАВЛЕНО: Передаем конкретный прямоугольник chunks[0] вместо всего массива геометрии
            f.render_widget(header, chunks[0]);

            let status_color = if s.is_active { Color::Green } else { Color::Red };
            let status_text = if s.is_active { "АКТИВЕН (Боты парализованы)" } else { "ВЫКЛЮЧЕН (Боты двигаются свободно)" };

            let mut info_string = format!(
                " Статус трейнера: {}\n\n Процесс игры:    {}\n Текущий паттерн:  [ {} ]\n Адрес ИИ кода:   ",
                status_text,
                if s.game_pid == 0 { String::from("Ожидание запуска FC26.exe...") } else { format!("FC26.exe (PID: {})", s.game_pid) },
                s.current_pattern_str
            );

            if s.target_address == 0 {
                info_string.push_str("Ожидание загрузки матча на поле стадиона...");
            } else {
                info_string.push_str(&format!("0x{:X}", s.target_address));
            }

            info_string.push_str(&format!("\n\n Системный лог:\n {}", s.log_message));

            let main_panel = Paragraph::new(info_string)
                .style(Style::default().fg(Color::White))
                .block(Block::default().title(" ПАНЕЛЬ КВАНТОВОГО ХУКА RING 0 ").borders(Borders::ALL).border_style(Style::default().fg(status_color)))
                .wrap(Wrap { trim: true });

            // ИСПРАВЛЕНО: Рендерим центральную панель в chunks[1]
            f.render_widget(main_panel, chunks[1]);

            let footer = Paragraph::new(" [F5] Прямо в игре - Вкл/Выкл Чит   |   [Пробел] в консоли - Переключить   |   [Q] - Выход ")
                .style(Style::default().fg(Color::Yellow))
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));

            // ИСПРАВЛЕНО: Рендерим подвал в chunks[2]
            f.render_widget(footer, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char(' ') => {
                        let mut s = state.lock().unwrap();
                        if s.target_address != 0 {
                            s.is_active = !s.is_active;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
