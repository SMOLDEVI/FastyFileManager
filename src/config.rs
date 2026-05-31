use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Clone)]
#[serde(default = "Config::default")]
pub struct Config {
    pub theme: ThemeConfig,
    pub keys: KeysConfig,
}

#[derive(Deserialize, Clone)]
#[serde(default = "ThemeConfig::default_theme")]
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
#[serde(default = "KeysConfig::default_keys")]
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
    pub reload: String,
    pub edit: String,
    pub rename: String,
    pub help: String,
    pub sort: String,
}

impl ThemeConfig {
    fn default_theme() -> Self {
        ThemeConfig {
            background: "Reset".to_string(),
            text: "#EADBB8".to_string(),
            selected_bg: "#D2B48C".to_string(),
            selected_fg: "#282828".to_string(),
            directory: "#E0C097".to_string(),
            file: "#C8B6A6".to_string(),
            highlight_symbol: "> ".to_string(),
        }
    }
}

impl KeysConfig {
    fn default_keys() -> Self {
        KeysConfig {
            quit: "q".to_string(),
            search: "/".to_string(),
            cancel: "esc".to_string(),
            submit: "l".to_string(),
            down: "j".to_string(),
            up: "k".to_string(),
            delete: "D".to_string(),
            create: "a".to_string(),
            focus_files: "ctrl-l".to_string(),
            focus_drives: "ctrl-h".to_string(),
            back_dir: "h".to_string(),
            reload: "F5".to_string(),
            edit: "e".to_string(),
            rename: "r".to_string(),
            help: "?".to_string(),
            sort: "s".to_string(),
        }
    }
}

impl Config {
    const DEFAULT_TOML: &'static str = r##"[theme]
background = "Reset"
text = "#EADBB8"
selected_bg = "#D2B48C"
selected_fg = "#282828"
directory = "#E0C097"
file = "#C8B6A6"
highlight_symbol = "> "

[keys]
quit = "q"
search = "/"
cancel = "esc"
submit = "l"
down = "j"
up = "k"
delete = "D"
create = "a"
focus_files = "ctrl-l"
focus_drives = "ctrl-h"
back_dir = "h"
reload = "F5"
edit = "e"
rename = "r"
help = "?"
sort = "s"
"##;

    /// "Умная" загрузка конфига
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let mut paths_to_check = Vec::new();

        // 1. Приоритет локальному config.toml (если он лежит прямо рядом с программой)
        paths_to_check.push(PathBuf::from("config.toml"));
        
        if let Ok(exe_path) = env::current_exe()
            && let Some(exe_dir) = exe_path.parent() {
                paths_to_check.push(exe_dir.join("config.toml"));
            }

        // 2. Системная директория (AppData/Roaming/ffm на Windows, ~/.config/ffm на Linux)
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "ffm") {
            let config_dir = proj_dirs.config_dir();
            let config_path = config_dir.join("config.toml");
            
            // Если конфига в системе еще нет — заботливо создаем его с дефолтными настройками
            if !config_path.exists()
                && fs::create_dir_all(config_dir).is_ok() {
                    let _ = fs::write(&config_path, Self::DEFAULT_TOML);
                }
            
            // Добавляем системный путь в список проверок как запасной вариант (или основной для пользователя)
            paths_to_check.push(config_path);
        }

        for path in paths_to_check {
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(config_str) => {
                        match toml::from_str(&config_str) {
                            Ok(config) => return Ok(config),
                            Err(e) => return Err(format!("Parse error in {:?}: {}", path.file_name().unwrap_or_default(), e).into()),
                        }
                    }
                    Err(e) => return Err(format!("Read error {:?}: {}", path.file_name().unwrap_or_default(), e).into()),
                }
            }
        }

        Err("No config.toml found in any standard location".into())
    }

    pub fn default() -> Self {
        Config {
            theme: ThemeConfig::default_theme(),
            keys: KeysConfig::default_keys(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn debug_config_load() {
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "ffm") {
            let config_dir = proj_dirs.config_dir();
            println!("Config dir: {:?}", config_dir);
            let config_path = config_dir.join("config.toml");
            println!("Config path: {:?}", config_path);
            println!("Exists before: {}", config_path.exists());
            if !config_path.exists() {
                let res1 = fs::create_dir_all(config_dir);
                println!("create_dir_all result: {:?}", res1);
                let res2 = fs::write(&config_path, Config::DEFAULT_TOML);
                println!("write result: {:?}", res2);
            }
        }
        
        let res = Config::load();
        println!("Load result: {:?}", res.map(|_| "OK").map_err(|e| e.to_string()));
        panic!("Show output");
    }
}
