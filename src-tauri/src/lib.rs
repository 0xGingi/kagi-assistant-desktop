use tauri_plugin_updater::UpdaterExt;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder, PredefinedMenuItem};
use tauri::Manager;

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

            let view_menu = SubmenuBuilder::new(app, "View")
                .items(&[&reload, &reload_f5])
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
                    .items(&[&app_submenu, &view_menu])
                    .build()?
            } else {
                MenuBuilder::new(app)
                    .items(&[&view_menu])
                    .build()?
            };

            app.set_menu(menu)?;

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
