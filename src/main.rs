// Windows subsystem to hide console window | 콘솔 창 숨기기
#![windows_subsystem = "windows"]

use muda::{IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu, MenuId, ContextMenu};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use tao::platform::windows::WindowExtWindows;
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use windows::{
    core::*, 
    Win32::UI::WindowsAndMessaging::*,
    Win32::Foundation::{POINT, HWND, BOOL, WPARAM, LPARAM},
    Win32::System::Threading::GetCurrentThreadId,
    Win32::UI::Input::KeyboardAndMouse::*,
    Win32::UI::Input::KeyboardAndMouse::{SetFocus, SetActiveWindow},
};
use single_instance::SingleInstance;
use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}, GlobalHotKeyEvent};

mod localization; // Localization module | 번역 모듈
use localization::LocalizedStrings;

use std::time::SystemTime;
use chrono::Local;

// Static Menu IDs | 고정 메뉴 ID
const MENU_ID_EDIT: &str = "menu_edit_env";
const MENU_ID_RELOAD: &str = "menu_reload";
const MENU_ID_EXIT: &str = "menu_exit";

// App. Version History
// - 251221a: QikMenu 호출하는 단축키 정의 추가하여 단축키를 누를 경우 메뉴가 바로 뜨도록 한다.
// - 251215a: 폴더명이나 파일명에 공백이 포함된 경우 따옴표(")로 묶어주면 해당 명령어는 하나로 인식하도록 함
// - 251208b: 첫 릴리즈
const APP_VERSION: &str = "251221a";

// Function: Load Config | 환경 설정 로드 함수
fn load_config(ini_path: &Path) -> (String, Vec<(String, String)>, String) {
    let contents = fs::read_to_string(ini_path).unwrap_or_default();
    
    let mut locale = "ko".to_string();
    let mut current_section = "".to_string();
    let mut short_key = "".to_string();
    let mut app_entries: Vec<(String, String)> = Vec::new();

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len()-1].to_lowercase();
            continue;
        }

        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            if !key.is_empty() && !value.is_empty() {
                if current_section == "global" {
                    if key.eq_ignore_ascii_case("locale") {
                        locale = value.to_lowercase();
                    }
                } else if current_section == "env" {
                    if key.eq_ignore_ascii_case("short_key") {
                        short_key = value.to_string();
                    }
                } else if current_section == "apps" {
                    app_entries.push((key.to_string(), value.to_string()));
                }
            }
        }
    }
    (locale, app_entries, short_key)
}

