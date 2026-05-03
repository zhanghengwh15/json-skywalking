use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
    Manager, WindowEvent,
};

/// 设置系统托盘
pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // 创建系统托盘菜单
    let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let tray_menu = Menu::with_items(app, &[&show, &quit])?;

    // 设置系统托盘
    // show_menu_on_left_click(false) 避免左键释放时弹出菜单与 show/hide 冲突
    let mut tray_builder = TrayIconBuilder::new()
        .menu(&tray_menu)
        .tooltip("DevTools")
        .show_menu_on_left_click(false);

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
            // 只处理左键释放的 Click 事件；忽略 Down(按下) 和 DoubleClick，
            // 防止 Windows 同时触发多个事件导致窗口状态抖动
            let is_left_click_up = matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            );
            if !is_left_click_up {
                return;
            }

            let app = tray.app_handle();
            if let Some(window) = app.get_webview_window("main") {
                match window.is_visible() {
                    Ok(true) => {
                        let _ = window.hide();
                        #[cfg(target_os = "windows")]
                        {
                            let _ = window.set_skip_taskbar(true);
                        }
                    }
                    Ok(false) => {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = window.set_skip_taskbar(false);
                        }
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    Err(_) => {}
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
                let _ = window.set_skip_taskbar(true);
            }
            let _ = window.hide();
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