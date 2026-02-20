use crate::config::Config;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::ListState;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use sysinfo::Disks;

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    Search,
}

#[derive(PartialEq)]
pub enum Focus {
    FileList,
    DriveList,
}

pub struct App {
    pub current_dir: PathBuf,
    pub items: Vec<PathBuf>,
    pub filtered_items: Vec<PathBuf>,
    pub state: ListState,

    pub drives: Vec<String>,
    pub drive_state: ListState,
    pub focus: Focus,

    pub input_mode: InputMode,
    pub input_buffer: String,
    pub search_query: String,
    pub message: String,
    pub preview_content: String,

    pub config: Config,
}

impl App {
    pub fn new() -> App {
        // Пытаемся загрузить конфиг, если не выходит — берем дефолтный и пишем ошибку
        let (config, msg) = match Config::load() {
            Ok(c) => (c, String::new()),
            Err(e) => (Config::default(), format!("Config Load Error: {}", e)),
        };

        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let mut app = App {
            current_dir,
            items: Vec::new(),
            filtered_items: Vec::new(),
            state: ListState::default(),
            drives: Vec::new(),
            drive_state: ListState::default(),
            focus: Focus::FileList,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            search_query: String::new(),
            message: msg,
            preview_content: String::new(),
            config,
        };

        app.refresh_items();
        app.refresh_drives();
        app
    }

    // --- HOT RELOAD CONFIG ---
    pub fn reload_config(&mut self) {
        match Config::load() {
            Ok(new_config) => {
                self.config = new_config;
                self.message = "Config reloaded successfully!".to_string();
            }
            Err(e) => {
                self.message = format!("Config reload failed: {}", e);
            }
        }
    }