// Function: Parse Hotkey String | 단축키 문자열 파싱
fn parse_hotkey(hotkey_str: &str) -> Option<HotKey> {
    if hotkey_str.is_empty() {
        return None;
    }

    let mut mods = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    // Split by '+' but handle '++' (PLUS key)
    // A simple split strategy:
    // 1. Identify modifiers like [Alt], [Ctrl], etc.
    // 2. Identify the main key.
    
    // Naive parsing: split by '+' might break '++'. 
    // Let's iterate manually or pre-process.
    // If we replace "++" with "+PLUS", then split by '+', it might work?
    // "[Alt]++" -> "[Alt]+PLUS" -> ["[Alt]", "PLUS"]
    
    let temp_str = hotkey_str.replace("++", "+Plus");
    let parts: Vec<&str> = temp_str.split('+').collect();

    for part in parts {
        let p = part.trim();
        match p.to_lowercase().as_str() {
            // Modifiers
            "[alt]" => mods |= Modifiers::ALT,
            "[ctrl]" => mods |= Modifiers::CONTROL,
            "[shift]" => mods |= Modifiers::SHIFT,
            "[win]" | "[meta]" => mods |= Modifiers::META,
            
            // Special Keys
            "[space]" => key_code = Some(Code::Space),
            "[tab]" => key_code = Some(Code::Tab),
            "[enter]" | "[return]" => key_code = Some(Code::Enter),
            "[backspace]" | "[back]" => key_code = Some(Code::Backspace),
            "[delete]" | "[del]" => key_code = Some(Code::Delete),
            "[esc]" | "[escape]" => key_code = Some(Code::Escape),
            "[up]" => key_code = Some(Code::ArrowUp),
            "[down]" => key_code = Some(Code::ArrowDown),
            "[left]" => key_code = Some(Code::ArrowLeft),
            "[right]" => key_code = Some(Code::ArrowRight),
            
            // F-Keys
            "[f1]" => key_code = Some(Code::F1),
            "[f2]" => key_code = Some(Code::F2),
            "[f3]" => key_code = Some(Code::F3),
            "[f4]" => key_code = Some(Code::F4),
            "[f5]" => key_code = Some(Code::F5),
            "[f6]" => key_code = Some(Code::F6),
            "[f7]" => key_code = Some(Code::F7),
            "[f8]" => key_code = Some(Code::F8),
            "[f9]" => key_code = Some(Code::F9),
            "[f10]" => key_code = Some(Code::F10),
            "[f11]" => key_code = Some(Code::F11),
            "[f12]" => key_code = Some(Code::F12),
            
            // Specific for "++" case handled above
            "plus" => key_code = Some(Code::Equal), // Usually '+' is on Equal key or NumpadAdd. 
                                                    // 'Code::Equal' is standard '=' key which is '+' with Shift.
                                                    // 'Code::NumpadAdd' is keypad +.
                                                    // Let's assume standard keyboard '+' (Shift+=). 
                                                    // But global-hotkey Code maps to physical keys.
                                                    // Code::Equal is the key next to Backspace.
                                                    // If user means keypad +, it's NumpadAdd.
                                                    // Based on request "++", it likely means the '+' character key.

            // Single Characters
            s if s.len() == 1 => {
                let c = s.chars().next().unwrap();
                if c.is_ascii_alphabetic() {
                    // Map A-Z to Code::KeyA...
                     match c {
                        'a' => key_code = Some(Code::KeyA),
                        'b' => key_code = Some(Code::KeyB),
                        'c' => key_code = Some(Code::KeyC),
                        'd' => key_code = Some(Code::KeyD),
                        'e' => key_code = Some(Code::KeyE),
                        'f' => key_code = Some(Code::KeyF),
                        'g' => key_code = Some(Code::KeyG),
                        'h' => key_code = Some(Code::KeyH),
                        'i' => key_code = Some(Code::KeyI),
                        'j' => key_code = Some(Code::KeyJ),
                        'k' => key_code = Some(Code::KeyK),
                        'l' => key_code = Some(Code::KeyL),
                        'm' => key_code = Some(Code::KeyM),
                        'n' => key_code = Some(Code::KeyN),
                        'o' => key_code = Some(Code::KeyO),
                        'p' => key_code = Some(Code::KeyP),
                        'q' => key_code = Some(Code::KeyQ),
                        'r' => key_code = Some(Code::KeyR),
                        's' => key_code = Some(Code::KeyS),
                        't' => key_code = Some(Code::KeyT),
                        'u' => key_code = Some(Code::KeyU),
                        'v' => key_code = Some(Code::KeyV),
                        'w' => key_code = Some(Code::KeyW),
                        'x' => key_code = Some(Code::KeyX),
                        'y' => key_code = Some(Code::KeyY),
                        'z' => key_code = Some(Code::KeyZ),
                        _ => {}
                    }
                } else if c.is_numeric() {
                     match c {
                        '1' => key_code = Some(Code::Digit1),
                        '2' => key_code = Some(Code::Digit2),
                        '3' => key_code = Some(Code::Digit3),
                        '4' => key_code = Some(Code::Digit4),
                        '5' => key_code = Some(Code::Digit5),
                        '6' => key_code = Some(Code::Digit6),
                        '7' => key_code = Some(Code::Digit7),
                        '8' => key_code = Some(Code::Digit8),
                        '9' => key_code = Some(Code::Digit9),
                        '0' => key_code = Some(Code::Digit0),
                        _ => {}
                     }
                } else {
                    // Symbols
                    match c {
                        '/' => key_code = Some(Code::Slash),
                        '.' => key_code = Some(Code::Period),
                        ',' => key_code = Some(Code::Comma),
                        ';' => key_code = Some(Code::Semicolon),
                        '\'' => key_code = Some(Code::Quote),
                        '[' => key_code = Some(Code::BracketLeft),
                        ']' => key_code = Some(Code::BracketRight),
                        '-' => key_code = Some(Code::Minus),
                        '=' => key_code = Some(Code::Equal),
                        '`' => key_code = Some(Code::Backquote),
                        '\\' => key_code = Some(Code::Backslash),
                        _ => {}
                    }
                }
            }
            _ => {
                log_msg("WARN", &format!("Unknown Key Part: {}", part));
            }
        }
    }

    if let Some(code) = key_code {
         Some(HotKey::new(Some(mods), code))
    } else {
        None
    }
}

