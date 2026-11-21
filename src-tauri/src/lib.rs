mod backup;
mod commands;
mod db;
mod export;
mod finance;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            use tauri::Manager;
            let directory = app.path().app_local_data_dir()?;
            backup::recover_interrupted_restore(&directory).map_err(std::io::Error::other)?;
            let database = db::open(directory)?;
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_artworks,
            commands::save_artwork,
            commands::delete_artwork,
            commands::list_collectors,
            commands::save_collector,
            commands::delete_collector,
            commands::list_sales,
            commands::record_sale,
            commands::delete_sale,
            commands::list_certificates,
            commands::issue_certificate,
            commands::overview,
            commands::data_directory,
            commands::save_artwork_image,
            commands::artwork_image_data,
            backup::create_backup,
            backup::inspect_backup,
            backup::restore_backup,
            finance::get_studio_settings,
            finance::save_studio_settings,
            finance::list_expenses,
            finance::save_expense,
            finance::delete_expense,
            finance::finance_overview,
            finance::price_preview,
            export::export_ledger_csv
        ])
        .run(tauri::generate_context!())
        .expect("unable to start Repertica Studio");
}
