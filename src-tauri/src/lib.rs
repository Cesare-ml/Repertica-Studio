mod commands;
mod db;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            use tauri::Manager;
            let directory = app.path().app_local_data_dir()?;
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
            commands::artwork_image_data
        ])
        .run(tauri::generate_context!())
        .expect("unable to start Repertica Studio");
}
