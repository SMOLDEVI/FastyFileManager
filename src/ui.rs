use crate::app::{App, ClipboardOp, Focus, InputMode};
use crate::icons::{get_icon, get_icon_color};
use crate::theme::parse_color;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
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
    let favorites_border_color = Color::Rgb(150, 120, 200); // фиолетовый акцент для избранного

    let area = f.area();

    // Главный фон
    let main_block = Block::default().style(Style::default().bg(bg_color));
    f.render_widget(main_block, area);

    // Вертикальное деление: Основной контент + Статус бар
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    // Горизонтальное деление:
    // 20% Левая панель (Диски + Избранное)
    // 40% Список файлов (Центр)
    // 40% Превью (Справа)
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
            Constraint::Percentage(50), // Избранное
            Constraint::Percentage(50), // Диски
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

            // Помечаем файл в буфере обмена
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

            // Иконка "звёздочка" если в избранном
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
        .title(" Preview ")
        .border_style(Style::default().fg(inactive_border_color))
        .style(Style::default().bg(bg_color));

    let preview_text = Paragraph::new(app.preview_content.clone())
        .block(preview_block)
        .style(Style::default().fg(text_color))
        .wrap(Wrap { trim: false });

    f.render_widget(preview_text, main_chunks[2]);

    // --- ФУТЕР ---
    let clipboard_hint = match &app.clipboard {
        Some((p, ClipboardOp::Copy)) => format!(
            " [Copy: {}]",
            p.file_name().unwrap_or_default().to_string_lossy()
        ),
        Some((p, ClipboardOp::Cut)) => format!(
            " [Cut: {}]",
            p.file_name().unwrap_or_default().to_string_lossy()
        ),
        None => String::new(),
    };

    let mode_text = match app.input_mode {
        InputMode::Normal => match app.focus {
            Focus::FileList => "FILES",
            Focus::DriveList => "DRIVES",
            Focus::Favorites => "FAVORITES",
        },
        InputMode::Editing => "EDITING",
        InputMode::Search => "SEARCHING",
    };

    let keys_hint = match app.input_mode {
        InputMode::Normal => match app.focus {
            Focus::FileList => format!(
                "[hjkl/←↑↓→]Nav [a]New [D]Del [{}]Edit [y]Copy [x]Cut [p]Paste [f]Fav [/]Search [Tab]Switch",
                app.config.keys.edit
            ),
            Focus::DriveList => "[jk/↑↓]Nav [Enter]Open [Tab]Switch".to_string(),
            Focus::Favorites => "[jk/↑↓]Nav [Enter]Open [D/F]Remove [Tab]Switch".to_string(),
        },
        InputMode::Editing => "[Enter]Save [Esc]Cancel".to_string(),
        InputMode::Search => "[Enter]Confirm [Esc]Cancel".to_string(),
    };

    let status_text = format!(
        " {} | {}{} | {}",
        mode_text, keys_hint, clipboard_hint, app.message
    );
    let footer = Paragraph::new(status_text)
        .style(Style::default().fg(text_color).bg(bg_color))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .style(Style::default().bg(bg_color)),
        );

    f.render_widget(footer, vertical_chunks[1]);

    // --- POPUPS ---

    // Создание файла
    if let InputMode::Editing = app.input_mode {
        let area_rect = centered_rect(60, 20, area);
        f.render_widget(Clear, area_rect);

        let popup_block = Block::default()
            .title(" New file/folder (end name with / for folder) ")
            .borders(Borders::ALL)
            .style(Style::default().bg(bg_color).fg(text_color));

        let input_text = Paragraph::new(app.input_buffer.clone())
            .style(Style::default().fg(Color::Yellow))
            .block(popup_block);

        f.render_widget(input_text, area_rect);
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
