use crate::app::{App, ClipboardOp, Focus, InputMode, SortMode};
use crate::icons::{get_icon, get_icon_color};
use crate::theme::parse_color;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use std::fs;
use std::time::UNIX_EPOCH;

pub fn render(f: &mut Frame, app: &mut App) {
    let theme = &app.config.theme;

    let bg_color = parse_color(&theme.background);
    let text_color = parse_color(&theme.text);
    let sel_bg = parse_color(&theme.selected_bg);
    let sel_fg = parse_color(&theme.selected_fg);

    let dir_base_color = parse_color(&theme.directory);
    let file_base_color = parse_color(&theme.file);

    let active_border_color = sel_bg;
    let inactive_border_color = Color::DarkGray;
    let favorites_border_color = Color::Rgb(150, 120, 200);

    let area = f.area();

    // Главный фон
    let main_block = Block::default().style(Style::default().bg(bg_color));
    f.render_widget(main_block, area);

    // Вертикальное деление: Основной контент + (опционально) Статус бар
    let vertical_constraints = if app.show_statusbar {
        vec![Constraint::Min(1), Constraint::Length(3)]
    } else {
        vec![Constraint::Min(1)]
    };

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vertical_constraints)
        .split(area);

    // Горизонтальное деление: динамические проценты
    let left_pct = app.left_panel_pct;
    let center_pct = app.center_panel_pct;
    let right_pct = 100u16.saturating_sub(left_pct + center_pct);
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(left_pct),
            Constraint::Percentage(center_pct),
            Constraint::Percentage(right_pct),
        ])
        .split(vertical_chunks[0]);

    // Левая панель: Избранное сверху, Диски снизу
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(main_chunks[0]);

    // --- 1. ПАНЕЛЬ ИЗБРАННОГО (СЛЕВА СВЕРХУ) ---
    let fav_items: Vec<ListItem> = app
        .favorites
        .iter()
        .map(|path| {
            let name = path
                .file_name()
                .unwrap_or(path.as_os_str())
                .to_string_lossy();
            let icon = if path.is_dir() { "󰉒" } else { "󰈔" };
            ListItem::new(format!("{} {}", icon, name))
                .style(Style::default().fg(Color::Rgb(210, 180, 255)).bg(bg_color))
        })
        .collect();

    let fav_border_style = if app.focus == Focus::Favorites {
        Style::default()
            .fg(favorites_border_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(inactive_border_color)
    };

    let fav_highlight_style = if app.focus == Focus::Favorites {
        Style::default()
            .bg(Color::Rgb(100, 60, 160))
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let fav_list = List::new(fav_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" ★ Favorites ")
                .border_style(fav_border_style)
                .style(Style::default().bg(bg_color)),
        )
        .highlight_style(fav_highlight_style)
        .highlight_symbol("★ ");

    f.render_stateful_widget(fav_list, left_chunks[0], &mut app.favorites_state);

    // --- 2. ПАНЕЛЬ ДИСКОВ (СЛЕВА СНИЗУ) ---
    fn format_bytes(bytes: u64) -> String {
        const U: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut s = bytes as f64;
        let mut i = 0;
        while s >= 1024.0 && i < U.len() - 1 { s /= 1024.0; i += 1; }
        if i == 0 { format!("{} B", bytes) } else { format!("{:.1} {}", s, U[i]) }
    }

    let drive_items: Vec<ListItem> = app
        .drives
        .iter()
        .map(|(mount, free)| {
            let free_str = format_bytes(*free);
            ListItem::new(format!("󰉉 {}  {} free", mount, free_str))
                .style(Style::default().fg(text_color).bg(bg_color))
        })
        .collect();

    let drive_border_style = if app.focus == Focus::DriveList {
        Style::default()
            .fg(active_border_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(inactive_border_color)
    };

    let drive_highlight_style = if app.focus == Focus::DriveList {
        Style::default()
            .bg(sel_bg)
            .fg(sel_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let drive_list = List::new(drive_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Drives ")
                .border_style(drive_border_style)
                .style(Style::default().bg(bg_color)),
        )
        .highlight_style(drive_highlight_style)
        .highlight_symbol(theme.highlight_symbol.as_str());

    f.render_stateful_widget(drive_list, left_chunks[1], &mut app.drive_state);

    // --- 3. ПАНЕЛЬ ФАЙЛОВ (ЦЕНТР) ---
    let panel_width = main_chunks[1].width as usize;
    let name_max_width = panel_width.saturating_sub(26);

    let file_items: Vec<ListItem> = app
        .filtered_items
        .iter()
        .enumerate()
        .map(|(idx, path)| {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let icon = get_icon(path);

            let is_selected = app.selected_indices.contains(&idx);

            let in_clipboard = app
                .clipboard
                .as_ref()
                .is_some_and(|(paths, _)| paths.contains(path));

            let icon_color = if path.is_dir() {
                dir_base_color
            } else {
                get_icon_color(path).unwrap_or(file_base_color)
            };

            let has_sel = !app.selected_indices.is_empty();
            let sel_mark = if has_sel {
                if is_selected { " [x]" } else { " [ ]" }
            } else {
                ""
            };

            let clipboard_mark = if in_clipboard {
                match app.clipboard.as_ref().map(|(_, op)| op) {
                    Some(ClipboardOp::Copy) => "[C]",
                    Some(ClipboardOp::Cut) => "[X]",
                    None => "",
                }
            } else {
                ""
            };

            let (size_str, date_str) = if path.is_dir() {
                (String::new(), String::new())
            } else if let Ok(meta) = fs::metadata(path) {
                (format_size(meta.len()), format_date(meta.modified()))
            } else {
                (String::new(), String::new())
            };

            let name_display = if name.len() > name_max_width {
                let truncated: String = name.chars().take(name_max_width.saturating_sub(1)).collect();
                format!("{}…", truncated)
            } else {
                name.to_string()
            };

            let line = Line::from(vec![
                Span::styled(sel_mark, Style::default().fg(text_color)),
                Span::styled(icon, Style::default().fg(icon_color)),
                Span::styled(
                    format!(" {} {} {:>8} {}", name_display, clipboard_mark, size_str, date_str),
                    Style::default().fg(text_color),
                ),
            ]);
            ListItem::new(line).style(Style::default().bg(bg_color))
        })
        .collect();

    let file_border_style = if app.focus == Focus::FileList {
        Style::default()
            .fg(active_border_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(inactive_border_color)
    };

    let file_highlight_style = if app.focus == Focus::FileList {
        Style::default()
            .bg(sel_bg)
            .fg(sel_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let sel_count = app.selected_indices.len();
    let sel_info = if sel_count > 0 { format!(" ({})", sel_count) } else { String::new() };

    let list_title = if app.input_mode == InputMode::Search {
        format!(" Search: {} ", app.search_query)
    } else {
        let path_str = app.current_dir.to_string_lossy();
        let path_str = if path_str.len() > 50 {
            let start = path_str.len().saturating_sub(47);
            format!("…{}", &path_str[start..])
        } else {
            path_str.to_string()
        };
        format!(" {}{} ", path_str, sel_info)
    };

    let file_list = List::new(file_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(list_title)
                .border_style(file_border_style)
                .style(Style::default().bg(bg_color)),
        )
        .highlight_style(file_highlight_style)
        .highlight_symbol(theme.highlight_symbol.as_str());

    f.render_stateful_widget(file_list, main_chunks[1], &mut app.state);

    // --- 4. ПАНЕЛЬ ПРЕВЬЮ (СПРАВА) ---
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Preview ")
        .border_style(Style::default().fg(inactive_border_color))
        .style(Style::default().bg(bg_color));

    let preview_text = Paragraph::new(app.preview_content.clone())
        .block(preview_block)
        .style(Style::default().fg(text_color))
        .wrap(Wrap { trim: false });

    f.render_widget(preview_text, main_chunks[2]);

    // --- ФУТЕР (если включён) ---
    if app.show_statusbar {
        let clipboard_hint = match &app.clipboard {
            Some((paths, ClipboardOp::Copy)) => {
                let n = paths.len();
                if n == 1 {
                    format!(" │ 󰆏 Copy: {}", paths[0].file_name().unwrap_or_default().to_string_lossy())
                } else {
                    format!(" │ 󰆏 Copy: {} items", n)
                }
            }
            Some((paths, ClipboardOp::Cut)) => {
                let n = paths.len();
                if n == 1 {
                    format!(" │ 󰆐 Cut: {}", paths[0].file_name().unwrap_or_default().to_string_lossy())
                } else {
                    format!(" │ 󰆐 Cut: {} items", n)
                }
            }
            None => String::new(),
        };

        let sort_label = match app.sort_mode {
            SortMode::Name => "Name",
            SortMode::Size => "Size",
            SortMode::Date => "Date",
        };
        let mode_text = match app.input_mode {
            InputMode::Normal => match app.focus {
                Focus::FileList => format!(" FILES [{}]", sort_label),
                Focus::DriveList => " DRIVES".to_string(),
                Focus::Favorites => "★ FAVORITES".to_string(),
            },
            InputMode::Editing => " EDITING".to_string(),
            InputMode::Search => " SEARCH".to_string(),
            InputMode::Renaming => " RENAME".to_string(),
        };

        let keys_hint = match app.input_mode {
            InputMode::Normal => match app.focus {
                Focus::FileList => format!(
                    "hjkl Nav │ Space Sel │ s Sort │ a New │ r Ren │ D Del │ {} Edit │ y Copy │ x Cut │ p Paste │ f Fav │ / Search │ ? Help │ Ctrl+B Bar",
                    app.config.keys.edit
                ),
                Focus::DriveList => "jk Nav │ Enter Open │ Tab Switch │ ? Help │ Ctrl+B Bar".to_string(),
                Focus::Favorites => "jk Nav │ Enter Open │ D Remove │ Tab Switch │ ? Help │ Ctrl+B Bar".to_string(),
            },
            InputMode::Editing => "Enter Save │ Esc Cancel".to_string(),
            InputMode::Search => "Enter Confirm │ Esc Cancel │ ↑↓ Navigate".to_string(),
            InputMode::Renaming => "Enter Confirm │ Esc Cancel".to_string(),
        };

        let msg = if app.message.is_empty() {
            String::new()
        } else {
            format!(" │ {}", app.message)
        };

        let status_text = format!(
            " {} │ {}{}{} ",
            mode_text, keys_hint, clipboard_hint, msg
        );
        let footer = Paragraph::new(status_text)
            .style(Style::default().fg(text_color).bg(bg_color))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(inactive_border_color))
                    .style(Style::default().bg(bg_color)),
            );

        f.render_widget(footer, vertical_chunks[1]);
    }

    // --- POPUPS ---

    // Создание файла/папки
    if let InputMode::Editing = app.input_mode {
        let area_rect = centered_rect(60, 20, area);
        f.render_widget(Clear, area_rect);

        let popup_block = Block::default()
            .title(" New file/folder  (end name with / for folder) ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(bg_color).fg(text_color));

        let input_text = Paragraph::new(app.input_buffer.clone())
            .style(Style::default().fg(Color::Yellow))
            .block(popup_block);

        f.render_widget(input_text, area_rect);
    }

    // Переименование файла/папки
    if let InputMode::Renaming = app.input_mode {
        let area_rect = centered_rect(60, 20, area);
        f.render_widget(Clear, area_rect);

        let popup_block = Block::default()
            .title(" Rename file/folder ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(bg_color).fg(text_color));

        let input_text = Paragraph::new(app.input_buffer.clone())
            .style(Style::default().fg(Color::Yellow))
            .block(popup_block);

        f.render_widget(input_text, area_rect);
    }

    // Подтверждение удаления
    if app.confirm_delete {
        let area_rect = centered_rect(50, 20, area);
        f.render_widget(Clear, area_rect);

        let delete_block = Block::default()
            .title(" Confirm Delete ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(bg_color).fg(text_color));

        let delete_text = Paragraph::new("Are you sure? (y/N)")
            .style(Style::default().fg(Color::LightRed))
            .block(delete_block);

        f.render_widget(delete_text, area_rect);
    }

    // Конфликт при вставке
    if app.conflict_src.is_some() {
        let area_rect = centered_rect(60, 20, area);
        f.render_widget(Clear, area_rect);

        let conflict_block = Block::default()
            .title(" File conflict ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(bg_color).fg(Color::LightRed));

        let conflict_text = Paragraph::new(vec![
            Line::from(Span::styled("Destination file already exists.", Style::default().fg(text_color))),
            Line::from(""),
            Line::from(Span::styled("(O) Overwrite   (S) Skip   (R) Auto-rename   (Esc) Cancel", Style::default().fg(Color::Yellow))),
        ])
        .block(conflict_block);

        f.render_widget(conflict_text, area_rect);
    }

    // Попап помощи
    if app.show_help {
        render_help_popup(f, area, bg_color, text_color, sel_bg, app.help_scroll);
    }
}

fn render_help_popup(
    f: &mut Frame,
    area: Rect,
    bg_color: Color,
    text_color: Color,
    accent: Color,
    scroll: u16,
) {
    let popup_area = centered_rect(72, 85, area);
    f.render_widget(Clear, popup_area);

    let header = Style::default()
        .fg(accent)
        .add_modifier(Modifier::BOLD);
    let key_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(text_color);
    let dim = Style::default().fg(Color::DarkGray);

    fn row<'a>(key: &'a str, desc: &'a str, key_style: Style, desc_style: Style) -> Line<'a> {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<18}", key), key_style),
            Span::styled(desc, desc_style),
        ])
    }

    let lines: Vec<Line> = vec![
        Line::from(Span::styled("  Navigation", header)),
        Line::from(Span::styled("  ──────────────────────────────────────────", dim)),
        row("j / ↓",         "Move down",                               key_style, desc_style),
        row("k / ↑",         "Move up",                                 key_style, desc_style),
        row("l / → / Enter", "Open directory",                          key_style, desc_style),
        row("h / ← / Bksp",  "Go to parent directory",                  key_style, desc_style),
        row("← Shift",       "Shrink center panel",                     key_style, desc_style),
        row("→ Shift",       "Expand center panel",                     key_style, desc_style),
        Line::from(""),
        Line::from(Span::styled("  File Operations", header)),
        Line::from(Span::styled("  ──────────────────────────────────────────", dim)),
        row("a",             "Create new file/folder (/ = folder)",     key_style, desc_style),
        row("r",             "Rename selected item",                    key_style, desc_style),
        row("D",             "Delete selected item (with confirm)",     key_style, desc_style),
        row("Space",         "Toggle selection",                        key_style, desc_style),
        row("s",             "Cycle sort: Name / Size / Date",          key_style, desc_style),
        row("e",             "Open in $EDITOR",                        key_style, desc_style),
        row("y",             "Copy to clipboard",                       key_style, desc_style),
        row("x",             "Cut (move) to clipboard",                 key_style, desc_style),
        row("p",             "Paste clipboard here",                    key_style, desc_style),
        Line::from(""),
        Line::from(Span::styled("  Favorites", header)),
        Line::from(Span::styled("  ──────────────────────────────────────────", dim)),
        row("f",             "Add selected to Favorites",               key_style, desc_style),
        row("D  (Fav panel)","Remove from Favorites",                   key_style, desc_style),
        row("Enter (Fav)",   "Navigate to favorited item",              key_style, desc_style),
        Line::from(""),
        Line::from(Span::styled("  Search", header)),
        Line::from(Span::styled("  ──────────────────────────────────────────", dim)),
        row("/",             "Start search / filter",                   key_style, desc_style),
        row("Esc",           "Cancel search",                           key_style, desc_style),
        Line::from(""),
        Line::from(Span::styled("  Global", header)),
        Line::from(Span::styled("  ──────────────────────────────────────────", dim)),
        row("Tab",           "Switch focus: Files → Drives → Favorites",key_style, desc_style),
        row("Ctrl+H",        "Focus Drives panel",                      key_style, desc_style),
        row("Ctrl+L",        "Focus Files panel",                       key_style, desc_style),
        row("Ctrl+B",        "Toggle status bar",                       key_style, desc_style),
        row("F5",            "Hot-reload config",                       key_style, desc_style),
        row("?",             "Toggle this help popup",                  key_style, desc_style),
        row("q",             "Quit",                                    key_style, desc_style),
        Line::from(""),
        Line::from(Span::styled("  Press ? or Esc to close", dim)),
    ];

    let help = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" ⌨  Keyboard Shortcuts ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(accent).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(bg_color)),
        )
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(help, popup_area);
}

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut s = size as f64;
    let mut i = 0;
    while s >= 1024.0 && i < UNITS.len() - 1 {
        s /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{} B", size)
    } else {
        format!("{:.1} {}", s, UNITS[i])
    }
}

fn format_date(modified: Result<std::time::SystemTime, std::io::Error>) -> String {
    match modified {
        Ok(time) => {
            let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
            let total_secs = duration.as_secs();
            let days = total_secs / 86400;

            let mut y = 1970i64;
            let mut remaining = days as i64;

            loop {
                let days_in_year = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 366 } else { 365 };
                if remaining < days_in_year {
                    break;
                }
                remaining -= days_in_year;
                y += 1;
            }

            let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
            let months_days: &[i64] = if leap {
                &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
            } else {
                &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
            };

            let mut m = 1i64;
            for &md in months_days {
                if remaining < md {
                    break;
                }
                remaining -= md;
                m += 1;
            }

            let d = remaining + 1;
            format!("{:04}-{:02}-{:02}", y, m, d)
        }
        Err(_) => String::new(),
    }
}

// Хелпер для центрирования попапов
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