// Helper: Force Window to Foreground
unsafe fn force_window_foreground(hwnd: HWND) {
    let foreground_window = GetForegroundWindow();
    let current_thread_id = GetCurrentThreadId();
    let foreground_thread_id = GetWindowThreadProcessId(foreground_window, None);
    
    // log_msg("INFO", &format!("Attempting Focus Steal: CurThread={}, ForeThread={}", current_thread_id, foreground_thread_id));

    // Try simple SetForegroundWindow first
    if SetForegroundWindow(hwnd).as_bool() {
        // log_msg("INFO", "Simple SetForegroundWindow Succeeded");
        BringWindowToTop(hwnd);
        return;
    }

    // Fallback to AttachThreadInput
    if current_thread_id != foreground_thread_id {
        // log_msg("INFO", "Simple failed. Trying AttachThreadInput...");
        let attached = windows::Win32::System::Threading::AttachThreadInput(foreground_thread_id, current_thread_id, BOOL(1));
        if attached.as_bool() {
             let set_res = SetForegroundWindow(hwnd);
             if set_res.as_bool() {
                 // log_msg("INFO", "SetForegroundWindow Success (Attached)");
                 SetActiveWindow(hwnd);
                 SetFocus(hwnd);
             } else {
                 log_msg("WARN", "SetForegroundWindow Failed (Attached)");
             }
             let _ = windows::Win32::System::Threading::AttachThreadInput(foreground_thread_id, current_thread_id, BOOL(0));
        } else {
             log_msg("WARN", "AttachThreadInput Failed");
             SetForegroundWindow(hwnd);
        }
    } else {
        // log_msg("INFO", "Already on Foreground Thread (but simple failed?)");
        SetForegroundWindow(hwnd);
        SetActiveWindow(hwnd);
        SetFocus(hwnd);
    }
    
    BringWindowToTop(hwnd);
    
    // Explicitly clear Alt key state by sending a dummy key up event?
    // Not implementing yet, risky.
}

// Function: Create Menu | 메뉴 생성 함수
fn create_menu(locale: &str, app_entries: &Vec<(String, String)>) -> (Menu, HashMap<String, String>) {
    let menu = Menu::new();
    let mut app_map: HashMap<String, String> = HashMap::new();
    let strings = LocalizedStrings::new(locale);
    
    // Add App Items
    for (key, value) in app_entries {
        let item = MenuItem::new(key, true, None);
        let _ = menu.append(&item);
        app_map.insert(item.id().as_ref().to_string(), value.clone());
    }
    
    let _ = menu.append(&PredefinedMenuItem::separator());

    // Static Items with fixed IDs
    let edit_env_item = MenuItem::with_id(MenuId::new(MENU_ID_EDIT), &strings.edit_environment, true, None);
    let _ = menu.append(&edit_env_item);
    
    let reload_item = MenuItem::with_id(MenuId::new(MENU_ID_RELOAD), &strings.reload, true, None);
    let _ = menu.append(&reload_item);
    
    let exit_item = MenuItem::with_id(MenuId::new(MENU_ID_EXIT), &strings.exit, true, None);
    let _ = menu.append(&exit_item);
    
    (menu, app_map)
}

// Function: Get Log Directory | 로그 디렉토리 가져오기
fn get_log_dir() -> PathBuf {
    let current_dir = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("."))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    
    let logs_dir = current_dir.join("logs");
    if !logs_dir.exists() {
        let _ = fs::create_dir(&logs_dir);
    }
    logs_dir
}

