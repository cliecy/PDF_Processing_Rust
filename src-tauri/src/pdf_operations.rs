use lopdf::{Document, Object, ObjectId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
#[cfg(target_os = "linux")]
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PDF error: {0}")]
    Pdf(#[from] lopdf::Error),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

impl serde::Serialize for PdfError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfInfo {
    pub page_count: usize,
    pub file_size: u64,
    pub title: Option<String>,
    pub author: Option<String>,
    pub pdf_version: String,
    pub is_encrypted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResult {
    pub message: String,
    pub output_path: Option<String>,
}

async fn run_blocking<F, T>(task: F) -> Result<T, PdfError>
where
    F: FnOnce() -> Result<T, PdfError> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|error| PdfError::InvalidOperation(format!("Background task failed: {}", error)))?
}

fn pdf_metadata_string(doc: &Document, key: &[u8]) -> Option<String> {
    doc.trailer
        .get(b"Info")
        .ok()
        .and_then(|info| info.as_reference().ok())
        .and_then(|info_ref| doc.get_object(info_ref).ok())
        .and_then(|object| object.as_dict().ok())
        .and_then(|dictionary| dictionary.get(key).ok())
        .and_then(|value| match value {
            Object::String(bytes, _) => String::from_utf8(bytes.clone()).ok(),
            _ => None,
        })
}

fn load_pdf_info(file_path: &str) -> Result<PdfInfo, PdfError> {
    let path = Path::new(file_path);
    let metadata = fs::metadata(path)?;
    let doc = Document::load(path)?;

    Ok(PdfInfo {
        page_count: doc.get_pages().len(),
        file_size: metadata.len(),
        title: pdf_metadata_string(&doc, b"Title"),
        author: pdf_metadata_string(&doc, b"Author"),
        pdf_version: doc.version.clone(),
        is_encrypted: doc.trailer.get(b"Encrypt").is_ok(),
    })
}

/// Get PDF information
#[tauri::command]
pub async fn get_pdf_info(file_path: String) -> Result<PdfInfo, PdfError> {
    run_blocking(move || load_pdf_info(&file_path)).await
}

/// Open folder in system file explorer
#[tauri::command]
pub async fn open_folder(path: String) -> Result<(), PdfError> {
    let path = Path::new(&path);
    let folder = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(folder)
            .spawn()
            .map_err(|e| PdfError::InvalidOperation(format!("Failed to open folder: {}", e)))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(folder)
            .spawn()
            .map_err(|e| PdfError::InvalidOperation(format!("Failed to open folder: {}", e)))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(folder)
            .spawn()
            .map_err(|e| PdfError::InvalidOperation(format!("Failed to open folder: {}", e)))?;
    }

    Ok(())
}

fn merge_pdf_documents(file_paths: &[String]) -> Result<Document, PdfError> {
    if file_paths.is_empty() {
        return Err(PdfError::InvalidOperation(
            "No input files provided".to_string(),
        ));
    }

    let mut max_id = 1;
    let mut page_order = Vec::new();
    let document = Document::with_version("1.5");
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();

    for path in file_paths {
        let mut doc = Document::load(path)?;
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        for (_, page_id) in doc.get_pages() {
            page_order.push(page_id);
            documents_pages.insert(page_id, doc.get_object(page_id)?.to_owned());
        }

        documents_objects.extend(doc.objects);
    }

    build_document_from_parts(document, documents_objects, documents_pages, page_order)
}

fn build_document_from_parts(
    mut document: Document,
    documents_objects: BTreeMap<ObjectId, Object>,
    documents_pages: BTreeMap<ObjectId, Object>,
    page_order: Vec<ObjectId>,
) -> Result<Document, PdfError> {
    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in documents_objects {
        match object.type_name().unwrap_or("") {
            "Catalog" => {
                catalog_object = Some((
                    catalog_object.map(|(id, _)| id).unwrap_or(object_id),
                    object,
                ));
            }
            "Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();

                    if let Some((_, ref existing_object)) = pages_object {
                        if let Ok(existing_dictionary) = existing_object.as_dict() {
                            dictionary.extend(existing_dictionary);
                        }
                    }

                    pages_object = Some((
                        pages_object.map(|(id, _)| id).unwrap_or(object_id),
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            "Page" | "Outlines" | "Outline" => {}
            _ => {
                document.objects.insert(object_id, object);
            }
        }
    }

    let (pages_id, pages_root) = pages_object
        .ok_or_else(|| PdfError::InvalidOperation("Pages root not found".to_string()))?;
    let (catalog_id, catalog_root) = catalog_object
        .ok_or_else(|| PdfError::InvalidOperation("Catalog root not found".to_string()))?;

    for page_id in &page_order {
        let page_object = documents_pages.get(page_id).ok_or_else(|| {
            PdfError::InvalidOperation("Merged page object not found".to_string())
        })?;

        if let Ok(dictionary) = page_object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_id);
            document
                .objects
                .insert(*page_id, Object::Dictionary(dictionary));
        }
    }

    if let Ok(dictionary) = pages_root.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Count", page_order.len() as u32);
        dictionary.set(
            "Kids",
            page_order
                .iter()
                .copied()
                .map(Object::Reference)
                .collect::<Vec<_>>(),
        );
        document
            .objects
            .insert(pages_id, Object::Dictionary(dictionary));
    }

    if let Ok(dictionary) = catalog_root.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_id);
        dictionary.remove(b"Outlines");
        document
            .objects
            .insert(catalog_id, Object::Dictionary(dictionary));
    }

    document.trailer.set("Root", catalog_id);
    document.max_id = document
        .objects
        .keys()
        .map(|(id, _)| *id)
        .max()
        .unwrap_or(0);
    document.renumber_objects();

    Ok(document)
}

