use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph, List, ListItem, BorderType},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::time::Duration;

pub const ASCII_LOGO: &str = r#"
  _____ _______ _____          _____ _  __
 |  ___|__   __|  __ \   /\   / ____| |/ /
 | |_     | |  | |__) | /  \ | |    | ' / 
 |  _|    | |  |  _  / / /\ \| |    |  <  
 | |      | |  | | \ \/ ____ \ |____| . \ 
 |_|      |_|  |_|  \_\_/    \_\_____|_|\_\
"#;

#[derive(Default)]
pub struct TuiState {
    pub current_url: String,
    pub visited_count: usize,
    pub queued_count: usize,
    pub error_count: usize,
    pub tech_detected: Vec<String>,
    pub leaks_detected: Vec<crate::types::leak::Leak>,
    pub api_endpoints: Vec<crate::types::endpoint::Endpoint>,
    pub logs: Vec<String>,
    pub should_quit: bool,
    pub frame_count: u64,
}

impl TuiState {
    pub fn add_log(&mut self, log: String) {
        self.logs.push(log);
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }
}

pub struct TuiDashboard {
    state: Arc<Mutex<TuiState>>,
}

impl TuiDashboard {
    pub fn new(state: Arc<Mutex<TuiState>>) -> Self {
        Self { state }
    }

    pub async fn run(&self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            // 1. Update frame count and check for quit signal
            {
                let mut state = self.state.lock().await;
                state.frame_count += 1;
                if state.should_quit {
                    break;
                }
            }

            // 2. Draw the TUI
            {
                let state = self.state.lock().await;
                terminal.draw(|f| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(8),  // Logo & Branding
                            Constraint::Length(3),  // Info/Stats Bar
                            Constraint::Min(10),    // Main Content (Grid)
                            Constraint::Length(10), // Logs/Activity
                        ])
                        .split(f.size());

                    // 0. Logo & Branding
                    let logo = Paragraph::new(ASCII_LOGO)
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::Cyan));
                    f.render_widget(logo, chunks[0]);

                    // 1. Info Bar (Stats)
                    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                    let spinner = spinner_frames[(state.frame_count % 10) as usize];
                    
                    let info = Paragraph::new(Line::from(vec![
                        Span::raw(" ["),
                        Span::styled(spinner, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::raw("] "),
                        Span::styled("Visited: ", Style::default().fg(Color::Gray)),
                        Span::styled(state.visited_count.to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        Span::raw(" | "),
                        Span::styled("Queued: ", Style::default().fg(Color::Gray)),
                        Span::styled(state.queued_count.to_string(), Style::default().fg(Color::Blue)),
                        Span::raw(" | "),
                        Span::styled("Errors: ", Style::default().fg(Color::Gray)),
                        Span::styled(state.error_count.to_string(), Style::default().fg(Color::Red)),
                    ]))
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(" Engine Status "));
                    f.render_widget(info, chunks[1]);

                    // 2. Main Content Grid
                    let grid = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(40), // Active & Tech
                            Constraint::Percentage(30), // Security
                            Constraint::Percentage(30), // API
                        ])
                        .split(chunks[2]);

                    // Left: Active Task & Stack
                    let left_col = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(5)])
                        .split(grid[0]);

                    let active_task = Paragraph::new(format!(" > {}", state.current_url))
                        .style(Style::default().fg(Color::White))
                        .block(Block::default().borders(Borders::ALL).title(" Active Target "));
                    f.render_widget(active_task, left_col[0]);

                    let tech_list: Vec<ListItem> = state.tech_detected.iter().rev().take(15)
                        .map(|t| ListItem::new(format!(" • {}", t)).style(Style::default().fg(Color::Cyan)))
                        .collect();
                    let tech_box = List::new(tech_list)
                        .block(Block::default().borders(Borders::ALL).title(" Tech Stack "));
                    f.render_widget(tech_box, left_col[1]);

                    // Center: Security Analysis
                    let leaks_list: Vec<ListItem> = state.leaks_detected.iter().rev().take(15)
                        .map(|l| ListItem::new(vec![
                            Line::from(vec![
                                Span::styled(format!("[!] {}", l.kind), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                            ]),
                            Line::from(vec![
                                Span::styled(format!("  └─ {}", l.value), Style::default().fg(Color::Gray)),
                            ]),
                        ]))
                        .collect();
                    let leaks_box = List::new(leaks_list)
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(if state.leaks_detected.is_empty() { Color::Gray } else { Color::Red }))
                            .title(" Security Alerts "));
                    f.render_widget(leaks_box, grid[1]);

                    // Right: API Mapping
                    let api_list: Vec<ListItem> = state.api_endpoints.iter().rev().take(15)
                        .map(|e| ListItem::new(format!("  {} {}", e.method, e.url))
                            .style(Style::default().fg(Color::Magenta)))
                        .collect();
                    let api_box = List::new(api_list)
                        .block(Block::default().borders(Borders::ALL).title(" API Mapping "));
                    f.render_widget(api_box, grid[2]);

                    // 3. Logs
                    let logs_list: Vec<ListItem> = state.logs.iter().rev().take(8)
                        .map(|l| ListItem::new(format!("  {}", l)).style(Style::default().fg(Color::DarkGray)))
                        .collect();
                    let logs_box = List::new(logs_list)
                        .block(Block::default().borders(Borders::ALL).title(" Activity Log "));
                    f.render_widget(logs_box, chunks[3]);
                })?;
            } // Lock dropped here

            // 3. Handle Events (Non-blocking)
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        let mut state = self.state.lock().await;
                        state.should_quit = true;
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        Ok(())
    }
}
