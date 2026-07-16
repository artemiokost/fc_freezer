use std::{io, sync::{Arc, Mutex}};
use std::time::Duration;
use fc_freezer::{config::load_or_create_config, worker::{spawn_workers, TrainerState}};
use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::{Color, Modifier, Style}, widgets::{Block, Borders, Paragraph, Wrap}, Terminal};

fn main() -> Result<(), io::Error> {
    let config = load_or_create_config();
    let state = Arc::new(Mutex::new(TrainerState {
        driver_status_str: String::from("Инициализация..."),
        game_pid: 0, addr_ai: 0, addr_net: 0,
        mod_disable_ai: false, mod_div_spoof: false, mod_draft_round: false, mod_wl_spoof: false, mod_server_change: false, mod_alttab_bypass: false,
        log_message: String::from("[*] Сканирование дескрипторов сети и сессий..."),
    }));

    spawn_workers(Arc::clone(&state), config.clone());
    enable_raw_mode()?; let mut stdout = io::stdout(); execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout); let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default().direction(Direction::Vertical).margin(1).constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(7), Constraint::Length(3)].as_ref()).split(area);
            let s = state.lock().unwrap();

            let header = Paragraph::new(" 🔥 EA SPORTS FC 26 - KERNEL CHEATARMY EXTENSION v4.5 ").style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)).alignment(ratatui::layout::Alignment::Center).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(header, chunks[0]);

            let mut active_str = format!(" ДРАЙВЕР ЯДРА RING 0: {}\n Процесс игры:       {}\n\n MODS STATUS:\n", s.driver_status_str, if s.game_pid == 0 { String::from("Ожидание запуска...") } else { format!("FC26.exe (PID: {})", s.game_pid) });
            active_str.push_str(&format!(" [F5] Disable AI Opponent:       {}\n", if s.mod_disable_ai { "[ ACTIVE ]" } else { "[ OFF ]" }));
            active_str.push_str(&format!(" [F6] Division Spoofer (Div {}):  {}\n", config.spoofed_division, if s.mod_div_spoof { "[ ACTIVE ]" } else { "[ OFF ]" }));
            active_str.push_str(&format!(" [F7] Draft Round Modifier (R{}): {}\n", config.spoofed_draft_round, if s.mod_draft_round { "[ ACTIVE ]" } else { "[ OFF ]" }));
            active_str.push_str(&format!(" [F8] WL Win Record Spoofer ({}): {}\n", config.spoofed_wl_wins, if s.mod_wl_spoof { "[ ACTIVE ]" } else { "[ OFF ]" }));
            active_str.push_str(&format!(" [F9] Server Changer (Loc ID {}): {}\n", config.server_location_id, if s.mod_server_change { "[ ACTIVE ]" } else { "[ OFF ]" }));
            active_str.push_str(&format!(" [F10] ALT-TAB Background Bypass: {}\n", if s.mod_alttab_bypass { "[ ACTIVE ]" } else { "[ OFF ]" }));
            active_str.push_str(&format!("\n Лог шины ядра:\n {}", s.log_message));

            let main_panel = Paragraph::new(active_str).style(Style::default().fg(Color::White)).block(Block::default().title(" АППАРАТНАЯ СЕТЕВАЯ ИГРОВАЯ СЕССИЯ ").borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta))).wrap(Wrap { trim: true });
            f.render_widget(main_panel, chunks[1]);

            let tips_str = String::from(" ⚠️ СИСТЕМА БЕЗОПАСНОСТИ CHEATARMY:\n\n • ИСПОЛЬЗУЙТЕ СТРОГО СДВОРЕННЫЕ / АЛЬТЕРНАТИВНЫЕ АККАУНТЫ, если беспокоитесь за прогресс.\n • ИЗБЕГАЙТЕ ОТКРОВЕННОЙ ДЕМОНСТРАЦИИ: не отключайте ИИ ботов в казуальных лобби на глазах игроков.\n • ДЕРЖИТЕ НАСТРОЙКИ СКРОМНЫМИ: устанавливайте дивизионы и победы, имитируя человеческие результаты.\n • СЛЕДИТЕ ЗА ОБНОВЛЕНИЯМИ В DISCORD-КАНАЛЕ для получения информации о патчах безопасности.");
            let tips_panel = Paragraph::new(tips_str).style(Style::default().fg(Color::Yellow)).block(Block::default().title(" TIPS FOR SAFE USE ").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow))).wrap(Wrap { trim: true });
            f.render_widget(tips_panel, chunks[2]);

            let footer = Paragraph::new(" Нажимайте клавиши F5 - F10 прямо внутри онлайн сессий | Нажмите [Q] для закрытия ").style(Style::default().fg(Color::DarkGray)).alignment(ratatui::layout::Alignment::Center).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(footer, chunks[3]);
        })?;

        if event::poll(Duration::from_millis(50))? { if let Event::Key(key) = event::read()? { match key.code { KeyCode::Char('q') | KeyCode::Char('Q') => break, _ => {} } } }
    }
    disable_raw_mode()?; execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?; terminal.show_cursor()?; Ok(())
}