fn reordered_pdf_document(file_path: &str, new_order: &[u32]) -> Result<Document, PdfError> {
    let mut doc = Document::load(file_path)?;
    doc.renumber_objects();

    let page_map = doc.get_pages();
    let total_pages = page_map.len() as u32;

    if new_order.len() != total_pages as usize {
        return Err(PdfError::InvalidOperation(
            "New order must contain all page numbers".to_string(),
        ));
    }

    let mut sorted_order = new_order.to_vec();
    sorted_order.sort_unstable();
    let expected_order: Vec<u32> = (1..=total_pages).collect();

    if sorted_order != expected_order {
        return Err(PdfError::InvalidOperation(
            "New order must be a permutation of every page number exactly once".to_string(),
        ));
    }

    let mut page_numbers: Vec<u32> = page_map.keys().copied().collect();
    page_numbers.sort_unstable();

    let page_order: Vec<ObjectId> = new_order
        .iter()
        .map(|page_index| {
            let actual_page_number = page_numbers[(*page_index - 1) as usize];
            page_map.get(&actual_page_number).copied().ok_or_else(|| {
                PdfError::InvalidOperation("Requested page not found in PDF".to_string())
            })
        })
        .collect::<Result<_, _>>()?;

    let documents_pages = page_map
        .values()
        .copied()
        .map(|page_id| {
            Ok((
                page_id,
                doc.get_object(page_id)
                    .map(|object| object.to_owned())
                    .map_err(PdfError::from)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, PdfError>>()?;

    build_document_from_parts(
        Document::with_version("1.5"),
        doc.objects,
        documents_pages,
        page_order,
    )
}

fn apply_compression_profile(doc: &mut Document, level: u8) {
    doc.compress();

    if level >= 50 {
        doc.delete_zero_length_streams();
    }

    if level >= 75 {
        doc.prune_objects();
    }

    if level >= 90 {
        doc.renumber_objects();
    }
}

fn format_size_change_message(original_size: u64, new_size: u64) -> String {
    let original_kb = original_size / 1024;
    let new_kb = new_size / 1024;

    if original_size == 0 {
        return format!("Saved optimized PDF: {} KB -> {} KB", original_kb, new_kb);
    }

    match new_size.cmp(&original_size) {
        std::cmp::Ordering::Less => {
            let reduction =
                ((original_size - new_size) as f64 / original_size as f64 * 100.0).round() as u32;
            format!(
                "Compressed PDF: {} KB -> {} KB ({}% reduction)",
                original_kb, new_kb, reduction
            )
        }
        std::cmp::Ordering::Greater => {
            let increase =
                ((new_size - original_size) as f64 / original_size as f64 * 100.0).round() as u32;
            format!(
                "Saved optimized PDF: {} KB -> {} KB ({}% larger)",
                original_kb, new_kb, increase
            )
        }
        std::cmp::Ordering::Equal => {
            format!(
                "Saved optimized PDF: {} KB -> {} KB (no size change)",
                original_kb, new_kb
            )
        }
    }
}

fn build_pdf_to_images_prefix(base_name: &str) -> Result<String, PdfError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| PdfError::InvalidOperation(format!("System clock error: {}", e)))?
        .as_millis();

    Ok(format!("{}_{}", base_name, timestamp))
}

fn path_directories() -> Vec<PathBuf> {
    env::var_os("PATH")
        .map(|path| env::split_paths(&path).collect())
        .unwrap_or_default()
}

fn add_candidate_dir(dirs: &mut Vec<PathBuf>, dir: Option<PathBuf>) {
    if let Some(dir) = dir.filter(|dir| !dir.as_os_str().is_empty()) {
        if !dirs.iter().any(|existing| existing == &dir) {
            dirs.push(dir);
        }
    }
}

fn pdftoppm_candidate_names() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &["pdftoppm.exe", "pdftoppm.cmd", "pdftoppm.bat", "pdftoppm"]
    }
    #[cfg(not(target_os = "windows"))]
    {
        &["pdftoppm"]
    }
}

