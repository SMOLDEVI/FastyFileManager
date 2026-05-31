use crate::config::Config;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::ListState;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;
use sysinfo::Disks;

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    Search,
    Renaming,
}

#[derive(PartialEq, Clone)]
pub enum Focus {
    FileList,
    DriveList,
    Favorites,
}

#[derive(PartialEq, Clone)]
pub enum ClipboardOp {
    Copy,
    Cut,
}

#[derive(PartialEq, Clone, Copy, Default)]
pub enum SortMode {
    #[default]
    Name,
    Size,
    Date,
}

#[derive(PartialEq, Clone, Copy)]
pub enum ConflictAction {
    Overwrite,
    Skip,
    RenameAuto,
    Cancel,
}

#[derive(Clone, Copy)]
pub struct CachedMeta {
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

pub struct App {
    pub current_dir: PathBuf,
    pub items: Vec<PathBuf>,
    pub meta_cache: HashMap<PathBuf, CachedMeta>,
    pub filtered_items: Vec<PathBuf>,
    pub state: ListState,

    pub drives: Vec<(String, u64)>,
    pub drive_state: ListState,
    pub focus: Focus,

    pub input_mode: InputMode,
    pub input_buffer: String,
    pub search_query: String,
    pub message: String,
    pub preview_content: String,
    pub file_to_edit: Option<PathBuf>,

    pub clipboard: Option<(Vec<PathBuf>, ClipboardOp)>,
    pub selected_indices: HashSet<usize>,

    pub favorites: Vec<PathBuf>,
    pub favorites_state: ListState,

    pub show_statusbar: bool,
    pub show_help: bool,
    pub help_scroll: u16,

    pub confirm_delete: bool,
    pub pending_delete: Option<PathBuf>,

    pub sort_mode: SortMode,
    pub left_panel_pct: u16,
    pub center_panel_pct: u16,

    pub conflict_src: Option<PathBuf>,
    pub conflict_dest: Option<PathBuf>,
    pub conflict_paste_queue: Option<(Vec<PathBuf>, ClipboardOp, usize)>,

