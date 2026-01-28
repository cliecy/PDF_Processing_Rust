mod pdf_operations;

use pdf_operations::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            merge_pdfs,
            split_pdf,
            delete_pages,
            extract_pages,
            compress_pdf,
            pdf_to_images,
            images_to_pdf,
            get_pdf_info,
            rotate_pages,
            reorder_pages
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
