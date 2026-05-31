use ratatui::style::Color;
use std::path::Path;

pub fn get_icon(path: &Path, is_dir: bool) -> &'static str {
    if is_dir {
        return "";
    }
    match path.extension().and_then(|e| e.to_str()) {
        // Языки программирования
        Some("rs") => "",
        Some("py") => "",
        Some("js") => "",
        Some("ts") => "",
        Some("tsx") => "",
        Some("jsx") => "",
        Some("go") => "",
        Some("java") => "",
        Some("class") => "",
        Some("rb") => "",
        Some("c") => "",
        Some("cpp") | Some("cxx") | Some("cc") => "",
        Some("h") | Some("hpp") => "",
        Some("cs") => "󰌛",
        Some("swift") => "",
        Some("kt") | Some("kts") => "",
        Some("scala") => "",
        Some("php") => "",
        Some("lua") => "",
        Some("r") => "ﳒ",
        Some("dart") => "",
        Some("elm") => "",
        Some("erl") => "",
        Some("hs") => "",
        Some("ex") | Some("exs") => "",
        Some("clj") | Some("cljs") | Some("cljc") => "",
        Some("fst") | Some("fs") | Some("fsx") => "",

        // Скрипты / shell
        Some("sh") | Some("bash") => "",
        Some("zsh") => "",
        Some("fish") => "",
        Some("ps1") | Some("psm1") => "󰨊",
        Some("bat") | Some("cmd") => "",

        // Веб / фронтенд
        Some("html") => "",
        Some("css") => "",
        Some("scss") | Some("sass") => "",
        Some("less") => "",
        Some("vue") => "󰡄",
        Some("svelte") => "",
        Some("astro") => "",

        // Конфиги / данные
        Some("json") => "",
        Some("toml") => "",
        Some("yaml") | Some("yml") => "",
        Some("xml") => "󰗀",
        Some("ini") | Some("cfg") | Some("conf") => "",
        Some("env") => "",
        Some("lock") => "",
        Some("sql") => "󰆼",
        Some("db") | Some("sqlite") => "",

        // Документы
        Some("md") => "",
        Some("rst") => "",
        Some("txt") => "",
        Some("pdf") => "",
        Some("doc") | Some("docx") => "",
        Some("xls") | Some("xlsx") => "",
        Some("ppt") | Some("pptx") => "",
        Some("csv") | Some("tsv") => "",
        Some("epub") => "",

        // Изображения
        Some("png") | Some("jpg") | Some("jpeg") => "",
        Some("gif") => "",
        Some("bmp") => "",
        Some("svg") => "󰜡",
        Some("ico") => "",
        Some("webp") => "",

        // Аудио
        Some("mp3") => "",
        Some("wav") => "",
        Some("flac") => "",
        Some("ogg") => "",
        Some("m4a") => "",
        Some("aac") => "",
        Some("wma") => "",

        // Видео
        Some("mp4") => "",
        Some("avi") => "",
        Some("mkv") => "",
        Some("mov") => "",
        Some("wmv") => "",
        Some("webm") => "",

        // Архивы
        Some("zip") | Some("tar") => "",
        Some("gz") | Some("bz2") | Some("xz") => "",
        Some("7z") => "",
        Some("rar") => "",
        Some("zst") => "",

        // Исполняемые / пакеты
        Some("exe") | Some("msi") => "",
        Some("deb") => "󰁔",
        Some("rpm") => "󰓷",
        Some("AppImage") => "",
        Some("dmg") => "",
        Some("apk") => "󱛎",

        // Системные / прочее
        Some("iso") | Some("img") => "",
        Some("log") => "",
        Some("bak") | Some("swp") | Some("tmp") => "",
        Some("dockerfile") | Some("Dockerfile") => "󰡨",

        _ => {
            // Проверяем имя файла (без расширения)
            let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            match fname {
                "Dockerfile" | "dockerfile" => "󰡨",
                "Makefile" | "makefile" | "CMakeLists.txt" => "",
                "Cargo.toml" | "Cargo.lock" => "",
                "package.json" | "package-lock.json" => "",
                ".gitignore" | ".gitattributes" | ".gitmodules" => "",
                ".env" | ".env.example" => "",
                "LICENSE" | "LICENSE.txt" | "LICENSE.md" => "",
                "README.md" | "README.txt" | "README" => "",
                "compose.yml" | "compose.yaml" | "docker-compose.yml" => "󰡨",
                _ => "",
            }
        },
    }
}

