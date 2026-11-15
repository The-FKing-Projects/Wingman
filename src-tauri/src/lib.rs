mod navigation;

use tauri::{WebviewUrl, WebviewWindowBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // GTK derives the Wayland app_id and X11 WM_CLASS from the program name;
    // it must match the bundled Wingman.desktop for taskbar and title-bar icons.
    #[cfg(target_os = "linux")]
    glib::set_prgname(Some("Wingman"));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Install the load-failure handler before navigating to Copilot.
            let mut config = app
                .config()
                .app
                .windows
                .iter()
                .find(|window| window.label == "main")
                .expect("main window config")
                .clone();
            let start_url = std::mem::replace(
                &mut config.url,
                WebviewUrl::External("about:blank".parse().unwrap()),
            );
            let builder = WebviewWindowBuilder::from_config(app.handle(), &config)?;
            let main_window = builder.build()?;
            let WebviewUrl::External(start_url) = start_url else {
                return Err("main window must have an external start URL".into());
            };
            #[cfg(target_os = "linux")]
            navigation::attach(&main_window, start_url)?;
            #[cfg(not(target_os = "linux"))]
            main_window.navigate(start_url)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
