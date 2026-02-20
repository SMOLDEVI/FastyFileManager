use crate::app::{App, Focus, InputMode};
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

    // Парсим цвета из конфига
    let bg_color = parse_color(&theme.background);
    let text_color = parse_color(&theme.text);
    let sel_bg = parse_color(&theme.selected_bg);
    let sel_fg = parse_color(&theme.selected_fg);

    // Получаем базовые цвета для файлов/папок из конфига (Fixes warning "field never read")
    let dir_base_color = parse_color(&theme.directory);
    let file_base_color = parse_color(&theme.file);

    // Цвета границ для активного и неактивного окна
    let active_border_color = sel_bg;
    let inactive_border_color = Color::DarkGray;

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
    // 20% Диски (Слева)
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

    // --- 1. ПАНЕЛЬ ДИСКОВ (СЛЕВА) ---
    let drive_items: Vec<ListItem> = app
        .drives
        .iter()
        .map(|drive| {
            // Иконка диска из Nerd Font
            ListItem::new(format!("󰉉 {}", drive))
                .style(Style::default().fg(text_color).bg(bg_color))
        })
        .collect();

    // Стиль границ зависит от фокуса
    let drive_border_style = if app.focus == Focus::DriveList {
        Style::default()
            .fg(active_border_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(inactive_border_color)
    };

    // Стиль выделения (активен только если фокус здесь)
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

    f.render_stateful_widget(drive_list, main_chunks[0], &mut app.drive_state);

    // --- 2. ПАНЕЛЬ ФАЙЛОВ (ЦЕНТР) ---
    let file_items: Vec<ListItem> = app
        .filtered_items
        .iter()
        .map(|path| {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let icon = get_icon(path);

            // ЛОГИКА ЦВЕТОВ:
            // 1. Папка -> цвет из конфига (theme.directory)
            // 2. Файл -> цвет из иконки (icons.rs). Если нет -> цвет из конфига (theme.file)
            let color = if path.is_dir() {
                dir_base_color
            } else {
                get_icon_color(path).unwrap_or(file_base_color)
            };

            let content = format!("{} {}", icon, name);
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

    // --- 3. ПАНЕЛЬ ПРЕВЬЮ (СПРАВА) ---
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
    let mode_text = match app.input_mode {
        InputMode::Normal => match app.focus {
            Focus::FileList => "FILES [h -> Drives]",
            Focus::DriveList => "DRIVES l -> Files]",
        },
        InputMode::Editing => "EDITING",
        InputMode::Search => "SEARCHING",
    };

    let keys_hint = match app.input_mode {
        InputMode::Normal => format!(
            "[a]New [D]Del [{}]Search [F5]Reload",
            app.config.keys.search
        ),
        InputMode::Editing => "[Enter]Save [Esc]Cancel".to_string(),
        InputMode::Search => "[Enter]Go [Esc]Cancel".to_string(),
    };

    let status_text = format!(" {} | {} | {}", mode_text, keys_hint, app.message);
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
            .title(" Name ")
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
