
// Windows subsystem to hide console window | 콘솔 창 숨기기
#![windows_subsystem = "windows"]

use muda::{IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu, MenuId};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use windows::{
    core::*, 
    Win32::UI::WindowsAndMessaging::*,
};
use single_instance::SingleInstance;

mod localization; // Localization module | 번역 모듈
use localization::LocalizedStrings;

// Static Menu IDs | 고정 메뉴 ID
const MENU_ID_EDIT: &str = "menu_edit_env";
const MENU_ID_RELOAD: &str = "menu_reload";
const MENU_ID_EXIT: &str = "menu_exit";

// Function: Load Config | 환경 설정 로드 함수
fn load_config(ini_path: &Path) -> (String, Vec<(String, String)>) {
    let contents = fs::read_to_string(ini_path).unwrap_or_default();
    
    let mut locale = "ko".to_string();
    let mut current_section = "".to_string();
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
                } else if current_section == "apps" {
                    app_entries.push((key.to_string(), value.to_string()));
                }
            }
        }
    }
    (locale, app_entries)
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

fn main() {
    let event_loop = EventLoopBuilder::new().build();

    // 4. Resolve INI Path
    // Look for QikMenu.ini in the same directory as the executable
    let ini_path = std::env::current_exe()
        .unwrap_or_else(|_| ".".into())
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("QikMenu.ini");

    // Initial Load
    let (mut locale, mut app_entries) = load_config(&ini_path);
    
    // Check Single Instance
    let instance = SingleInstance::new("QikMenu_Lock").unwrap();
    if !instance.is_single() {
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
        .build(&event_loop)
        .unwrap();

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

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Ok(event) = menu_channel.try_recv() {
             let id = event.id.as_ref();
             
             if id == MENU_ID_EDIT {
                 let _ = open::that(&ini_path);
             } else if id == MENU_ID_RELOAD {
                 // Reload Logic
                 let (new_locale, new_app_entries) = load_config(&ini_path);
                 let (new_menu, new_map) = create_menu(&new_locale, &new_app_entries);
                 
                 // Update State
                 locale = new_locale;
                 app_entries = new_app_entries;
                 menu = new_menu;
                 app_map = new_map;
                 
                 // Update Tray Menu
                 let _ = tray_icon.set_menu(Some(Box::new(menu.clone())));
                 
             } else if id == MENU_ID_EXIT {
                 *control_flow = ControlFlow::Exit;
             } else if let Some(cmd) = app_map.get(id) {
                 let cmd_clone = cmd.clone();
                 std::thread::spawn(move || {
                     if let Err(e) = open::that(cmd_clone) {
                        eprintln!("Failed to open: {}", e);
                     }
                 });
             }
        }
        
        if let Ok(event) = tray_channel.try_recv() {
            // println!("{event:?}");
        }
    });
}

