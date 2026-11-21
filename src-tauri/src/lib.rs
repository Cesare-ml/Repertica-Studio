mod backup;
mod commands;
mod db;
mod export;
mod finance;
mod gallery;
mod media_audit;
mod models;
mod receipts;
mod receipt_attachments;

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
            gallery::list_artwork_gallery,
            gallery::add_artwork_gallery_image,
            gallery::update_artwork_gallery_caption,
            gallery::promote_artwork_gallery_image,
            gallery::move_artwork_gallery_image,
            gallery::artwork_gallery_image_data,
            gallery::delete_artwork_gallery_image,
            media_audit::audit_artwork_media,
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
            receipts::list_sale_receipts,
            receipts::record_sale_receipt,
            receipts::delete_sale_receipt,
            receipts::list_sale_balances,
            receipts::receipt_year_overview,
            receipt_attachments::get_sale_receipt_attachment,
            receipt_attachments::attach_sale_receipt_file,
            receipt_attachments::sale_receipt_attachment_data,
            receipt_attachments::remove_sale_receipt_attachment,
            receipt_attachments::export_sale_receipt_attachment,
            export::export_ledger_csv
        ])
        .run(tauri::generate_context!())
        .expect("unable to start Repertica Studio");
}
