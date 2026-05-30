use ratatui::style::Color;
use std::path::Path;

pub fn get_icon(path: &Path) -> &'static str {
    if path.is_dir() {
        return "";
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => "",
        Some("py") => "",
        Some("js") => "",
        Some("html") => "",
        Some("css") => "",
        Some("json") => "",
        Some("toml") => "",
        Some("md") => "",
        Some("png") | Some("jpg") => "",
        Some("zip") | Some("tar") => "",
        Some("txt") => "",
        Some("mp3") => "",
        Some("exe") => "",
        _ => "",
    }
}

// Возвращаем Option. Если None - используем цвет из конфига
pub fn get_icon_color(path: &Path) -> Option<Color> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => Some(Color::Red),
        Some("js") | Some("ts") => Some(Color::Yellow),
        Some("css") | Some("html") => Some(Color::Blue),
        Some("json") | Some("toml") => Some(Color::LightYellow),
        Some("png") | Some("jpg") => Some(Color::Magenta),
        Some("zip") | Some("tar") => Some(Color::LightRed),
        Some("py") => Some(Color::LightBlue),
        _ => None, // <--- Тут будем брать из конфига
    }
}
