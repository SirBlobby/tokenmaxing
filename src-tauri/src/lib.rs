mod agents;
mod history;
mod jsonio;
mod limits;
mod paths;
mod report;
mod settings;
mod sources;
mod stats;
mod tally;
mod timeline;
mod tray;

use agents::build_report;
use report::{RefreshOptions, UsageReport};
use settings::Settings;

#[tauri::command]
async fn usage_report(options: Option<RefreshOptions>) -> Result<UsageReport, String> {
    let options = options.unwrap_or_default();
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings::load_settings();
        build_report(options, &settings)
    })
    .await
    .map_err(|failure| failure.to_string())
}

#[tauri::command]
fn read_settings() -> Settings {
    settings::load_settings()
}

#[tauri::command]
fn write_settings(settings: Settings) -> Result<Settings, String> {
    settings::save_settings(&settings)?;
    Ok(settings)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            tray::create(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                tray::hide_to_tray(window);
            }
        })
        .invoke_handler(tauri::generate_handler![usage_report, read_settings, write_settings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