    pub config: Config,
    pub update_available: Option<String>,
    pub update_checker: Arc<Mutex<Option<String>>>,
}

impl App {
    pub fn new() -> App {
        let (config, msg) = match Config::load() {
            Ok(c) => (c, String::new()),
            Err(e) => (Config::default(), format!("Config Load Error: {}", e)),
        };

        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let favorites = load_favorites();

        let mut app = App {
            current_dir,
            items: Vec::new(),
            meta_cache: HashMap::new(),
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
            file_to_edit: None,
            clipboard: None,
            selected_indices: HashSet::new(),
            favorites,
            favorites_state: ListState::default(),
            show_statusbar: true,
            show_help: false,
            help_scroll: 0,
            confirm_delete: false,
            pending_delete: None,
            sort_mode: SortMode::Name,
            left_panel_pct: 20,
            center_panel_pct: 40,
            conflict_src: None,
            conflict_dest: None,
            conflict_paste_queue: None,
            config,
            update_available: None,
            update_checker: Arc::new(Mutex::new(None)),
        };

        app.refresh_items();
        app.refresh_drives();
        app.spawn_update_checker();
        app
    }

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
        self.meta_cache.clear();
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let is_dir = path.is_dir();
                if let Ok(meta) = fs::metadata(&path) {
                    self.meta_cache.insert(path.clone(), CachedMeta {
                        is_dir,
                        size: meta.len(),
                        modified: meta.modified().ok(),
                    });
                }
                self.items.push(path);
            }
        }

        match self.sort_mode {
            SortMode::Name => {
                self.items.sort_by(|a, b| {
                    let a_is_dir = self.meta_cache.get(a).is_some_and(|m| m.is_dir);
                    let b_is_dir = self.meta_cache.get(b).is_some_and(|m| m.is_dir);
                    if a_is_dir == b_is_dir {
                        a.file_name().cmp(&b.file_name())
                    } else if a_is_dir {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                });
            }
            SortMode::Size => {
                self.items.sort_by(|a, b| {
                    let a_is_dir = self.meta_cache.get(a).is_some_and(|m| m.is_dir);
                    let b_is_dir = self.meta_cache.get(b).is_some_and(|m| m.is_dir);
                    if a_is_dir != b_is_dir {
                        return if a_is_dir { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
                    }
                    let a_size = self.meta_cache.get(a).map(|m| m.size).unwrap_or(0);
                    let b_size = self.meta_cache.get(b).map(|m| m.size).unwrap_or(0);
                    b_size.cmp(&a_size)
                });
            }
            SortMode::Date => {
                self.items.sort_by(|a, b| {
                    let a_is_dir = self.meta_cache.get(a).is_some_and(|m| m.is_dir);
                    let b_is_dir = self.meta_cache.get(b).is_some_and(|m| m.is_dir);
                    if a_is_dir != b_is_dir {
                        return if a_is_dir { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
                    }
                    let a_date = self.meta_cache.get(a).and_then(|m| m.modified);
                    let b_date = self.meta_cache.get(b).and_then(|m| m.modified);
                    b_date.cmp(&a_date)
                });
            }
        }

        self.search_query.clear();
        self.selected_indices.clear();
        self.update_search();
    }

    pub fn update_search(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_items = self.items.clone();
        } else {
            self.filtered_items = self
                .items
                .iter()
                .filter(|path| {
                    if let Some(name) = path.file_name() {
                        fuzzy_match(&name.to_string_lossy(), &self.search_query)
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
            .map(|disk| (disk.mount_point().to_string_lossy().to_string(), disk.available_space()))
            .collect();

        if !self.drives.is_empty() {
            self.drive_state.select(Some(0));
        }
    }

    pub fn update_preview(&mut self) {
        self.preview_content.clear();

        if let Some(selected) = self.state.selected()
            && let Some(path) = self.filtered_items.get(selected) {
                let is_dir = self.meta_cache.get(path).is_some_and(|m| m.is_dir);
                if is_dir {
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
                            let mut buffer = [0; 4096];
                            if let Ok(n) = file.read(&mut buffer) {
                                let text = String::from_utf8_lossy(&buffer[..n]);
                                let content = text.to_string();
                                self.preview_content = content.lines().take(60).collect::<Vec<_>>().join("\n");
                            } else {
                                self.preview_content = "Binary/Unreadable".to_string();
                            }
                        }
                        Err(e) => self.preview_content = format!("Error: {}", e),
                    }
                }
            } else {
                self.preview_content = "No file selected\n——————————————\nNavigate with j/k   Open with l/Enter\nSearch with /       Copy with y, Paste with p\nFavorites with f    Delete with D\n\nPress ? for help".to_string();
            }
    }

    // --- SELECTION ---
    pub fn toggle_selection(&mut self) {
        if let Some(selected) = self.state.selected()
            && selected < self.filtered_items.len()
            && !self.selected_indices.remove(&selected) {
                self.selected_indices.insert(selected);
            }
    }

    // --- SORTING ---
    pub fn cycle_sort_mode(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Name => SortMode::Size,
            SortMode::Size => SortMode::Date,
            SortMode::Date => SortMode::Name,
        };
        let label = match self.sort_mode {
            SortMode::Name => "Name",
            SortMode::Size => "Size",
            SortMode::Date => "Date",
        };
        self.message = format!("Sort by: {}", label);
        self.refresh_items();
    }

    // --- UPDATE CHECKER ---
    pub fn spawn_update_checker(&self) {
        let checker = self.update_checker.clone();
        let current = env!("CARGO_PKG_VERSION").to_string();
        thread::spawn(move || {
            let latest = check_github_version(&current);
            if let Some(v) = latest
                && let Ok(mut guard) = checker.lock() {
                    *guard = Some(v);
                }
        });
    }

    pub fn check_update_result(&mut self) {
        if self.update_available.is_some() {
            return;
        }
        if let Ok(mut guard) = self.update_checker.lock()
            && let Some(v) = guard.take() {
                self.update_available = Some(v.clone());
                self.message = format!("Update v{}! Run: cargo install ffm --force", v);
            }
    }

    // --- PANEL RESIZE ---
    pub fn resize_center(&mut self, delta: i16) {
        let new = (self.center_panel_pct as i16 + delta).clamp(20, 70) as u16;
        if (100u16).saturating_sub(self.left_panel_pct + new) >= 10 {
            self.center_panel_pct = new;
        }
    }

    fn selected_paths(&self) -> Vec<PathBuf> {
        if !self.selected_indices.is_empty() {
            let mut indices: Vec<_> = self.selected_indices.iter().copied().collect();
            indices.sort();
            indices.iter().filter_map(|&i| self.filtered_items.get(i).cloned()).collect()
        } else if let Some(selected) = self.state.selected() {
            self.filtered_items.get(selected).cloned().into_iter().collect()
        } else {
            Vec::new()
        }
    }

    // --- CLIPBOARD: COPY ---
    pub fn copy_item(&mut self) {
        let paths = self.selected_paths();
        if !paths.is_empty() {
            self.clipboard = Some((paths.clone(), ClipboardOp::Copy));
            self.message = format!("Copied {} item(s).", paths.len());
        }
    }

    // --- CLIPBOARD: CUT ---
    pub fn cut_item(&mut self) {
        let paths = self.selected_paths();
        if !paths.is_empty() {
            self.clipboard = Some((paths.clone(), ClipboardOp::Cut));
            self.message = format!("Cut {} item(s).", paths.len());
        }
    }

    // --- CLIPBOARD: PASTE ---
    pub fn paste_item(&mut self) {
        let (paths, op) = match self.clipboard.clone() {
            Some(p) => p,
            None => {
                self.message = "Clipboard is empty".to_string();
                return;
            }
        };
        if paths.is_empty() {
            self.message = "Clipboard is empty".to_string();
            return;
        }
        self.paste_next(paths, op, 0);
    }

    fn paste_next(&mut self, paths: Vec<PathBuf>, op: ClipboardOp, idx: usize) {
        if idx >= paths.len() {
            self.message = format!("Pasted {} item(s).", paths.len());
            if op == ClipboardOp::Cut {
                self.clipboard = None;
            }
            self.refresh_items();
            self.update_preview();
            return;
        }

        let src = &paths[idx];
        let file_name = match src.file_name() {
            Some(n) => n.to_os_string(),
            None => return self.paste_next(paths, op, idx + 1),
        };
        let dest = self.current_dir.join(&file_name);

        if dest == *src {
            return self.paste_next(paths, op, idx + 1);
        }

        if dest.exists() {
            self.conflict_src = Some(src.clone());
            self.conflict_dest = Some(dest);
            self.conflict_paste_queue = Some((paths, op, idx));
            self.message = format!(
                "'{}' exists. (O)verwrite / (S)kip / (R)ename / (Esc)Cancel",
                file_name.to_string_lossy()
            );
            return;
        }

        let result = match op {
            ClipboardOp::Copy => copy_recursive(src, &dest),
            ClipboardOp::Cut => fs::rename(src, &dest).map_err(|e| e.to_string()),
        };
        if let Err(e) = result {
            self.message = format!("Paste error: {}", e);
        }
        self.paste_next(paths, op, idx + 1);
    }

    fn do_paste_one(&mut self, src: &PathBuf, dest: &PathBuf, op: &ClipboardOp) {
        let result = match op {
            ClipboardOp::Copy => copy_recursive(src, dest),
            ClipboardOp::Cut => fs::rename(src, dest).map_err(|e| e.to_string()),
        };
        if let Err(e) = result {
            self.message = format!("Paste error: {}", e);
        }
    }

    pub fn resolve_conflict(&mut self, action: ConflictAction) {
        let (src, dest) = match (self.conflict_src.take(), self.conflict_dest.take()) {
            (Some(s), Some(d)) => (s, d),
            _ => return,
        };
        let (paths, op, idx) = match self.conflict_paste_queue.take() {
            Some(q) => q,
            _ => return,
        };

        match action {
            ConflictAction::Overwrite => {
                let _ = if dest.is_dir() { fs::remove_dir_all(&dest) } else { fs::remove_file(&dest) };
                self.do_paste_one(&src, &dest, &op);
                self.paste_next(paths, op, idx + 1);
            }
            ConflictAction::Skip => {
                self.message = "Skipped.".to_string();
                self.paste_next(paths, op, idx + 1);
            }
            ConflictAction::RenameAuto => {
                let new_dest = find_available_name(&dest);
                self.do_paste_one(&src, &new_dest, &op);
                self.paste_next(paths, op, idx + 1);
            }
            ConflictAction::Cancel => {
                self.message = "Paste cancelled.".to_string();
                if op == ClipboardOp::Cut {
                    self.clipboard = None;
                }
                self.refresh_items();
                self.update_preview();
            }
        }
    }

    // --- FAVORITES: ADD ---
    pub fn add_favorite(&mut self) {
        if let Some(selected) = self.state.selected()
            && let Some(path) = self.filtered_items.get(selected) {
                if !self.favorites.contains(path) {
                    self.favorites.push(path.clone());
                    save_favorites(&self.favorites);
                    self.message = format!(
                        "Added to favorites: {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    );
                } else {
                    self.message = "Already in favorites".to_string();
                }
            }
    }

    // --- FAVORITES: REMOVE (from favorites panel) ---
    pub fn remove_favorite(&mut self) {
        if let Some(selected) = self.favorites_state.selected()
            && selected < self.favorites.len() {
                let removed = self.favorites.remove(selected);
                save_favorites(&self.favorites);
                self.message = format!(
                    "Removed from favorites: {}",
                    removed.file_name().unwrap_or_default().to_string_lossy()
                );
                if self.favorites.is_empty() {
                    self.favorites_state.select(None);
                } else {
                    let new_sel = selected.min(self.favorites.len() - 1);
                    self.favorites_state.select(Some(new_sel));
                }
            }
    }

    // --- FAVORITES: NAVIGATE ---
    pub fn open_favorite(&mut self) {
        if let Some(selected) = self.favorites_state.selected()
            && let Some(path) = self.favorites.get(selected).cloned() {
                if path.is_dir() {
                    self.current_dir = path;
                    self.refresh_items();
                    self.focus = Focus::FileList;
                } else if let Some(parent) = path.parent() {
                    self.current_dir = parent.to_path_buf();
                    self.refresh_items();
                    self.focus = Focus::FileList;
                }
            }
    }

    pub fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> io::Result<()> {
        loop {
            self.check_update_result();

            terminal
                .draw(|f| crate::ui::render(f, self))
                .map_err(|e| io::Error::other(e.to_string()))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // --- GLOBAL: HOT RELOAD ---
                if key.code == KeyCode::F(5) || key_matches(&key, &self.config.keys.reload) {
                    self.reload_config();
                    continue;
                }

                // --- GLOBAL: TOGGLE STATUSBAR (Ctrl+B) ---
                if key.code == KeyCode::Char('b') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.show_statusbar = !self.show_statusbar;
                    continue;
                }

                // --- GLOBAL: TOGGLE HELP POPUP (?) ---
                if key.code == KeyCode::Char('?') {
                    self.show_help = !self.show_help;
                    if self.show_help {
                        self.help_scroll = 0;
                    }
                    continue;
                }

                // Esc закрывает help попап
                if key.code == KeyCode::Esc && self.show_help {
                    self.show_help = false;
                    self.help_scroll = 0;
                    continue;
                }

                // --- HELP SCROLL ---
                if self.show_help {
                    match key.code {
                        KeyCode::Up => self.help_scroll = self.help_scroll.saturating_sub(1),
                        KeyCode::Down => self.help_scroll = self.help_scroll.saturating_add(1),
                        _ => {}
                    }
                    continue;
                }

                // --- CONFLICT RESOLUTION ---
                if self.conflict_src.is_some() {
                    match key.code {
                        KeyCode::Char('o') | KeyCode::Char('O') => {
                            self.resolve_conflict(ConflictAction::Overwrite);
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            self.resolve_conflict(ConflictAction::Skip);
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            self.resolve_conflict(ConflictAction::RenameAuto);
                        }
                        KeyCode::Esc => {
                            self.resolve_conflict(ConflictAction::Cancel);
                        }
                        _ => {}
                    }
                    continue;
                }

                // --- DELETE CONFIRMATION ---
                if self.confirm_delete {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            self.execute_delete();
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            self.confirm_delete = false;
                            self.pending_delete = None;
                            self.message = "Delete cancelled.".to_string();
                        }
                        _ => {}
                    }
                    continue;
                }

                // --- GLOBAL: FOCUS SWITCH ---
                if key.code == KeyCode::Tab {
                    self.focus = match self.focus {
                        Focus::FileList => Focus::DriveList,
                        Focus::DriveList => Focus::Favorites,
                        Focus::Favorites => Focus::FileList,
                    };
                    continue;
                }
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
                            write_cwd(&self.current_dir);
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
                            Focus::Favorites => self.handle_favorites_input(key),
                        }
                    }
                    InputMode::Editing => self.handle_editing_input(key),
                    InputMode::Search => self.handle_search_input(key),
                    InputMode::Renaming => self.handle_renaming_input(key),
                }
            }

            if let Some(path) = self.file_to_edit.take() {
                let _ = terminal.clear();
                let _ = crossterm::terminal::disable_raw_mode();
                let mut stdout = std::io::stdout();
                let _ = crossterm::execute!(
                    stdout,
                    crossterm::terminal::LeaveAlternateScreen,
                );
                let _ = terminal.show_cursor();

                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
                let child = std::process::Command::new(editor)
                    .arg(&path)
                    .spawn();
                if let Ok(mut child) = child {
                    let _ = child.wait();
                } else {
                    self.message = "Failed to launch editor".to_string();
                }

                let _ = crossterm::terminal::enable_raw_mode();
                let mut stdout = std::io::stdout();
                let _ = crossterm::execute!(
                    stdout,
                    crossterm::terminal::EnterAlternateScreen,
                );
                let _ = terminal.hide_cursor();
                let _ = terminal.clear();
                self.update_preview();
            }
        }
    }

    fn handle_file_list_input(&mut self, key: KeyEvent) {
        if key_matches(&key, &self.config.keys.down) || key.code == KeyCode::Down {
            self.next_item();
        } else if key_matches(&key, &self.config.keys.up) || key.code == KeyCode::Up {
            self.previous_item();
        } else if key.code == KeyCode::Char(' ') {
            self.toggle_selection();
        } else if key.code == KeyCode::Left && key.modifiers.contains(KeyModifiers::SHIFT) {
            self.resize_center(-5);
        } else if key.code == KeyCode::Right && key.modifiers.contains(KeyModifiers::SHIFT) {
            self.resize_center(5);
        } else if key_matches(&key, &self.config.keys.submit) || key.code == KeyCode::Enter || key.code == KeyCode::Right {
            if let Some(selected) = self.state.selected()
                && let Some(path) = self.filtered_items.get(selected) {
                    let path = path.clone();
                    if path.is_dir() {
                        self.current_dir = path;
                        self.refresh_items();
                    }
                }
        } else if key_matches(&key, &self.config.keys.back_dir) || key.code == KeyCode::Backspace || key.code == KeyCode::Left {
            if let Some(parent) = self.current_dir.parent() {
                self.current_dir = parent.to_path_buf();
                self.refresh_items();
            }
        } else if key_matches(&key, &self.config.keys.create) {
            self.input_mode = InputMode::Editing;
        } else if key_matches(&key, &self.config.keys.delete) {
            self.delete_prompt();
        } else if key_matches(&key, &self.config.keys.rename) {
            self.start_rename();
        } else if key_matches(&key, &self.config.keys.edit) {
            if !self.selected_indices.is_empty() {
                let paths = self.selected_paths();
                if let Some(path) = paths.into_iter().find(|p| !p.is_dir()) {
                    self.file_to_edit = Some(path);
                }
            } else if let Some(selected) = self.state.selected()
                && let Some(path) = self.filtered_items.get(selected)
                    && !path.is_dir() {
                        self.file_to_edit = Some(path.clone());
                    }
        } else if key_matches(&key, &self.config.keys.sort) {
            self.cycle_sort_mode();
        }
        // --- CLIPBOARD ---
        else if key.code == KeyCode::Char('y') {
            self.copy_item();
        } else if key.code == KeyCode::Char('x') {
            self.cut_item();
        } else if key.code == KeyCode::Char('p') {
            self.paste_item();
        }
        // --- FAVORITES ---
        else if key.code == KeyCode::Char('f') {
            self.add_favorite();
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
        } else if (key_matches(&key, &self.config.keys.submit) || key.code == KeyCode::Enter || key.code == KeyCode::Right)
            && let Some(selected) = self.drive_state.selected()
                && let Some((mount, _)) = self.drives.get(selected) {
                    self.current_dir = PathBuf::from(mount);
                    self.refresh_items();
                    self.focus = Focus::FileList;
                }
    }

    fn handle_favorites_input(&mut self, key: KeyEvent) {
        if key_matches(&key, &self.config.keys.down) || key.code == KeyCode::Down {
            let len = self.favorites.len();
            if len == 0 {
                return;
            }
            let i = match self.favorites_state.selected() {
                Some(i) => {
                    if i >= len - 1 { 0 } else { i + 1 }
                }
                None => 0,
            };
            self.favorites_state.select(Some(i));
        } else if key_matches(&key, &self.config.keys.up) || key.code == KeyCode::Up {
            let len = self.favorites.len();
            if len == 0 {
                return;
            }
            let i = match self.favorites_state.selected() {
                Some(i) => {
                    if i == 0 { len - 1 } else { i - 1 }
                }
                None => 0,
            };
            self.favorites_state.select(Some(i));
        } else if key_matches(&key, &self.config.keys.submit)
            || key.code == KeyCode::Enter
            || key.code == KeyCode::Right
        {
            self.open_favorite();
        } else if key_matches(&key, &self.config.keys.delete) || key.code == KeyCode::Char('F') {
            self.remove_favorite();
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

    fn handle_renaming_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                self.rename_item();
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
                self.message = "Rename cancelled.".to_string();
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

    fn delete_prompt(&mut self) {
        let count = self.selected_indices.len();
        if count > 0 {
            self.pending_delete = None;
            self.confirm_delete = true;
            self.message = format!("Delete {} item(s)? (y/N)", count);
        } else if let Some(selected) = self.state.selected()
            && let Some(path) = self.filtered_items.get(selected) {
                self.pending_delete = Some(path.clone());
                self.confirm_delete = true;
                self.message = format!(
                    "Delete '{}'? (y/N)",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
            }
    }

    fn execute_delete(&mut self) {
        self.confirm_delete = false;
        let count = self.selected_indices.len();

        if count > 0 {
            let paths = std::mem::take(&mut self.selected_indices)
                .into_iter()
                .filter_map(|i| self.filtered_items.get(i).cloned())
                .collect::<Vec<_>>();
            for path in &paths {
                let _ = if path.is_dir() { fs::remove_dir_all(path) } else { fs::remove_file(path) };
            }
            self.message = format!("Deleted {} item(s).", paths.len());
        } else if let Some(path) = self.pending_delete.take() {
            let res = if path.is_dir() { fs::remove_dir_all(&path) } else { fs::remove_file(&path) };
            match res {
                Ok(_) => self.message = "Deleted.".to_string(),
                Err(e) => self.message = format!("Error: {}", e),
            }
        }
        self.refresh_items();
        self.update_preview();
    }

    fn start_rename(&mut self) {
        if let Some(selected) = self.state.selected()
            && let Some(path) = self.filtered_items.get(selected)
                && let Some(name) = path.file_name() {
                    self.input_buffer = name.to_string_lossy().to_string();
                    self.input_mode = InputMode::Renaming;
                    self.message = "Edit name and press Enter to rename.".to_string();
                }
    }

    fn rename_item(&mut self) {
        if let Some(selected) = self.state.selected()
            && let Some(path) = self.filtered_items.get(selected) {
                let new_path = path.parent().unwrap_or(&self.current_dir).join(&self.input_buffer);
                if new_path == *path {
                    self.message = "Same name, nothing changed.".to_string();
                } else {
                    match fs::rename(path, &new_path) {
                        Ok(_) => self.message = format!("Renamed to: {}", self.input_buffer),
                        Err(e) => self.message = format!("Rename error: {}", e),
                    }
                    self.refresh_items();
                    self.update_preview();
                }
            }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }
}

fn copy_recursive(src: &PathBuf, dest: &PathBuf) -> Result<(), String> {
    if src.is_dir() {
        fs::create_dir_all(dest).map_err(|e| e.to_string())?;
        for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let child_src = entry.path();
            let child_dest = dest.join(entry.file_name());
            copy_recursive(&child_src, &child_dest)?;
        }
        Ok(())
    } else {
        fs::copy(src, dest).map(|_| ()).map_err(|e| e.to_string())
    }
}

fn find_available_name(dest: &std::path::Path) -> PathBuf {
    let parent = dest.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
    let stem = dest.file_stem().and_then(|s| s.to_str()).unwrap_or("file").to_string();
    let ext = dest.extension().and_then(|e| e.to_str()).map(|e| format!(".{}", e)).unwrap_or_default();
    for i in 1..9999 {
        let name = format!("{} ({}){}", stem, i, ext);
        let candidate = parent.join(&name);
        if !candidate.exists() {
            return candidate;
        }
    }
    dest.with_file_name(format!("{} ({}){}", stem, 9999, ext))
}

fn write_cwd(dir: &std::path::Path) {
    if let Some(path) = cwd_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, dir.to_string_lossy().as_ref());
    }
}

