use ratatui::style::Color;
use std::str::FromStr;

pub fn parse_color(c: &str) -> Color {
    match c.to_lowercase().as_str() {
        "reset" | "none" => Color::Reset,
        "black" => Color::Black,
        "white" => Color::White,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" => Color::Gray,
        "darkgray" => Color::DarkGray,
        "lightblue" => Color::LightBlue,
        "lightgreen" => Color::LightGreen,
        "lightcyan" => Color::LightCyan,
        "lightred" => Color::LightRed,
        "lightmagenta" => Color::LightMagenta,
        "lightyellow" => Color::LightYellow,
        _ => Color::from_str(c).unwrap_or(Color::Reset),
    }
}