pub fn get_icon_color(path: &Path) -> Option<Color> {
    match path.extension().and_then(|e| e.to_str()) {
        // Языки программирования
        Some("rs") => Some(Color::Red),
        Some("py") => Some(Color::LightBlue),
        Some("js") | Some("jsx") => Some(Color::Yellow),
        Some("ts") | Some("tsx") => Some(Color::Blue),
        Some("go") => Some(Color::Cyan),
        Some("java") | Some("class") => Some(Color::LightRed),
        Some("rb") => Some(Color::Red),
        Some("c") | Some("h") => Some(Color::Blue),
        Some("cpp") | Some("hpp") | Some("cc") | Some("cxx") => Some(Color::LightBlue),
        Some("cs") => Some(Color::Green),
        Some("swift") => Some(Color::Red),
        Some("kt") | Some("kts") => Some(Color::LightRed),
        Some("scala") => Some(Color::Red),
        Some("php") => Some(Color::LightBlue),
        Some("lua") => Some(Color::Blue),
        Some("r") => Some(Color::LightBlue),
        Some("dart") => Some(Color::Cyan),
        Some("elm") => Some(Color::Cyan),
        Some("erl") => Some(Color::Red),
        Some("hs") => Some(Color::Magenta),
        Some("ex") | Some("exs") => Some(Color::Magenta),
        Some("fst") | Some("fs") | Some("fsx") => Some(Color::Blue),

        // Скрипты
        Some("sh") | Some("bash") | Some("zsh") | Some("fish") => Some(Color::Green),
        Some("ps1") | Some("psm1") => Some(Color::Blue),
        Some("bat") | Some("cmd") => Some(Color::DarkGray),

        // Веб
        Some("html") => Some(Color::LightRed),
        Some("css") | Some("scss") | Some("sass") | Some("less") => Some(Color::Blue),
        Some("vue") => Some(Color::Green),
        Some("svelte") => Some(Color::Red),
        Some("astro") => Some(Color::Red),

        // Конфиги
        Some("json") | Some("yaml") | Some("yml") | Some("toml") => Some(Color::LightYellow),
        Some("xml") => Some(Color::Yellow),
        Some("ini") | Some("cfg") | Some("conf") => Some(Color::LightYellow),
        Some("env") => Some(Color::Yellow),
        Some("lock") => Some(Color::DarkGray),
        Some("sql") => Some(Color::LightBlue),
        Some("db") | Some("sqlite") => Some(Color::LightBlue),

        // Документы
        Some("md") | Some("rst") => Some(Color::Cyan),
        Some("txt") => Some(Color::Gray),
        Some("pdf") => Some(Color::Red),
        Some("doc") | Some("docx") => Some(Color::Blue),
        Some("xls") | Some("xlsx") => Some(Color::Green),
        Some("ppt") | Some("pptx") => Some(Color::LightRed),
        Some("csv") | Some("tsv") => Some(Color::Green),

        // Изображения
        Some("png") | Some("jpg") | Some("jpeg") | Some("bmp") | Some("webp") => Some(Color::Magenta),
        Some("gif") => Some(Color::Magenta),
        Some("svg") => Some(Color::Yellow),
        Some("ico") => Some(Color::Magenta),

        // Аудио
        Some("mp3") | Some("wav") | Some("flac") | Some("ogg") | Some("m4a") | Some("aac") | Some("wma") => Some(Color::Cyan),

        // Видео
        Some("mp4") | Some("avi") | Some("mkv") | Some("mov") | Some("wmv") | Some("webm") => Some(Color::LightMagenta),

        // Архивы
        Some("zip") | Some("tar") | Some("gz") | Some("bz2") | Some("xz") | Some("7z") | Some("rar") | Some("zst") => Some(Color::LightRed),

        // Исполняемые
        Some("exe") | Some("msi") | Some("dmg") | Some("AppImage") => Some(Color::Green),
        Some("deb") => Some(Color::LightBlue),
        Some("rpm") => Some(Color::LightRed),

        // Прочее
        Some("iso") | Some("img") => Some(Color::LightMagenta),
        Some("log") => Some(Color::DarkGray),
        Some("dockerfile") | Some("Dockerfile") => Some(Color::Blue),

        _ => None,
    }
}
