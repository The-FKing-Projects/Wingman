use tauri::{WebviewUrl, WebviewWindowBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let _copilot_window = WebviewWindowBuilder::new(
                app,
                "copilot",
                WebviewUrl::External(
                    "https://copilot.com/chat"
                        .parse()
                        .expect("Invalid URL"),
                ),
            )
            .title("Wingman")
            .inner_size(1200.0, 800.0)
            .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}