fn cwd_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "ffm")
        .map(|p| p.data_dir().join("cwd"))
}

fn favorites_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "ffm")
        .map(|p| p.data_dir().join("favorites.txt"))
}

fn load_favorites() -> Vec<PathBuf> {
    if let Some(path) = favorites_path()
        && let Ok(content) = fs::read_to_string(&path) {
            return content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(PathBuf::from)
                .collect();
        }
    Vec::new()
}

fn save_favorites(favorites: &[PathBuf]) {
    if let Some(path) = favorites_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content: String = favorites
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let _ = fs::write(&path, content);
    }
}

fn check_github_version(current: &str) -> Option<String> {
    let cmd = if cfg!(windows) { "curl.exe" } else { "curl" };
    let url = "https://api.github.com/repos/SMOLDEVI/FastyFileManager/releases/latest";
    let mut child = std::process::Command::new(cmd);
    child.args(["-sL", "-H", "User-Agent: ffm", "--connect-timeout", "5", "--max-time", "10", url]);
    if cfg!(windows) {
        child.arg("--ssl-no-revoke");
    }
    let output = child.output().ok().filter(|o| o.status.success())?;
    let response = String::from_utf8(output.stdout).ok()?;
    let prefix = "\"tag_name\":\"";
    let start = response.find(prefix)?;
    let start = start + prefix.len();
    let end = response[start..].find('\"')?;
    let tag = &response[start..start + end];
    let version = tag.strip_prefix('v').unwrap_or(tag);
    if version != current { Some(version.to_string()) } else { None }
}

fn fuzzy_match(text: &str, query: &str) -> bool {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();
    let mut qi = query_lower.chars();
    let mut current = match qi.next() {
        Some(c) => c,
        None => return true,
    };
    for c in text_lower.chars() {
        if c == current {
            match qi.next() {
                Some(next) => current = next,
                None => return true,
            }
        }
    }
    false
}

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
        KeyCode::F(n) => format!("f{}", n),
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