    pub fn refresh_items(&mut self) {
        self.items.clear();
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                self.items.push(entry.path());
            }
        }

        self.items.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();
            if a_is_dir == b_is_dir {
                a.file_name().cmp(&b.file_name())
            } else if a_is_dir {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        // Сброс поиска при обновлении списка
        self.search_query.clear();
        self.update_search();
    }

    pub fn update_search(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_items = self.items.clone();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_items = self
                .items
                .iter()
                .filter(|path| {
                    if let Some(name) = path.file_name() {
                        name.to_string_lossy().to_lowercase().contains(&query)
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();
        }
        self.state.select(Some(0));
        self.update_preview();
    }

    pub fn refresh_drives(&mut self) {
        let disks = Disks::new_with_refreshed_list();
        self.drives = disks
            .list()
            .iter()
            .map(|disk| disk.mount_point().to_string_lossy().to_string())
            .collect();

        if !self.drives.is_empty() {
            self.drive_state.select(Some(0));
        }
    }

    pub fn update_preview(&mut self) {
        self.preview_content.clear();

        if let Some(selected) = self.state.selected() {
            if let Some(path) = self.filtered_items.get(selected) {
                if path.is_dir() {
                    self.preview_content = format!("Directory: {}\n\nContains:", path.display());
                    if let Ok(entries) = fs::read_dir(path) {
                        for (i, entry) in entries.flatten().enumerate() {
                            if i > 20 {
                                self.preview_content.push_str("\n...and more...");
                                break;
                            }
                            let name = entry.file_name().to_string_lossy().to_string();
                            self.preview_content.push_str(&format!("\n- {}", name));
                        }
                    }
                } else {
                    match fs::File::open(path) {
                        Ok(mut file) => {
                            let mut buffer = [0; 1024];
                            if let Ok(n) = file.read(&mut buffer) {
                                let text = String::from_utf8_lossy(&buffer[..n]);
                                self.preview_content = text.to_string();
                            } else {
                                self.preview_content = "Binary/Unreadable".to_string();
                            }
                        }
                        Err(e) => self.preview_content = format!("Error: {}", e),
                    }
                }
            }
        }
    }

    pub fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> io::Result<()> {
        loop {
            terminal
                .draw(|f| crate::ui::render(f, self))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // --- GLOBAL: HOT RELOAD ---
                // F5 работает всегда + кнопка из конфига
                if key.code == KeyCode::F(5) || key_matches(&key, &self.config.keys.reload) {
                    self.reload_config();
                    continue;
                }

                // --- GLOBAL: FOCUS SWITCH ---
                if key_matches(&key, &self.config.keys.focus_drives) {
                    self.focus = Focus::DriveList;
                    continue;
                }
                if key_matches(&key, &self.config.keys.focus_files) {
                    self.focus = Focus::FileList;
                    continue;
                }

                match self.input_mode {
                    InputMode::Normal => {
                        if key_matches(&key, &self.config.keys.quit) {
                            return Ok(());
                        }

                        if key_matches(&key, &self.config.keys.search) {
                            self.input_mode = InputMode::Search;
                            self.search_query.clear();
                            self.update_search();
                            continue;
                        }

                        match self.focus {
                            Focus::FileList => self.handle_file_list_input(key),
                            Focus::DriveList => self.handle_drive_list_input(key),
                        }
                    }
                    InputMode::Editing => self.handle_editing_input(key),
                    InputMode::Search => self.handle_search_input(key),
                }
            }
        }
    }

    fn handle_file_list_input(&mut self, key: KeyEvent) {
        if key_matches(&key, &self.config.keys.down) || key.code == KeyCode::Down {
            self.next_item();
        } else if key_matches(&key, &self.config.keys.up) || key.code == KeyCode::Up {
            self.previous_item();
        } else if key_matches(&key, &self.config.keys.submit) {
            if let Some(selected) = self.state.selected() {
                if let Some(path) = self.filtered_items.get(selected) {
                    let path = path.clone();
                    if path.is_dir() {
                        self.current_dir = path;
                        self.refresh_items();
                    }
                }
            }
        } else if key_matches(&key, &self.config.keys.back_dir) {
            if let Some(parent) = self.current_dir.parent() {
                self.current_dir = parent.to_path_buf();
                self.refresh_items();
            }
        } else if key_matches(&key, &self.config.keys.create) {
            self.input_mode = InputMode::Editing;
        } else if key_matches(&key, &self.config.keys.delete) {
            self.delete_item();
        }
    }

    fn handle_drive_list_input(&mut self, key: KeyEvent) {
        if key_matches(&key, &self.config.keys.down) || key.code == KeyCode::Down {
            let i = match self.drive_state.selected() {
                Some(i) => {
                    if i >= self.drives.len().saturating_sub(1) {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.drive_state.select(Some(i));
        } else if key_matches(&key, &self.config.keys.up) || key.code == KeyCode::Up {
            let i = match self.drive_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.drives.len().saturating_sub(1)
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.drive_state.select(Some(i));
        } else if key_matches(&key, &self.config.keys.submit) {
            if let Some(selected) = self.drive_state.selected() {
                if let Some(drive_str) = self.drives.get(selected) {
                    self.current_dir = PathBuf::from(drive_str);
                    self.refresh_items();
                    self.focus = Focus::FileList;
                }
            }
        }
    }

    fn handle_editing_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                self.create_item();
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Char(c) => self.input_buffer.push(c),
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            _ => {}
        }
    }

    fn handle_search_input(&mut self, key: KeyEvent) {
        if key_matches(&key, &self.config.keys.submit) {
            self.input_mode = InputMode::Normal;
        } else if key_matches(&key, &self.config.keys.cancel) {
            self.input_mode = InputMode::Normal;
            self.search_query.clear();
            self.update_search();
        } else if key.code == KeyCode::Backspace {
            self.search_query.pop();
            self.update_search();
        } else if let KeyCode::Char(c) = key.code {
            self.search_query.push(c);
            self.update_search();
        } else if key_matches(&key, &self.config.keys.down) || key.code == KeyCode::Down {
            self.next_item();
        } else if key_matches(&key, &self.config.keys.up) || key.code == KeyCode::Up {
            self.previous_item();
        }
    }

    fn next_item(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.update_preview();
    }

    fn previous_item(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.update_preview();
    }

    fn create_item(&mut self) {
        let new_path = self.current_dir.join(&self.input_buffer);
        let res = if self.input_buffer.ends_with('/') {
            fs::create_dir(&new_path)
        } else {
            fs::File::create(&new_path).map(|_| ())
        };
        match res {
            Ok(_) => self.message = format!("Created: {}", self.input_buffer),
            Err(e) => self.message = format!("Error: {}", e),
        }
        self.refresh_items();
        self.update_preview();
    }

    fn delete_item(&mut self) {
        if let Some(selected) = self.state.selected() {
            if let Some(path) = self.filtered_items.get(selected) {
                let res = if path.is_dir() {
                    fs::remove_dir_all(path)
                } else {
                    fs::remove_file(path)
                };
                match res {
                    Ok(_) => self.message = "Deleted".to_string(),
                    Err(e) => self.message = format!("Error: {}", e),
                }
                self.refresh_items();
                self.update_preview();
            }
        }
    }
}

// Хелпер сравнения клавиш
fn key_matches(key: &KeyEvent, binding: &str) -> bool {
    let binding = binding.to_lowercase();
    let code_str = match key.code {
        KeyCode::Char(c) => c.to_string().to_lowercase(),
        KeyCode::Enter => "enter".to_string(),
        KeyCode::Esc => "esc".to_string(),
        KeyCode::Backspace => "backspace".to_string(),
        KeyCode::Tab => "tab".to_string(),
        KeyCode::Delete => "delete".to_string(),
        KeyCode::Left => "left".to_string(),
        KeyCode::Right => "right".to_string(),
        KeyCode::Up => "up".to_string(),
        KeyCode::Down => "down".to_string(),
        KeyCode::F(n) => format!("f{}", n), // Поддержка F1-F12
        _ => String::new(),
    };

    let mut parts = Vec::new();
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("ctrl");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        parts.push("alt");
    }

    if !parts.is_empty() || code_str.len() > 1 {
        parts.push(&code_str);
        let formed = parts.join("-");
        formed == binding
    } else {
        code_str == binding
    }
}
