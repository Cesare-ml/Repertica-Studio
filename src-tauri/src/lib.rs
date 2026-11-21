mod allocations;
mod archive_code;
mod auto_backup;
mod backup;
mod collectors;
mod commands;
pub mod db;
mod export;
mod finance;
mod gallery;
mod media_audit;
mod models;
mod receipts;
mod util;
pub mod mcp;
mod mcp_client;
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
            auto_backup::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_artworks,
            commands::save_artwork,
            commands::create_artwork_full,
            commands::delete_artwork,
            collectors::list_collectors,
            collectors::save_collector,
            collectors::create_collector_full,
            collectors::delete_collector,
            collectors::list_collector_images,
            collectors::add_collector_image_from_path,
            collectors::collector_image_data,
            collectors::delete_collector_image,
            commands::list_sales,
            commands::record_sale,
            commands::delete_sale,
            commands::list_certificates,
            commands::issue_certificate,
            commands::overview,
            commands::data_directory,
            commands::save_artwork_image,
            commands::artwork_image_data,
            commands::save_artwork_image_from_path,
            gallery::list_artwork_gallery,
            gallery::add_artwork_gallery_image,
            gallery::add_artwork_gallery_image_from_path,
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
            export::export_ledger_csv,
            mcp::mcp_setup_info,
            mcp_client::mcp_client_status,
            mcp_client::mcp_client_configure,
            mcp_client::mcp_client_remove,
            auto_backup::get_backup_schedule,
            auto_backup::save_backup_schedule,
            auto_backup::run_backup_now,
            allocations::get_allocation_plan,
            allocations::save_allocation_plan,
            archive_code::get_archive_code_policy,
            archive_code::save_archive_code_policy,
            archive_code::next_archive_code
        ])
        .run(tauri::generate_context!())
        .expect("unable to start Repertica Studio");
}
