use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton},
    Manager, WindowEvent,
};

/// 设置系统托盘
pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // 创建系统托盘菜单
    let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let tray_menu = Menu::with_items(app, &[&show, &quit])?;

    // 设置系统托盘
    let mut tray_builder = TrayIconBuilder::new()
        .menu(&tray_menu)
        .tooltip("DevTools");

    // Windows 下如果不显式设置托盘图标，可能出现有占位但无图标的情况
    if let Some(icon) = app.default_window_icon() {
        tray_builder = tray_builder.icon(icon.clone());
    }

    let _tray = tray_builder
        .on_menu_event(move |app, event| {
            if let Some(window) = app.get_webview_window("main") {
                match event.id().as_ref() {
                    "show" => {
                        #[cfg(target_os = "windows")]
                        {
                            window.set_skip_taskbar(false).unwrap();
                        }
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let is_visible = window.is_visible().unwrap();
                    if is_visible {
                        window.hide().unwrap();
                        #[cfg(target_os = "windows")]
                        {
                            window.set_skip_taskbar(true).unwrap();
                        }
                    } else {
                        #[cfg(target_os = "windows")]
                        {
                            window.set_skip_taskbar(false).unwrap();
                        }
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// 处理窗口事件
pub fn handle_window_event(window: &tauri::Window, event: &WindowEvent) {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            #[cfg(target_os = "windows")]
            {
                window.set_skip_taskbar(true).unwrap();
            }
            window.hide().unwrap();
        }
        _ => {}
    }
}

/// 设置全局快捷键
#[cfg(desktop)]
pub fn setup_global_shortcut(app: &tauri::App) {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, GlobalShortcutExt};
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyQ);
    match app.global_shortcut().register(shortcut) {
        Ok(_) => println!("[GlobalShortcut] 快捷键注册成功: CTRL+ALT+Q"),
        Err(e) => eprintln!("[GlobalShortcut] 快捷键注册失败: {}", e),
    }
}

/// 处理应用运行事件
pub fn handle_run_event(_app_handle: &tauri::AppHandle, _event: &tauri::RunEvent) {
    #[cfg(target_os = "macos")]
    match _event {
        tauri::RunEvent::Reopen { .. } => {
            if let Some(window) = _app_handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        _ => {}
    }
} 