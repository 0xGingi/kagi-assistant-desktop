use tauri_plugin_updater::UpdaterExt;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder, PredefinedMenuItem};
use tauri::Manager;

#[cfg(target_os = "windows")]
fn cleanup_old_uninstall_entries() {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
    use winreg::RegKey;

    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let expected_display_name = "Kagi Assistant";

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let uninstall_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall";
    let uninstall = match hkcu.open_subkey_with_flags(uninstall_path, KEY_READ | KEY_SET_VALUE) {
        Ok(k) => k,
        Err(_) => return,
    };

    let mut keys_to_delete: Vec<String> = Vec::new();

    for entry in uninstall.enum_keys() {
        let key_name = match entry { Ok(n) => n, Err(_) => continue };
        let sub = match uninstall.open_subkey_with_flags(&key_name, KEY_READ | KEY_SET_VALUE) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let display_name: String = sub.get_value("DisplayName").unwrap_or_default();
        if display_name != expected_display_name {
            continue;
        }

        let display_version: String = sub.get_value("DisplayVersion").unwrap_or_default();
        if display_version == current_version {
            continue;
        }

        let uninstall_string: String = sub
            .get_value::<String, _>("UninstallString")
            .unwrap_or_default()
            .to_lowercase();
        let install_location: String = sub
            .get_value::<String, _>("InstallLocation")
            .unwrap_or_default()
            .to_lowercase();
        let is_our_app = uninstall_string.contains("kagi") || install_location.contains("kagi");
        if !is_our_app {
            continue;
        }

        keys_to_delete.push(key_name);
    }

    for key in keys_to_delete {
        let _ = uninstall.delete_subkey_all(&key);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
    .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "reload" => {
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.eval("location.reload()");
                    }
                }
                "reload-f5" => {
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.eval("location.reload()");
                    }
                }
                _ => {}
            }
        })
        .on_window_event(|_window, event| match event {
            tauri::WindowEvent::CloseRequested { .. } => {
            }
            _ => {}
        })
        .setup(|app| {
            let reload = MenuItemBuilder::new("Reload")
                .id("reload")
                .accelerator("CmdOrCtrl+R")
                .build(app)?;
            let reload_f5 = MenuItemBuilder::new("Reload")
                .id("reload-f5")
                .accelerator("F5")
                .build(app)?;

            let view_menu = if cfg!(target_os = "macos") {
                SubmenuBuilder::new(app, "View")
                    .items(&[&reload])
                    .build()?
            } else {
                SubmenuBuilder::new(app, "View")
                    .items(&[&reload, &reload_f5])
                    .build()?
            };

            let undo = PredefinedMenuItem::undo(app, None)?;
            let redo = PredefinedMenuItem::redo(app, None)?;
            let cut = PredefinedMenuItem::cut(app, None)?;
            let copy = PredefinedMenuItem::copy(app, None)?;
            let paste = PredefinedMenuItem::paste(app, None)?;
            let select_all = PredefinedMenuItem::select_all(app, None)?;
            let edit_sep1 = PredefinedMenuItem::separator(app)?;
            let edit_sep2 = PredefinedMenuItem::separator(app)?;
            let edit_menu = SubmenuBuilder::new(app, "Edit")
                .items(&[&undo, &redo, &edit_sep1, &cut, &copy, &paste, &edit_sep2, &select_all])
                .build()?;

        let menu = if cfg!(target_os = "macos") {
                let about = PredefinedMenuItem::about(app, None, None)?;
                let services = PredefinedMenuItem::services(app, None)?;
                let hide = PredefinedMenuItem::hide(app, None)?;
                let hide_others = PredefinedMenuItem::hide_others(app, None)?;
                let show_all = PredefinedMenuItem::show_all(app, None)?;
                let quit = PredefinedMenuItem::quit(app, None)?;
                let sep1 = PredefinedMenuItem::separator(app)?;
                let sep2 = PredefinedMenuItem::separator(app)?;
                let sep3 = PredefinedMenuItem::separator(app)?;

                let app_submenu = SubmenuBuilder::new(app, "Kagi Assistant")
                    .items(&[&about, &sep1, &services, &sep2, &hide, &hide_others, &show_all, &sep3, &quit])
                    .build()?;

                MenuBuilder::new(app)
            .items(&[&app_submenu, &edit_menu, &view_menu])
                    .build()?
            } else {
                MenuBuilder::new(app)
            .items(&[&edit_menu, &view_menu])
                    .build()?
            };

            app.set_menu(menu)?;

            #[cfg(target_os = "windows")]
            cleanup_old_uninstall_entries();

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = update(handle).await {
                    eprintln!("Failed to check for updates: {}", e);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        let mut downloaded = 0;

        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    println!("downloaded {} from {:?}", downloaded, content_length);
                },
                || {
                    println!("download finished");
                },
            )
            .await?;

        println!("update installed");
        app.restart();
    }

    Ok(())
}