fn resolve_pdftoppm_from_dir(dir: &Path) -> Option<PathBuf> {
    for candidate_name in pdftoppm_candidate_names() {
        let candidate = dir.join(candidate_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

fn pdftoppm_search_dirs() -> Vec<PathBuf> {
    let mut dirs = path_directories();

    #[cfg(target_os = "windows")]
    {
        if let Some(scoop_root) = env::var_os("SCOOP").map(PathBuf::from) {
            add_candidate_dir(
                &mut dirs,
                Some(scoop_root.join("apps/poppler/current/Library/bin")),
            );
            add_candidate_dir(&mut dirs, Some(scoop_root.join("shims")));
        }

        if let Some(user_profile) = env::var_os("USERPROFILE").map(PathBuf::from) {
            add_candidate_dir(
                &mut dirs,
                Some(user_profile.join("scoop/apps/poppler/current/Library/bin")),
            );
            add_candidate_dir(&mut dirs, Some(user_profile.join("scoop/shims")));
        }

        if let Some(program_data) = env::var_os("ProgramData").map(PathBuf::from) {
            add_candidate_dir(
                &mut dirs,
                Some(program_data.join("scoop/apps/poppler/current/Library/bin")),
            );
            add_candidate_dir(&mut dirs, Some(program_data.join("scoop/shims")));
            add_candidate_dir(&mut dirs, Some(program_data.join("chocolatey/bin")));
            add_candidate_dir(
                &mut dirs,
                Some(program_data.join("chocolatey/lib/poppler/tools")),
            );
        }

        for var in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(program_files) = env::var_os(var).map(PathBuf::from) {
                add_candidate_dir(&mut dirs, Some(program_files.join("poppler/Library/bin")));
                add_candidate_dir(&mut dirs, Some(program_files.join("poppler/bin")));
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    for dir in [
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/opt/homebrew/opt/poppler/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/local/opt/poppler/bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/opt/local/bin"),
        PathBuf::from("/nix/var/nix/profiles/default/bin"),
    ] {
        add_candidate_dir(&mut dirs, Some(dir));
    }

    if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        add_candidate_dir(&mut dirs, Some(home.join(".nix-profile/bin")));
    }

    if let Some(conda_prefix) = env::var_os("CONDA_PREFIX").map(PathBuf::from) {
        add_candidate_dir(&mut dirs, Some(conda_prefix.join("bin")));
        add_candidate_dir(&mut dirs, Some(conda_prefix.join("Library/bin")));
        add_candidate_dir(&mut dirs, Some(conda_prefix.join("Scripts")));
    }

    dirs
}

fn resolve_brew_pdftoppm() -> Option<PathBuf> {
    for brew_path in ["/opt/homebrew/bin/brew", "/usr/local/bin/brew"] {
        let output = match Command::new(brew_path)
            .args(["--prefix", "poppler"])
            .output()
        {
            Ok(output) => output,
            Err(_) => continue,
        };

        if !output.status.success() {
            continue;
        }

        let prefix = String::from_utf8(output.stdout).ok()?;
        if let Some(candidate) =
            resolve_pdftoppm_from_dir(&PathBuf::from(prefix.trim()).join("bin"))
        {
            return Some(candidate);
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn resolve_windows_where_pdftoppm() -> Option<PathBuf> {
    let output = Command::new("where").arg("pdftoppm").output().ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .find(|candidate| candidate.is_file())
}

fn count_generated_image_files(
    output_dir: &Path,
    run_prefix: &str,
    ext: &str,
) -> Result<usize, PdfError> {
    let stem_prefix = format!("{run_prefix}-");

    Ok(fs::read_dir(output_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            let extension_matches = path
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| extension.eq_ignore_ascii_case(ext))
                .unwrap_or(false);
            let stem_matches = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(|stem| stem.starts_with(&stem_prefix))
                .unwrap_or(false);

            extension_matches && stem_matches
        })
        .count())
}

/// Merge multiple PDFs into one
#[tauri::command]
pub async fn merge_pdfs(
    file_paths: Vec<String>,
    output_path: String,
) -> Result<ProcessResult, PdfError> {
    run_blocking(move || {
        let mut merged_doc = merge_pdf_documents(&file_paths)?;
        merged_doc.save(&output_path)?;

        Ok(ProcessResult {
            message: format!("Successfully merged {} PDFs", file_paths.len()),
            output_path: Some(output_path),
        })
    })
    .await
}

/// Split PDF by page ranges
#[tauri::command]
pub async fn split_pdf(
    file_path: String,
    ranges: Vec<(usize, usize)>,
    output_dir: String,
) -> Result<ProcessResult, PdfError> {
    run_blocking(move || {
        if ranges.is_empty() {
            return Err(PdfError::InvalidOperation(
                "At least one page range is required".to_string(),
            ));
        }

        let doc = Document::load(&file_path)?;
        let total_pages = doc.get_pages().len();
        let base_name = Path::new(&file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("split");

        let all_pages = sorted_page_numbers(&doc);

        if all_pages.len() != total_pages {
            return Err(PdfError::InvalidOperation(format!(
                "Page count mismatch: expected {}, found {}",
                total_pages,
                all_pages.len()
            )));
        }

        let mut output_files = Vec::new();

        for (idx, (start, end)) in ranges.iter().enumerate() {
            if *start < 1 || *end > total_pages || start > end {
                return Err(PdfError::InvalidOperation(format!(
                    "Invalid page range: {}-{}",
                    start, end
                )));
            }

            let pages_to_extract: Vec<u32> = (*start..=*end).map(|i| all_pages[i - 1]).collect();

            let mut new_doc = doc.clone();
            let pages_to_delete: Vec<u32> = all_pages
                .iter()
                .filter(|&&p| !pages_to_extract.contains(&p))
                .cloned()
                .collect();

            if !pages_to_delete.is_empty() {
                new_doc.delete_pages(&pages_to_delete);
                new_doc.prune_objects();
            }

            let output_path = format!("{}/{}_{}.pdf", output_dir, base_name, idx + 1);
            new_doc.save(&output_path)?;
            output_files.push(output_path);
        }

        Ok(ProcessResult {
            message: format!("Successfully split into {} files", output_files.len()),
            output_path: Some(output_dir),
        })
    })
    .await
}

fn sorted_page_numbers(doc: &Document) -> Vec<u32> {
    let mut all_pages: Vec<u32> = doc.get_pages().keys().cloned().collect();
    all_pages.sort();
    all_pages
}

fn validated_page_numbers(doc: &Document, requested_pages: &[u32]) -> Result<Vec<u32>, PdfError> {
    let all_pages = sorted_page_numbers(doc);
    let page_count = all_pages.len() as u32;

    if requested_pages.is_empty() {
        return Err(PdfError::InvalidOperation(
            "At least one page must be selected".to_string(),
        ));
    }

    let mut seen = BTreeSet::new();
    let mut validated = Vec::with_capacity(requested_pages.len());

    for &page_index in requested_pages {
        if page_index < 1 || page_index > page_count {
            return Err(PdfError::InvalidOperation(format!(
                "Page {} is outside the valid range 1-{}",
                page_index, page_count
            )));
        }

        if !seen.insert(page_index) {
            return Err(PdfError::InvalidOperation(format!(
                "Page {} was selected more than once",
                page_index
            )));
        }

        validated.push(all_pages[(page_index - 1) as usize]);
    }

    Ok(validated)
}

fn delete_pages_document(file_path: &str, pages_to_delete: &[u32]) -> Result<Document, PdfError> {
    let mut doc = Document::load(file_path)?;
    let all_pages = sorted_page_numbers(&doc);
    let actual_pages_to_delete = validated_page_numbers(&doc, pages_to_delete)?;

    if actual_pages_to_delete.len() >= all_pages.len() {
        return Err(PdfError::InvalidOperation(
            "Cannot delete all pages: the output PDF would have no pages".to_string(),
        ));
    }

    doc.delete_pages(&actual_pages_to_delete);
    doc.prune_objects();
    Ok(doc)
}

fn extract_pages_document(file_path: &str, pages_to_extract: &[u32]) -> Result<Document, PdfError> {
    let mut doc = Document::load(file_path)?;
    let all_pages = sorted_page_numbers(&doc);
    let actual_pages_to_extract = validated_page_numbers(&doc, pages_to_extract)?;

    let pages_to_delete: Vec<u32> = all_pages
        .iter()
        .filter(|&&p| !actual_pages_to_extract.contains(&p))
        .cloned()
        .collect();

    doc.delete_pages(&pages_to_delete);
    doc.prune_objects();
    Ok(doc)
}

/// Delete specific pages from PDF
#[tauri::command]
pub async fn delete_pages(
    file_path: String,
    pages_to_delete: Vec<u32>,
    output_path: String,
) -> Result<ProcessResult, PdfError> {
    run_blocking(move || {
        let mut new_doc = delete_pages_document(&file_path, &pages_to_delete)?;
        new_doc.save(&output_path)?;

        Ok(ProcessResult {
            message: format!("Successfully deleted {} pages", pages_to_delete.len()),
            output_path: Some(output_path),
        })
    })
    .await
}

/// Extract specific pages from PDF
#[tauri::command]
pub async fn extract_pages(
    file_path: String,
    pages_to_extract: Vec<u32>,
    output_path: String,
) -> Result<ProcessResult, PdfError> {
    run_blocking(move || {
        let mut new_doc = extract_pages_document(&file_path, &pages_to_extract)?;
        new_doc.save(&output_path)?;

        Ok(ProcessResult {
            message: format!("Successfully extracted {} pages", pages_to_extract.len()),
            output_path: Some(output_path),
        })
    })
    .await
}

/// Compress PDF with lossless structural optimizations
#[tauri::command]
pub async fn compress_pdf(
    file_path: String,
    output_path: String,
    quality: u8,
) -> Result<ProcessResult, PdfError> {
    run_blocking(move || {
        let mut doc = Document::load(&file_path)?;
        apply_compression_profile(&mut doc, quality);

        doc.save(&output_path)?;

        let original_size = fs::metadata(&file_path)?.len();
        let new_size = fs::metadata(&output_path)?.len();

        Ok(ProcessResult {
            message: format_size_change_message(original_size, new_size),
            output_path: Some(output_path),
        })
    })
    .await
}

/// Find pdftoppm executable in likely install locations
fn find_pdftoppm() -> Option<PathBuf> {
    for dir in pdftoppm_search_dirs() {
        if let Some(candidate) = resolve_pdftoppm_from_dir(&dir) {
            return Some(candidate);
        }
    }

    #[cfg(target_os = "windows")]
    if let Some(candidate) = resolve_windows_where_pdftoppm() {
        return Some(candidate);
    }

    #[cfg(not(target_os = "windows"))]
    {
        resolve_brew_pdftoppm()
    }

    #[cfg(target_os = "windows")]
    {
        None
    }
}

#[cfg(target_os = "linux")]
fn filtered_appdir_ld_library_path(current: &OsStr, app_dir: &Path) -> Option<OsString> {
    let entries: Vec<PathBuf> = env::split_paths(current)
        .filter(|entry| !entry.starts_with(app_dir))
        .collect();

    if entries.is_empty() {
        return None;
    }

    env::join_paths(entries).ok()
}

#[cfg(target_os = "linux")]
fn set_or_remove_env(command: &mut Command, key: &str, value: Option<OsString>) {
    match value {
        Some(value) if !value.is_empty() => {
            command.env(key, value);
        }
        _ => {
            command.env_remove(key);
        }
    }
}

#[cfg(target_os = "linux")]
fn configure_external_tool_environment(command: &mut Command, program: &Path) {
    let app_dir = env::var_os("APPDIR").map(PathBuf::from);
    let program_is_bundled = app_dir
        .as_deref()
        .map(|app_dir| program.starts_with(app_dir))
        .unwrap_or(false);

    if program_is_bundled {
        return;
    }

    let launched_from_appimage = app_dir.is_some() || env::var_os("APPIMAGE").is_some();

    if let Some(original_ld_library_path) = env::var_os("LD_LIBRARY_PATH_ORIG") {
        set_or_remove_env(command, "LD_LIBRARY_PATH", Some(original_ld_library_path));
    } else if let (Some(current_ld_library_path), Some(app_dir)) =
        (env::var_os("LD_LIBRARY_PATH"), app_dir.as_deref())
    {
        set_or_remove_env(
            command,
            "LD_LIBRARY_PATH",
            filtered_appdir_ld_library_path(&current_ld_library_path, app_dir),
        );
    }

    if launched_from_appimage {
        command.env_remove("LD_PRELOAD");
        command.env_remove("LD_AUDIT");
    }
}

#[cfg(not(target_os = "linux"))]
fn configure_external_tool_environment(_command: &mut Command, _program: &Path) {}

fn external_tool_command(program: &Path) -> Command {
    let mut command = Command::new(program);
    configure_external_tool_environment(&mut command, program);
    command
}

/// Convert PDF pages to images
#[tauri::command]
pub async fn pdf_to_images(
    file_path: String,
    output_dir: String,
    format: String,
    dpi: Option<u32>,
) -> Result<ProcessResult, PdfError> {
    run_blocking(move || {
        fs::create_dir_all(&output_dir)?;

        let pdftoppm_path = find_pdftoppm().ok_or_else(|| {
            PdfError::InvalidOperation(
                "Could not find pdftoppm. Install Poppler or add pdftoppm to PATH.".to_string(),
            )
        })?;

        let dpi_value = dpi.unwrap_or(150);
        let format_flag = if format.to_lowercase() == "jpg" || format.to_lowercase() == "jpeg" {
            "-jpeg"
        } else {
            "-png"
        };

        let ext = if format.to_lowercase() == "jpg" || format.to_lowercase() == "jpeg" {
            "jpg"
        } else {
            "png"
        };

        let base_name = Path::new(&file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("page");

        let run_prefix = build_pdf_to_images_prefix(base_name)?;
        let output_prefix = Path::new(&output_dir).join(&run_prefix);

        let output = external_tool_command(&pdftoppm_path)
            .arg(format_flag)
            .arg("-r")
            .arg(dpi_value.to_string())
            .arg(&file_path)
            .arg(&output_prefix)
            .output()
            .map_err(|e| {
                PdfError::InvalidOperation(format!(
                    "Failed to execute pdftoppm at '{}': {}. Install Poppler or add pdftoppm to PATH.",
                    pdftoppm_path.display(),
                    e
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let details = if stderr.is_empty() {
                "no stderr output".to_string()
            } else {
                stderr
            };
            return Err(PdfError::InvalidOperation(format!(
                "pdftoppm failed to convert PDF to images: {}",
                details
            )));
        }

        let file_count = count_generated_image_files(Path::new(&output_dir), &run_prefix, ext)?;

        Ok(ProcessResult {
            message: format!(
                "成功转换为 {} 张 {} 图片（{}DPI）",
                file_count,
                ext.to_uppercase(),
                dpi_value
            ),
            output_path: Some(output_dir),
        })
    })
    .await
}

fn images_to_pdf_document(image_paths: &[String]) -> Result<Document, PdfError> {
    use image::{ColorType, GenericImageView, ImageFormat, ImageReader};
    use lopdf::dictionary;
    use lopdf::Stream;

    let mut doc = Document::with_version("1.5");
    let mut pages_kids = Vec::new();

    for image_path in image_paths.iter() {
        let reader = ImageReader::open(image_path)?.with_guessed_format()?;
        let format = reader.format();
        let img = reader.decode()?;
        let (width, height) = img.dimensions();

        let image_stream = if format == Some(ImageFormat::Jpeg)
            && matches!(img.color(), ColorType::Rgb8 | ColorType::L8)
        {
            // Embed the original JPEG bytes directly: no re-encoding, no size blowup
            let color_space = if img.color() == ColorType::L8 {
                "DeviceGray"
            } else {
                "DeviceRGB"
            };
            Stream::new(
                dictionary! {
                    "Type" => "XObject",
                    "Subtype" => "Image",
                    "Width" => width as i64,
                    "Height" => height as i64,
                    "ColorSpace" => color_space,
                    "BitsPerComponent" => 8,
                    "Filter" => "DCTDecode",
                },
                fs::read(image_path)?,
            )
        } else {
            let img_data = if img.color().has_alpha() {
                // Composite transparent pixels over white instead of dropping alpha
                let rgba = img.to_rgba8();
                let mut data = Vec::with_capacity((width as usize) * (height as usize) * 3);
                for pixel in rgba.pixels() {
                    let [r, g, b, a] = pixel.0;
                    let alpha = a as u32;
                    for channel in [r, g, b] {
                        data.push(((channel as u32 * alpha + 255 * (255 - alpha)) / 255) as u8);
                    }
                }
                data
            } else {
                img.to_rgb8().into_raw()
            };

            // No Filter here: doc.compress() below deflates filterless streams
            Stream::new(
                dictionary! {
                    "Type" => "XObject",
                    "Subtype" => "Image",
                    "Width" => width as i64,
                    "Height" => height as i64,
                    "ColorSpace" => "DeviceRGB",
                    "BitsPerComponent" => 8,
                },
                img_data,
            )
        };

        let image_id = doc.add_object(image_stream);
        let resources_id = doc.add_object(dictionary! {
            "XObject" => dictionary! {
                "Im1" => image_id,
            },
        });

        let content = format!("q {} 0 0 {} 0 0 cm /Im1 Do Q", width, height);
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.into_bytes()));

        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "MediaBox" => vec![0.into(), 0.into(), (width as i64).into(), (height as i64).into()],
            "Resources" => resources_id,
            "Contents" => content_id,
        });

        pages_kids.push(page_id.into());
    }

    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Kids" => pages_kids.clone(),
        "Count" => pages_kids.len() as i64,
    });

    for kid in &pages_kids {
        if let Object::Reference(page_ref) = kid {
            if let Ok(Object::Dictionary(ref mut page_dict)) = doc.get_object_mut(*page_ref) {
                page_dict.set("Parent", pages_id);
            }
        }
    }

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });

    doc.trailer.set("Root", catalog_id);
    doc.compress();

    Ok(doc)
}

/// Convert images to PDF
#[tauri::command]
pub async fn images_to_pdf(
    image_paths: Vec<String>,
    output_path: String,
) -> Result<ProcessResult, PdfError> {
    run_blocking(move || {
        let mut doc = images_to_pdf_document(&image_paths)?;
        doc.save(&output_path)?;

        Ok(ProcessResult {
            message: format!("Successfully created PDF from {} images", image_paths.len()),
            output_path: Some(output_path),
        })
    })
    .await
}

fn rotate_pages_document(
    file_path: &str,
    pages: &[u32],
    rotation: i64,
) -> Result<Document, PdfError> {
    if !matches!(rotation, 90 | 180 | 270) {
        return Err(PdfError::InvalidOperation(
            "Rotation must be 90, 180, or 270 degrees".to_string(),
        ));
    }

    let mut doc = Document::load(file_path)?;
    let actual_pages = validated_page_numbers(&doc, pages)?;
    let page_ids = doc.get_pages();

    for actual_page_number in actual_pages {
        let page_id = page_ids.get(&actual_page_number).copied().ok_or_else(|| {
            PdfError::InvalidOperation("Requested page not found in PDF".to_string())
        })?;
        let page_dictionary = doc.get_object_mut(page_id)?.as_dict_mut()?;
        let current_rotation = page_dictionary
            .get(b"Rotate")
            .ok()
            .and_then(|value| value.as_i64().ok())
            .unwrap_or(0);
        page_dictionary.set("Rotate", (current_rotation + rotation).rem_euclid(360));
    }

    Ok(doc)
}

/// Rotate pages in PDF
#[tauri::command]
pub async fn rotate_pages(
    file_path: String,
    pages: Vec<u32>,
    rotation: i64,
    output_path: String,
) -> Result<ProcessResult, PdfError> {
    run_blocking(move || {
        let mut doc = rotate_pages_document(&file_path, &pages, rotation)?;
        doc.save(&output_path)?;

        Ok(ProcessResult {
            message: format!(
                "Successfully rotated {} pages by {}°",
                pages.len(),
                rotation
            ),
            output_path: Some(output_path),
        })
    })
    .await
}

/// Reorder pages in PDF
#[tauri::command]
pub async fn reorder_pages(
    file_path: String,
    new_order: Vec<u32>,
    output_path: String,
) -> Result<ProcessResult, PdfError> {
    run_blocking(move || {
        let total_pages = new_order.len();
        let mut reordered_doc = reordered_pdf_document(&file_path, &new_order)?;
        reordered_doc.save(&output_path)?;

        Ok(ProcessResult {
            message: format!("Successfully reordered {} pages", total_pages),
            output_path: Some(output_path),
        })
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn object_to_f32(object: &Object) -> Result<f32, PdfError> {
        match object {
            Object::Integer(value) => Ok(*value as f32),
            Object::Real(value) => Ok(*value),
            _ => Err(PdfError::InvalidOperation(
                "Page box contains a non-numeric value".to_string(),
            )),
        }
    }

    fn page_media_box(doc: &Document, page_id: ObjectId) -> Result<Vec<f32>, PdfError> {
        let mut current_id = page_id;

        loop {
            let dictionary = doc.get_dictionary(current_id)?;

            if let Ok(media_box) = dictionary.get(b"MediaBox") {
                let media_box = media_box.as_array()?;
                return media_box.iter().map(object_to_f32).collect();
            }

            current_id = dictionary.get(b"Parent")?.as_reference()?;
        }
    }

    fn create_test_pdf(path: &Path, width: i64, height: i64, title: &str) -> Result<(), PdfError> {
        use lopdf::{dictionary, Stream};

        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();
        let content_id = doc.add_object(Stream::new(dictionary! {}, Vec::new()));

        doc.objects.insert(
            page_id,
            dictionary! {
                "Type" => "Page",
                "Parent" => pages_id,
                "MediaBox" => vec![0.into(), 0.into(), width.into(), height.into()],
                "Contents" => content_id,
            }
            .into(),
        );
        doc.objects.insert(
            pages_id,
            dictionary! {
                "Type" => "Pages",
                "Kids" => vec![page_id.into()],
                "Count" => 1,
            }
            .into(),
        );

        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        let info_id = doc.add_object(dictionary! {
            "Title" => Object::string_literal(title),
            "Author" => Object::string_literal("PDF Toolkit Tests"),
        });
        doc.trailer.set("Root", catalog_id);
        doc.trailer.set("Info", info_id);
        doc.save(path)?;
        Ok(())
    }

    fn create_fixture_pdfs(directory: &Path) -> Result<Vec<String>, PdfError> {
        let fixtures = [
            ("portrait.pdf", 612, 792, "Portrait"),
            ("landscape.pdf", 640, 480, "Landscape"),
            ("tall.pdf", 300, 900, "Tall"),
        ];

        fixtures
            .iter()
            .map(|(name, width, height, title)| {
                let path = directory.join(name);
                create_test_pdf(&path, *width, *height, title)?;
                Ok(path.to_string_lossy().into_owned())
            })
            .collect()
    }

    #[test]
    fn merge_preserves_page_sizes_from_source_documents() -> Result<(), PdfError> {
        let temp_dir = tempdir()?;
        let input_paths = create_fixture_pdfs(temp_dir.path())?;
        let output_path = temp_dir.path().join("merged.pdf");

        let mut merged_doc = merge_pdf_documents(&input_paths)?;
        merged_doc.save(&output_path)?;

        let merged_doc = Document::load(&output_path)?;
        let merged_pages = merged_doc.get_pages();
        assert_eq!(merged_pages.len(), input_paths.len());

        for (source_path, (_, merged_page_id)) in input_paths.iter().zip(merged_pages) {
            let source_doc = Document::load(source_path)?;
            let (_, source_page_id) =
                source_doc.get_pages().into_iter().next().ok_or_else(|| {
                    PdfError::InvalidOperation("Source PDF has no pages".to_string())
                })?;

            assert_eq!(
                page_media_box(&source_doc, source_page_id)?,
                page_media_box(&merged_doc, merged_page_id)?,
            );
        }

        Ok(())
    }

    #[test]
    fn reorder_preserves_requested_page_sequence() -> Result<(), PdfError> {
        let requested_order = vec![3, 1, 2];
        let temp_dir = tempdir()?;
        let source_paths = create_fixture_pdfs(temp_dir.path())?;
        let expected_source_paths = [&source_paths[2], &source_paths[0], &source_paths[1]];
        let merged_input_path = temp_dir.path().join("merged-input.pdf");
        let output_path = temp_dir.path().join("reordered.pdf");

        let mut merged_doc = merge_pdf_documents(&source_paths)?;
        merged_doc.save(&merged_input_path)?;

        let mut reordered_doc = reordered_pdf_document(
            merged_input_path.to_string_lossy().as_ref(),
            &requested_order,
        )?;
        reordered_doc.save(&output_path)?;

        let reordered_doc = Document::load(&output_path)?;
        let reordered_pages = reordered_doc.get_pages();
        assert_eq!(reordered_pages.len(), requested_order.len());

        for (source_path, (_, reordered_page_id)) in
            expected_source_paths.iter().zip(reordered_pages)
        {
            let source_doc = Document::load(source_path)?;
            let (_, source_page_id) =
                source_doc.get_pages().into_iter().next().ok_or_else(|| {
                    PdfError::InvalidOperation("Source PDF has no pages".to_string())
                })?;

            assert_eq!(
                page_media_box(&source_doc, source_page_id)?,
                page_media_box(&reordered_doc, reordered_page_id)?,
            );
        }

        Ok(())
    }

    #[test]
    fn size_change_message_handles_growth_without_underflow() {
        assert_eq!(
            format_size_change_message(1_000, 800),
            "Compressed PDF: 0 KB -> 0 KB (20% reduction)"
        );
        assert_eq!(
            format_size_change_message(1_000, 1_250),
            "Saved optimized PDF: 0 KB -> 1 KB (25% larger)"
        );
        assert_eq!(
            format_size_change_message(1_000, 1_000),
            "Saved optimized PDF: 0 KB -> 0 KB (no size change)"
        );
    }

    #[test]
    fn count_generated_images_ignores_unrelated_files() -> Result<(), PdfError> {
        let temp_dir = tempdir()?;
        let dir = temp_dir.path();

        fs::write(dir.join("report-1.png"), [])?;
        fs::write(dir.join("report-2.png"), [])?;
        fs::write(dir.join("other-1.png"), [])?;
        fs::write(dir.join("report-3.jpg"), [])?;

        assert_eq!(count_generated_image_files(dir, "report", "png")?, 2);

        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn appimage_ld_library_path_filter_removes_appdir_entries() {
        let app_dir = Path::new("/tmp/.mount_pdf-toolkit");
        let current = std::ffi::OsString::from(
            "/tmp/.mount_pdf-toolkit/usr/lib:/usr/lib:/tmp/.mount_pdf-toolkit/lib",
        );

        let filtered = filtered_appdir_ld_library_path(&current, app_dir)
            .expect("non-appdir library path should remain");
        let entries: Vec<PathBuf> = env::split_paths(&filtered).collect();

        assert_eq!(entries, vec![PathBuf::from("/usr/lib")]);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn appimage_ld_library_path_filter_returns_none_when_only_appdir_entries_remain() {
        let app_dir = Path::new("/tmp/.mount_pdf-toolkit");
        let current =
            std::ffi::OsString::from("/tmp/.mount_pdf-toolkit/usr/lib:/tmp/.mount_pdf-toolkit/lib");

        assert!(filtered_appdir_ld_library_path(&current, app_dir).is_none());
    }

    #[test]
    fn delete_all_pages_is_rejected() -> Result<(), PdfError> {
        let temp_dir = tempdir()?;
        let source_paths = create_fixture_pdfs(temp_dir.path())?;
        let error = delete_pages_document(&source_paths[0], &[1])
            .expect_err("deleting every page should fail");

        assert!(
            error.to_string().contains("Cannot delete all pages"),
            "unexpected error: {error}"
        );
        Ok(())
    }

    #[test]
    fn page_operations_reject_invalid_and_duplicate_page_numbers() -> Result<(), PdfError> {
        let temp_dir = tempdir()?;
        let source_paths = create_fixture_pdfs(temp_dir.path())?;
        let source_path = &source_paths[0];

        let invalid_extract = extract_pages_document(source_path, &[99])
            .expect_err("extracting an out-of-range page should fail");
        let duplicate_extract = extract_pages_document(source_path, &[1, 1])
            .expect_err("extracting a duplicate page should fail");
        let partially_invalid_delete = delete_pages_document(source_path, &[1, 99])
            .expect_err("a partially invalid delete request should fail");
        let invalid_rotation_page = rotate_pages_document(source_path, &[99], 90)
            .expect_err("rotating an out-of-range page should fail");
        let invalid_rotation = rotate_pages_document(source_path, &[1], 45)
            .expect_err("unsupported rotation should fail");

        assert!(
            invalid_extract
                .to_string()
                .contains("outside the valid range"),
            "unexpected error: {invalid_extract}"
        );
        assert!(
            duplicate_extract.to_string().contains("more than once"),
            "unexpected error: {duplicate_extract}"
        );
        assert!(
            partially_invalid_delete
                .to_string()
                .contains("outside the valid range"),
            "unexpected error: {partially_invalid_delete}"
        );
        assert!(
            invalid_rotation_page
                .to_string()
                .contains("outside the valid range"),
            "unexpected error: {invalid_rotation_page}"
        );
        assert!(
            invalid_rotation.to_string().contains("90, 180, or 270"),
            "unexpected error: {invalid_rotation}"
        );
        Ok(())
    }

    #[test]
    fn pdf_info_reads_title_and_author_metadata() -> Result<(), PdfError> {
        let temp_dir = tempdir()?;
        let source_paths = create_fixture_pdfs(temp_dir.path())?;
        let info = load_pdf_info(&source_paths[0])?;

        assert_eq!(info.page_count, 1);
        assert_eq!(info.title.as_deref(), Some("Portrait"));
        assert_eq!(info.author.as_deref(), Some("PDF Toolkit Tests"));
        Ok(())
    }

    #[test]
    fn delete_pages_prunes_orphaned_objects() -> Result<(), PdfError> {
        let temp_dir = tempdir()?;
        let source_paths = create_fixture_pdfs(temp_dir.path())?;
        let input_path = temp_dir.path().join("input.pdf");
        let output_path = temp_dir.path().join("output.pdf");

        let mut merged_doc = merge_pdf_documents(&source_paths)?;
        merged_doc.save(&input_path)?;

        let mut new_doc = delete_pages_document(input_path.to_string_lossy().as_ref(), &[2, 3])?;
        new_doc.save(&output_path)?;

        let saved_doc = Document::load(&output_path)?;
        assert_eq!(saved_doc.get_pages().len(), 1);

        // Only page 1 (from plot_01) is kept, so the output should not be much
        // larger than that page's source document; without pruning it would
        // stay close to the full merged input size.
        let retained_source_size = fs::metadata(&source_paths[0])?.len();
        let output_size = fs::metadata(&output_path)?.len();
        assert!(
            output_size <= retained_source_size * 11 / 10,
            "deleted pages should be pruned from the output: retained source {} bytes, output {} bytes",
            retained_source_size,
            output_size
        );

        Ok(())
    }

    #[test]
    fn images_to_pdf_embeds_jpeg_directly_and_composites_alpha() -> Result<(), PdfError> {
        use image::{Rgb, RgbImage, Rgba, RgbaImage};

        let temp_dir = tempdir()?;
        let jpeg_path = temp_dir.path().join("photo.jpg");
        let png_path = temp_dir.path().join("transparent.png");

        RgbImage::from_pixel(64, 64, Rgb([200, 30, 30])).save(&jpeg_path)?;
        RgbaImage::from_pixel(32, 32, Rgba([0, 0, 0, 0])).save(&png_path)?;

        let mut doc = images_to_pdf_document(&[
            jpeg_path.to_string_lossy().into_owned(),
            png_path.to_string_lossy().into_owned(),
        ])?;
        let output_path = temp_dir.path().join("output.pdf");
        doc.save(&output_path)?;

        let saved_doc = Document::load(&output_path)?;
        assert_eq!(saved_doc.get_pages().len(), 2);

        let mut found_jpeg = false;
        let mut found_flate = false;
        for object in saved_doc.objects.values() {
            let Object::Stream(stream) = object else {
                continue;
            };
            if stream
                .dict
                .get(b"Subtype")
                .and_then(Object::as_name_str)
                .ok()
                != Some("Image")
            {
                continue;
            }

            match stream.dict.get(b"Filter").and_then(Object::as_name_str) {
                Ok("DCTDecode") => {
                    assert_eq!(
                        stream.content,
                        fs::read(&jpeg_path)?,
                        "JPEG bytes should be embedded unchanged"
                    );
                    found_jpeg = true;
                }
                Ok("FlateDecode") => {
                    // lopdf refuses decompressed_content() on Subtype /Image
                    // streams, so strip the key on a clone before inflating
                    let mut stream = stream.clone();
                    stream.dict.remove(b"Subtype");
                    let data = stream.decompressed_content()?;
                    assert_eq!(
                        &data[..3],
                        &[255, 255, 255],
                        "transparent pixels should be composited over white"
                    );
                    found_flate = true;
                }
                other => panic!("unexpected image filter: {:?}", other),
            }
        }

        assert!(found_jpeg, "output should contain a DCTDecode image stream");
        assert!(
            found_flate,
            "output should contain a FlateDecode image stream"
        );

        Ok(())
    }

    #[test]
    fn reorder_rejects_duplicate_page_numbers() -> Result<(), PdfError> {
        let temp_dir = tempdir()?;
        let source_paths = create_fixture_pdfs(temp_dir.path())?;
        let merged_input_path = temp_dir.path().join("merged-input.pdf");
        let mut merged_doc = merge_pdf_documents(&source_paths)?;
        merged_doc.save(&merged_input_path)?;

        let error =
            reordered_pdf_document(merged_input_path.to_string_lossy().as_ref(), &[1, 1, 2])
                .expect_err("duplicate page order should fail");

        assert!(
            error
                .to_string()
                .contains("permutation of every page number exactly once"),
            "unexpected error: {error}"
        );
        Ok(())
    }
}