// Function: Write Log Message | 로그 메시지 기록
fn log_msg(level: &str, msg: &str) {
    let now = Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let time_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    
    let logs_dir = get_log_dir();
    let log_file_path = logs_dir.join(format!("{}.log", date_str));
    
    let log_entry = format!("[{}] [{}] {}\n", time_str, level, msg);
    
    // Append to file
    use std::io::Write;
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(log_file_path) {
        let _ = file.write_all(log_entry.as_bytes());
    }
}

// Function: Clean Old Logs (Older than 30 days) | 오래된 로그 삭제
fn clean_old_logs() {
    let logs_dir = get_log_dir();
    let now = SystemTime::now();
    let max_age = std::time::Duration::from_secs(30 * 24 * 60 * 60); // 30 days

    if let Ok(entries) = fs::read_dir(logs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age > max_age {
                                let _ = fs::remove_file(path);
                            }
                        }
                    }
                }
            }
        }
    }
}

// Function: Configure Command | 명령어 파싱 함수
// Splits string by spaces but respects quotes
fn parse_cmd(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in input.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c == ' ' && !in_quotes {
            if !current.is_empty() {
                args.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

fn main() {
    // 1. Logging Initialization
    clean_old_logs();
    log_msg("INFO", &format!("Application Started. Version: {}", APP_VERSION));

    let event_loop = EventLoopBuilder::new().build();

    // 4. Resolve INI Path
    // Look for QikMenu.ini in the same directory as the executable
    let ini_path = std::env::current_exe()
        .unwrap_or_else(|_| ".".into())
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("QikMenu.ini");

    // Initial Load
    let (mut locale, mut app_entries, mut short_key_str) = load_config(&ini_path);
    log_msg("INFO", &format!("Config Loaded. Locale: {}, Item Count: {}", locale, app_entries.len()));

    // Check Single Instance
    let instance = SingleInstance::new("QikMenu_Lock").unwrap();
    if !instance.is_single() {
        log_msg("WARN", "Another instance is already running.");
        let strings = LocalizedStrings::new(&locale);
        unsafe {
            let title_h = HSTRING::from(&strings.warning_title);
            let msg_h = HSTRING::from(&strings.warning_msg);
            MessageBoxW(None, &msg_h, &title_h, MB_OK | MB_ICONWARNING);
        }
        return;
    }

    let window = WindowBuilder::new()
        .with_visible(false)
        .with_decorations(false)
        .with_inner_size(tao::dpi::LogicalSize::new(0.0, 0.0))
        .with_position(tao::dpi::LogicalPosition::new(-10000.0, -10000.0))
        .build(&event_loop)
        .unwrap();

    // 5. Setup Hotkey
    let hotkey_manager = GlobalHotKeyManager::new().unwrap();
    let mut current_hotkey: Option<HotKey> = parse_hotkey(&short_key_str);

    if let Some(hk) = current_hotkey {
        if let Err(e) = hotkey_manager.register(hk) {
            log_msg("ERROR", &format!("Failed to register hotkey: {}", e));
        } else {
            log_msg("INFO", &format!("Hotkey registered: {}", short_key_str));
        }
    }

    // Build initial menu
    let (mut menu, mut app_map) = create_menu(&locale, &app_entries);

    // 6. Create Tray Icon | 트레이 아이콘 생성
    // Load Icon
    let icon_bytes = include_bytes!("../assets/tray_icon.png");
    let icon_image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = icon_image.dimensions();
    let rgba = icon_image.into_raw();
    
    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("QikMenu")
        .with_icon(tray_icon::Icon::from_rgba(rgba, width, height).expect("Failed to create icon"))
        .with_menu(Box::new(menu.clone()))
        .build()
        .unwrap();

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
    let hotkey_channel = GlobalHotKeyEvent::receiver();

    event_loop.run(move |_event, _, control_flow| {
        // Poll every 50ms to check channels (Hotkeys, Tray, Menu)
        // This is necessary because these channels do not wake up the TAO event loop on their own.
        *control_flow = ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_millis(50)
        );

        if let Ok(event) = menu_channel.try_recv() {
             let id = event.id.as_ref();
             log_msg("INFO", &format!("Menu Item Clicked: {}", id));

             if id == MENU_ID_EDIT {
                 let _ = open::that(&ini_path);
             } else if id == MENU_ID_RELOAD {
                 // Reload Logic
                 log_msg("INFO", "Reloading Configuration...");
                 let (new_locale, new_app_entries, new_short_key_str) = load_config(&ini_path);
                 let (new_menu, new_map) = create_menu(&new_locale, &new_app_entries);
                 
                 // Update Hotkey
                 if new_short_key_str != short_key_str {
                     if let Some(hk) = current_hotkey {
                         let _ = hotkey_manager.unregister(hk);
                     }
                     current_hotkey = parse_hotkey(&new_short_key_str);
                     if let Some(hk) = current_hotkey {
                         if let Err(e) = hotkey_manager.register(hk) {
                             log_msg("ERROR", &format!("Failed to register new hotkey: {}", e));
                         } else {
                             log_msg("INFO", &format!("New hotkey registered: {}", new_short_key_str));
                         }
                     }
                     short_key_str = new_short_key_str;
                 }
                 
                 // Update State
                 locale = new_locale;
                 app_entries = new_app_entries;
                 menu = new_menu;
                 app_map = new_map;
                 
                 // Update Tray Menu
                 let _ = tray_icon.set_menu(Some(Box::new(menu.clone())));
                 log_msg("INFO", "Configuration Reloaded.");
                 
             } else if id == MENU_ID_EXIT {
                 log_msg("INFO", "Exiting Application.");
                 *control_flow = ControlFlow::Exit;
             } else if let Some(cmd) = app_map.get(id) {
                 log_msg("INFO", &format!("Executing Command: {}", cmd));
                 let parts = parse_cmd(cmd);
                 if !parts.is_empty() {
                     let res = if parts.len() > 1 {
                         // Multiple parts: Execute as Command (Exe + Args)
                         std::process::Command::new(&parts[0])
                             .args(&parts[1..])
                             .spawn()
                             .map(|_| ())
                             .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                     } else {
                         // Single part: Use open::that (Supports URLs, Files, Folders)
                         open::that(&parts[0])
                     };

                     if let Err(e) = res {
                        let err_msg = format!("Execution Failed: {}", e);
                        eprintln!("{}", err_msg);
                        log_msg("ERROR", &err_msg);
                     } else {
                        log_msg("INFO", "Execution Triggered Successfully.");
                     }
                 }
             }
        }
        
        if let Ok(event) = tray_channel.try_recv() {
            // println!("{event:?}");
        }

        if let Ok(event) = hotkey_channel.try_recv() {
            if event.state == global_hotkey::HotKeyState::Pressed {
                 if let Some(hk) = current_hotkey {
                     if event.id == hk.id() {
                         log_msg("INFO", "Valid Hotkey Pressed. Processing...");
                         
                         // Drain multiple clicks
                         while let Ok(_) = hotkey_channel.try_recv() {}

                         // Show context menu at cursor
                         unsafe {
                             window.set_visible(true);
                             
                             let hwnd = HWND(window.hwnd() as _);
                             force_window_foreground(hwnd);
                             
                             // Reset any stuck menu state (e.g. from Alt key)
                             SendMessageW(hwnd, WM_CANCELMODE, WPARAM(0), LPARAM(0));
                             
                             log_msg("INFO", "Showing Menu...");
                             let _ = menu.show_context_menu_for_hwnd(
                                 window.hwnd() as isize, 
                                 None
                             );
                             log_msg("INFO", "Menu Closed (Event Loop Resuming)");
                             
                             window.set_visible(false);
                             
                             // Ensure we release focus/foreground cleanly (optional, but good practice)
                             // SetForegroundWindow(GetDesktopWindow()); 
                         }
                     }
                 } else {
                     log_msg("WARN", "Hotkey Pressed but ID mismatch or unknown");
                 }
            }
        }
    });
}
