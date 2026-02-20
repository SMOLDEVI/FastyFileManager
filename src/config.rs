use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub theme: ThemeConfig,
    pub keys: KeysConfig,
}

#[derive(Deserialize, Clone)]
pub struct ThemeConfig {
    pub background: String,
    pub text: String,
    pub selected_bg: String,
    pub selected_fg: String,
    pub directory: String,
    pub file: String,
    pub highlight_symbol: String,
}

#[derive(Deserialize, Clone)]
pub struct KeysConfig {
    pub quit: String,
    pub search: String,
    pub cancel: String,
    pub submit: String,
    pub down: String,
    pub up: String,
    pub delete: String,
    pub create: String,
    pub focus_files: String,
    pub focus_drives: String,
    pub back_dir: String,
    pub reload: String, // Поле для кнопки перезагрузки
}

impl Config {
    /// "Умная" загрузка конфига
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let mut paths_to_check = Vec::new();

        // 1. Ищем рядом с исполняемым файлом (.exe)
        // Это самый надежный способ для скомпилированной программы
        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                paths_to_check.push(exe_dir.join("config.toml"));
            }
        }

        // 2. Ищем в текущей рабочей директории (откуда запустили команду)
        paths_to_check.push(PathBuf::from("config.toml"));

        // 3. Ищем на уровень выше (полезно при запуске через `cargo run` из корня проекта)
        paths_to_check.push(PathBuf::from("../config.toml"));

        for path in paths_to_check {
            if path.exists() {
                // Если нашли файл — пытаемся прочитать
                let config_str = fs::read_to_string(&path)?;
                let config: Config = toml::from_str(&config_str)?;
                return Ok(config);
            }
        }

        // Если нигде не нашли
        Err("config.toml not found in exe dir, current dir, or parent dir".into())
    }

    pub fn default() -> Self {
        toml::from_str(
            r#"
            [theme]
            background = "Reset"
            text = "White"
            selected_bg = "Blue"
            selected_fg = "Black"
            directory = "Cyan"
            file = "Gray"
            highlight_symbol = "> "

            [keys]
            quit = "q"
            search = "/"
            cancel = "esc"
            submit = "enter"
            down = "j"
            up = "k"
            delete = "D"
            create = "a"
            focus_files = "L"
            focus_drives = "H"
            back_dir = "backspace"
            reload = "F5"
        "#,
        )
        .unwrap()
    }
}
