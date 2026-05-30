use crate::app::{App, ClipboardOp, Focus, InputMode};
use crate::icons::{get_icon, get_icon_color};
use crate::theme::parse_color;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

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

    // Горизонтальное деление: 20% | 40% | 40%
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
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
    let drive_items: Vec<ListItem> = app
        .drives
        .iter()
        .map(|drive| {
            ListItem::new(format!("󰉉 {}", drive))
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
    let file_items: Vec<ListItem> = app
        .filtered_items
        .iter()
        .map(|path| {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let icon = get_icon(path);

            let in_clipboard = app
                .clipboard
                .as_ref()
                .map(|(p, _)| p == path)
                .unwrap_or(false);

            let color = if path.is_dir() {
                dir_base_color
            } else {
                get_icon_color(path).unwrap_or(file_base_color)
            };

            let marker = if in_clipboard {
                match app.clipboard.as_ref().map(|(_, op)| op) {
                    Some(ClipboardOp::Copy) => " [C]",
                    Some(ClipboardOp::Cut) => " [X]",
                    None => "",
                }
            } else {
                ""
            };

            let fav_mark = if app.favorites.contains(path) { " ★" } else { "" };

            let content = format!("{} {}{}{}", icon, name, marker, fav_mark);
            ListItem::new(content).style(Style::default().fg(color).bg(bg_color))
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

    let list_title = if app.input_mode == InputMode::Search {
        format!(" Search: {} ", app.search_query)
    } else {
        format!(" {} ", app.current_dir.to_string_lossy())
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
            Some((p, ClipboardOp::Copy)) => format!(
                " │ 󰆏 Copy: {}",
                p.file_name().unwrap_or_default().to_string_lossy()
            ),
            Some((p, ClipboardOp::Cut)) => format!(
                " │ 󰆐 Cut: {}",
                p.file_name().unwrap_or_default().to_string_lossy()
            ),
            None => String::new(),
        };

        let mode_text = match app.input_mode {
            InputMode::Normal => match app.focus {
                Focus::FileList => " FILES",
                Focus::DriveList => " DRIVES",
                Focus::Favorites => "★ FAVORITES",
            },
            InputMode::Editing => " EDITING",
            InputMode::Search => " SEARCH",
        };

        let keys_hint = match app.input_mode {
            InputMode::Normal => match app.focus {
                Focus::FileList => format!(
                    "hjkl Nav │ a New │ D Del │ {} Edit │ y Copy │ x Cut │ p Paste │ f Fav │ / Search │ ? Help │ Ctrl+B Bar",
                    app.config.keys.edit
                ),
                Focus::DriveList => "jk Nav │ Enter Open │ Tab Switch │ ? Help │ Ctrl+B Bar".to_string(),
                Focus::Favorites => "jk Nav │ Enter Open │ D Remove │ Tab Switch │ ? Help │ Ctrl+B Bar".to_string(),
            },
            InputMode::Editing => "Enter Save │ Esc Cancel".to_string(),
            InputMode::Search => "Enter Confirm │ Esc Cancel │ ↑↓ Navigate".to_string(),
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

    // Попап помощи
    if app.show_help {
        render_help_popup(f, area, bg_color, text_color, sel_bg);
    }
}

fn render_help_popup(
    f: &mut Frame,
    area: Rect,
    bg_color: Color,
    text_color: Color,
    accent: Color,
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
        Line::from(""),
        Line::from(Span::styled("  File Operations", header)),
        Line::from(Span::styled("  ──────────────────────────────────────────", dim)),
        row("a",             "Create new file/folder (/ = folder)",     key_style, desc_style),
        row("D",             "Delete selected item",                    key_style, desc_style),
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
        .wrap(Wrap { trim: false });

    f.render_widget(help, popup_area);
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
