# FastyFileManager

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)
![TUI](https://img.shields.io/badge/interface-TUI-blue)
![License](https://img.shields.io/badge/license-MIT-green)
<img width="1893" height="931" alt="image" src="https://github.com/user-attachments/assets/84f5e0e3-59e8-49d9-b48d-d8859a9efbba" />

## 🚀 About

**FastyFileManager** — минималистичный и молниеносно быстрый файловый менеджер с текстовым интерфейсом (TUI), написанный на **Rust**.  
---

## ✨ Features

- 🦀 **Написан на Rust** — максимальная производительность и безопасность
- 🏎️ **Мгновенный запуск** — работает быстро даже на слабом железе
- 🖥️ **TUI-интерфейс** — полностью управляется с клавиатуры
- 💾 **Переключение между дисками** — быстрая навигация по всем разделам
- 📝 **Открытие в редакторе** — использует ваш `$EDITOR` по умолчанию
- 🎯 **Vim-like управление** — интуитивные горячие клавиши

---

## ⌨️ Controls

| Клавиша | Действие |
|---|---|
| `j` | Курсор вниз |
| `k` | Курсор вверх |
| `h` | Переключение на панель **Дисков** |
| `l` | Переключение на панель **Файлов** |
| `q` | Выход |
| `Enter` | Открыть файл / войти в директорию |
| `Backspace` | Назад (родительская директория) |

---

## 📦 Installation

### Из исходников

```bash
# Клонировать репозиторий
git clone https://github.com/your-username/FastyFileManager.git
cd FastyFileManager

# Собрать проект (требуется Rust 1.70+)
cargo build --release

# Бинарник будет в target/release/fastyfm
./target/release/fastyfm
```
# 🔧 Build Guide

Инструкция по сборке FastyFileManager для Windows и Linux.

---

## 📋 Requirements

- [Rust](https://rustup.rs/) 1.70+
- Cargo (устанавливается вместе с Rust)

```bash
# Проверить версию Rust
rustc --version
cargo --version

```
🐧 Linux
Базовая сборка
```Bash

# Клонировать репозиторий
git clone https://github.com/your-username/FastyFileManager.git
cd FastyFileManager

chmod +x build.sh
./build.sh

```

🪟 Windows
Базовая сборка
PowerShell
```bash
# Клонировать репозиторий
git clone https://github.com/your-username/FastyFileManager.git
cd FastyFileManager

# Сборка для разработки
cargo build

# Релизная сборка (оптимизированная)
cargo build --release

# Бинарник будет здесь:
.\target\release\ffm.exe
Установка в систему
PowerShell

# Вариант 1: через cargo
cargo install --path .

# Вариант 2: копирование вручную
copy target\release\ffm.exe C:\Windows\System32\

# Или добавить папку в PATH
$env:PATH += ";C:\path\to\FastyFileManager\target\release"
Сборка для разных архитектур
PowerShell
```


<p align="center"> Made with ❤️ and 🦀 Rust </p> 
