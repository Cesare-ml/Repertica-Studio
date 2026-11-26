mod allocations;
mod archive_code;
mod auto_backup;
mod backup;
mod brochures;
mod business_mail;
mod cascade;
mod collectors;
mod commands;
pub mod db;
mod export;
mod finance;
mod gallery;
mod imaging;
mod installments;
mod media;
mod media_audit;
mod models;
mod raw_files;
mod receipts;
mod util;
pub mod mcp;
mod mcp_client;
mod receipt_attachments;
mod updater;
mod reveal;
mod mail_watch;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().pubkey(updater::PUBLIC_KEY).build())
        .manage(updater::UpdateState::default())
        .manage(mail_watch::MailWatch::default())
        .setup(|app| {
            use tauri::Manager;
            let directory = app.path().app_local_data_dir()?;
            backup::recover_interrupted_restore(&directory).map_err(std::io::Error::other)?;
            let database = db::open(directory)?;
            app.manage(database);
            auto_backup::start(app.handle().clone());
            mail_watch::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            business_mail::mail_list_accounts,
            business_mail::mail_save_account,
            business_mail::mail_remove_account,
            business_mail::mail_test_account,
            business_mail::mail_list_folders,
            business_mail::mail_list_messages,
            business_mail::mail_overview,
            business_mail::mail_read_message,
            business_mail::mail_set_seen,
            mail_watch::mail_sync_state,
            mail_watch::mail_sync_now,
            business_mail::mail_list_drafts,
            business_mail::mail_save_draft,
            business_mail::mail_remove_draft,
            business_mail::mail_send,
            brochures::list_brochures,
            brochures::save_brochure,
            brochures::delete_brochure,
            brochures::export_brochure_pdf,
            commands::list_artworks,
            commands::save_artwork,
            commands::create_artwork_full,
            commands::delete_artwork,
            cascade::artwork_deletion_impact,
            cascade::collector_deletion_impact,
            cascade::delete_certificate,
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
            commands::certificate_image_data,
            commands::save_artwork_image_from_path,
            gallery::list_artwork_gallery,
            gallery::add_artwork_gallery_image,
            gallery::add_artwork_gallery_image_from_path,
            gallery::update_artwork_gallery_caption,
            gallery::update_artwork_gallery_kind,
            reveal::reveal_in_folder,
            updater::check_for_update,
            updater::install_update,
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
            mcp_client::mcp_clients_status,
            mcp_client::mcp_client_configure,
            mcp_client::mcp_client_remove,
            media::media_list,
            media::media_detail,
            media::media_update,
            media::media_thumbnail,
            media::media_preview,
            media::media_file_preview,
            media::media_apply_edit,
            media::media_restore_revision,
            media::media_import,
            media::media_tags,
            media::media_create_tag,
            media::media_set_tags,
            media::media_collections,
            media::media_create_collection,
            media::media_delete_collection,
            media::media_collection_add,
            media::media_collection_remove,
            media::media_trash,
            media::media_untrash,
            media::media_purge,
            media::media_export,
            media::media_id_for_path,
            media::media_exact_duplicates,
            media::media_similar,
            media::artwork_set_cover_from_media,
            media::artwork_add_gallery_from_media,
            raw_files::list_artwork_raw_files,
            raw_files::attach_artwork_raw,
            raw_files::remove_artwork_raw,
            raw_files::export_artwork_raw,
            installments::list_sale_installments,
            installments::save_sale_installments,
            installments::generate_installment_plan,
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
