use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::time::Duration;

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
            let state = self.state.lock().await;
            if state.should_quit {
                break;
            }

            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // Info bar
                        Constraint::Min(5),    // Main content
                        Constraint::Length(10), // Logs
                    ])
                    .split(f.size());

                // 1. Info Bar
                let info = Paragraph::new(format!(
                    " Visited: {} | Queued: {} | Errors: {} | Tech: {}",
                    state.visited_count, state.queued_count, state.error_count, state.tech_detected.len()
                ))
                .block(Block::default().borders(Borders::ALL).title("Statistics"));
                f.render_widget(info, chunks[0]);

                // 2. Main Content
                let main_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(25), // Active Task
                        Constraint::Percentage(25), // Tech Stack
                        Constraint::Percentage(25), // Leaks
                        Constraint::Percentage(25), // APIs
                    ])
                    .split(chunks[1]);

                let current_task = Paragraph::new(format!("Current: {}", state.current_url))
                    .block(Block::default().borders(Borders::ALL).title("Active Task"));
                f.render_widget(current_task, main_layout[0]);

                let tech_list: Vec<ListItem> = state.tech_detected.iter().rev().take(10)
                    .map(|t| ListItem::new(t.as_str())).collect();
                let tech_box = List::new(tech_list)
                    .block(Block::default().borders(Borders::ALL).title("Tech Stack"));
                f.render_widget(tech_box, main_layout[1]);

                let leaks_list: Vec<ListItem> = state.leaks_detected.iter().rev().take(10)
                    .map(|l| ListItem::new(format!("[!] {}: {}", l.kind, l.value))).collect();
                let leaks_box = List::new(leaks_list)
                    .block(Block::default().borders(Borders::ALL).title("Security"));
                f.render_widget(leaks_box, main_layout[2]);

                let api_list: Vec<ListItem> = state.api_endpoints.iter().rev().take(10)
                    .map(|e| ListItem::new(format!("{} {}", e.method, e.url))).collect();
                let api_box = List::new(api_list)
                    .block(Block::default().borders(Borders::ALL).title("API Mapping"));
                f.render_widget(api_box, main_layout[3]);

                // 3. Logs
                let logs_list: Vec<ListItem> = state.logs.iter().rev().take(8)
                    .map(|l| ListItem::new(l.as_str())).collect();
                let logs_box = List::new(logs_list)
                    .block(Block::default().borders(Borders::ALL).title("Recent Activity"));
                f.render_widget(logs_box, chunks[2]);
            })?;

            if event::poll(Duration::from_millis(100))? {
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
