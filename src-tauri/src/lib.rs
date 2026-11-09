use tauri::Manager;
use url::Url;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();

            let url = Url::parse("https://copilot.com/chat")
                .expect("Invalid URL");

            main_window.navigate(url)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}