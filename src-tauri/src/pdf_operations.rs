use lopdf::{Document, Object, ObjectId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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
    pub success: bool,
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

fn pdf_path_error(action: &str, path: &Path, error: impl ToString) -> PdfError {
    let detail = error.to_string();
    let mut message = format!("{}失败：{}\n{}", action, path.display(), detail);
    let detail_lower = detail.to_ascii_lowercase();

    if detail_lower.contains("timed out") || detail_lower.contains("os error 60") {
        message.push_str(
            "\n该路径读写超时。常见原因是文件位于 iCloud、网盘、外置磁盘，或目录尚未完成同步。请先改到本地磁盘目录后再试。",
        );
    }

    PdfError::InvalidOperation(message)
}

fn load_pdf_document(path: impl AsRef<Path>) -> Result<Document, PdfError> {
    let path = path.as_ref();
    Document::load(path).map_err(|error| pdf_path_error("读取 PDF", path, error))
}

fn ensure_output_directory(output_path: &Path) -> Result<PathBuf, PdfError> {
    let parent = output_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    if !parent.exists() {
        return Err(PdfError::InvalidOperation(format!(
            "输出目录不存在：{}",
            parent.display()
        )));
    }

    if !parent.is_dir() {
        return Err(PdfError::InvalidOperation(format!(
            "输出路径的上级不是目录：{}",
            parent.display()
        )));
    }

    Ok(parent)
}

fn same_filesystem_path(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }

    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn ensure_output_is_not_an_input(
    input_paths: &[String],
    output_path: impl AsRef<Path>,
) -> Result<(), PdfError> {
    let output_path = output_path.as_ref();

    if input_paths
        .iter()
        .map(Path::new)
        .any(|input_path| same_filesystem_path(input_path, output_path))
    {
        return Err(PdfError::InvalidOperation(
            "输出文件不能覆盖输入文件，请选择新的保存路径。".to_string(),
        ));
    }

    Ok(())
}

fn save_pdf_document(
    document: &mut Document,
    output_path: impl AsRef<Path>,
) -> Result<(), PdfError> {
    let output_path = output_path.as_ref();
    let output_dir = ensure_output_directory(output_path)?;
    let temp_path = tempfile::Builder::new()
        .prefix(".pdf-toolkit-")
        .suffix(".pdf")
        .tempfile_in(&output_dir)
        .map_err(|error| pdf_path_error("创建临时输出文件", &output_dir, error))?
        .into_temp_path();

    document
        .save(&temp_path)
        .map_err(|error| pdf_path_error("写入输出 PDF", temp_path.as_ref(), error))?;

    if output_path.exists() {
        fs::remove_file(output_path)
            .map_err(|error| pdf_path_error("覆盖现有输出文件", output_path, error))?;
    }

    temp_path
        .persist(output_path)
        .map_err(|error| pdf_path_error("保存输出 PDF", output_path, error.error))?;

    Ok(())
}

/// Get PDF information
#[tauri::command]
pub async fn get_pdf_info(file_path: String) -> Result<PdfInfo, PdfError> {
    run_blocking(move || {
        let path = Path::new(&file_path);
        let metadata =
            fs::metadata(path).map_err(|error| pdf_path_error("读取文件信息", path, error))?;
        let doc = load_pdf_document(path)?;

        let page_count = doc.get_pages().len();
        let pdf_version = doc.version.clone();
        let is_encrypted = doc.trailer.get(b"Encrypt").is_ok();

        let title = doc
            .trailer
            .get(b"Info")
            .ok()
            .and_then(|info| info.as_reference().ok())
            .and_then(|info_ref| doc.get_object(info_ref).ok())
            .and_then(|info_obj| {
                if let Object::Dictionary(dict) = info_obj {
                    dict.get(b"Title").ok().and_then(|t| {
                        if let Object::String(s, _) = t {
                            String::from_utf8(s.clone()).ok()
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            });

        Ok(PdfInfo {
            page_count,
            file_size: metadata.len(),
            title,
            author: None,
            pdf_version,
            is_encrypted,
        })
    })
    .await
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
        let mut doc = load_pdf_document(path)?;
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
        let output = match std::process::Command::new(brew_path)
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
    let output = std::process::Command::new("where")
        .arg("pdftoppm")
        .output()
        .ok()?;

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
        ensure_output_is_not_an_input(&file_paths, &output_path)?;
        let mut merged_doc = merge_pdf_documents(&file_paths)?;
        save_pdf_document(&mut merged_doc, &output_path)?;

        Ok(ProcessResult {
            success: true,
            message: format!("成功合并 {} 个 PDF 文件", file_paths.len()),
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
        let doc = load_pdf_document(&file_path)?;
        let total_pages = doc.get_pages().len();
        let base_name = Path::new(&file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("split");

        let mut all_pages: Vec<u32> = doc.get_pages().keys().cloned().collect();
        all_pages.sort();

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

            let pages_to_extract: Vec<u32> = (*start..=*end)
                .map(|i| all_pages[(i - 1) as usize])
                .collect();

            let mut new_doc = doc.clone();
            let pages_to_delete: Vec<u32> = all_pages
                .iter()
                .filter(|&&p| !pages_to_extract.contains(&p))
                .cloned()
                .collect();

            if !pages_to_delete.is_empty() {
                new_doc.delete_pages(&pages_to_delete);
            }

            let output_path = format!("{}/{}_{}.pdf", output_dir, base_name, idx + 1);
            save_pdf_document(&mut new_doc, &output_path)?;
            output_files.push(output_path);
        }

        Ok(ProcessResult {
            success: true,
            message: format!("Successfully split into {} files", output_files.len()),
            output_path: Some(output_dir),
        })
    })
    .await
}

/// Delete specific pages from PDF
#[tauri::command]
pub async fn delete_pages(
    file_path: String,
    pages_to_delete: Vec<u32>,
    output_path: String,
) -> Result<ProcessResult, PdfError> {
    run_blocking(move || {
        let doc = load_pdf_document(&file_path)?;
        let mut all_pages: Vec<u32> = doc.get_pages().keys().cloned().collect();
        all_pages.sort();

        let page_count = all_pages.len() as u32;
        let actual_pages_to_delete: Vec<u32> = pages_to_delete
            .iter()
            .filter(|&&idx| idx >= 1 && idx <= page_count)
            .map(|&idx| all_pages[(idx - 1) as usize])
            .collect();

        let mut new_doc = doc.clone();
        new_doc.delete_pages(&actual_pages_to_delete);
        save_pdf_document(&mut new_doc, &output_path)?;

        Ok(ProcessResult {
            success: true,
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
        let doc = load_pdf_document(&file_path)?;
        let mut all_pages: Vec<u32> = doc.get_pages().keys().cloned().collect();
        all_pages.sort();

        let page_count = all_pages.len() as u32;
        let actual_pages_to_extract: Vec<u32> = pages_to_extract
            .iter()
            .filter(|&&idx| idx >= 1 && idx <= page_count)
            .map(|&idx| all_pages[(idx - 1) as usize])
            .collect();

        let pages_to_delete: Vec<u32> = all_pages
            .iter()
            .filter(|&&p| !actual_pages_to_extract.contains(&p))
            .cloned()
            .collect();

        let mut new_doc = doc.clone();
        new_doc.delete_pages(&pages_to_delete);
        save_pdf_document(&mut new_doc, &output_path)?;

        Ok(ProcessResult {
            success: true,
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
        let mut doc = load_pdf_document(&file_path)?;
        apply_compression_profile(&mut doc, quality);

        save_pdf_document(&mut doc, &output_path)?;

        let original_size = fs::metadata(&file_path)
            .map_err(|error| pdf_path_error("读取文件信息", Path::new(&file_path), error))?
            .len();
        let new_size = fs::metadata(&output_path)
            .map_err(|error| pdf_path_error("读取文件信息", Path::new(&output_path), error))?
            .len();

        Ok(ProcessResult {
            success: true,
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
        return resolve_brew_pdftoppm();
    }

    #[cfg(target_os = "windows")]
    {
        None
    }
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
        use std::process::Command;

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

        let output = Command::new(&pdftoppm_path)
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
            success: true,
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

/// Convert images to PDF
#[tauri::command]
pub async fn images_to_pdf(
    image_paths: Vec<String>,
    output_path: String,
) -> Result<ProcessResult, PdfError> {
    run_blocking(move || {
        use image::GenericImageView;
        use lopdf::Stream;
        use lopdf::dictionary;

        let mut doc = Document::with_version("1.5");
        let mut pages_kids = Vec::new();

        for (_idx, image_path) in image_paths.iter().enumerate() {
            let img = image::open(image_path)?;
            let (width, height) = img.dimensions();

            let rgb_img = img.to_rgb8();
            let img_data = rgb_img.into_raw();

            let image_stream = Stream::new(
                dictionary! {
                    "Type" => "XObject",
                    "Subtype" => "Image",
                    "Width" => width as i64,
                    "Height" => height as i64,
                    "ColorSpace" => "DeviceRGB",
                    "BitsPerComponent" => 8,
                },
                img_data,
            );

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
        save_pdf_document(&mut doc, &output_path)?;

        Ok(ProcessResult {
            success: true,
            message: format!("Successfully created PDF from {} images", image_paths.len()),
            output_path: Some(output_path),
        })
    })
    .await
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
        let mut doc = load_pdf_document(&file_path)?;
        let mut all_pages: Vec<u32> = doc.get_pages().keys().cloned().collect();
        all_pages.sort();

        let page_ids = doc.get_pages();
        let page_count = all_pages.len() as u32;

        for page_idx in &pages {
            if *page_idx >= 1 && *page_idx <= page_count {
                let actual_page_num = all_pages[(*page_idx - 1) as usize];
                if let Some(&page_id) = page_ids.get(&actual_page_num) {
                    if let Ok(Object::Dictionary(ref mut page_dict)) = doc.get_object_mut(page_id) {
                        let current_rotation = page_dict
                            .get(b"Rotate")
                            .ok()
                            .and_then(|r| r.as_i64().ok())
                            .unwrap_or(0);
                        let new_rotation = (current_rotation + rotation) % 360;
                        page_dict.set("Rotate", new_rotation);
                    }
                }
            }
        }

        save_pdf_document(&mut doc, &output_path)?;

        Ok(ProcessResult {
            success: true,
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
        save_pdf_document(&mut reordered_doc, &output_path)?;

        Ok(ProcessResult {
            success: true,
            message: format!("Successfully reordered {} pages", total_pages),
            output_path: Some(output_path),
        })
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{dictionary, Stream};
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

    fn create_test_pdf(path: &Path, width: i64, height: i64) -> Result<(), PdfError> {
        let mut doc = Document::with_version("1.5");
        let resources_id = doc.add_object(dictionary! {});
        let content_id = doc.add_object(Stream::new(dictionary! {}, Vec::new()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "MediaBox" => vec![0.into(), 0.into(), width.into(), height.into()],
            "Resources" => resources_id,
            "Contents" => content_id,
        });
        let pages_id = doc.add_object(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        });

        if let Ok(Object::Dictionary(page_dict)) = doc.get_object_mut(page_id) {
            page_dict.set("Parent", pages_id);
        }

        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });

        doc.trailer.set("Root", catalog_id);
        save_pdf_document(&mut doc, path)
    }

    fn create_merge_fixture_pdfs(temp_dir: &Path) -> Result<Vec<String>, PdfError> {
        let page_sizes = [(320, 480), (612, 792), (420, 420)];
        page_sizes
            .iter()
            .enumerate()
            .map(|(index, (width, height))| {
                let path = temp_dir.join(format!("fixture-{}.pdf", index + 1));
                create_test_pdf(&path, *width, *height)?;
                Ok(path.to_string_lossy().into_owned())
            })
            .collect()
    }

    #[test]
    fn merge_preserves_page_sizes_from_source_documents() -> Result<(), PdfError> {
        let temp_dir = tempdir()?;
        let input_paths = create_merge_fixture_pdfs(temp_dir.path())?;
        let output_path = temp_dir.path().join("merged.pdf");

        let mut merged_doc = merge_pdf_documents(&input_paths)?;
        save_pdf_document(&mut merged_doc, &output_path)?;

        let merged_doc = load_pdf_document(&output_path)?;
        let merged_pages = merged_doc.get_pages();
        assert_eq!(merged_pages.len(), input_paths.len());

        for (source_path, (_, merged_page_id)) in input_paths.iter().zip(merged_pages.into_iter()) {
            let source_doc = load_pdf_document(source_path)?;
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
        let source_paths = create_merge_fixture_pdfs(temp_dir.path())?;
        let expected_source_paths = [
            source_paths[2].clone(),
            source_paths[0].clone(),
            source_paths[1].clone(),
        ];
        let merged_input_path = temp_dir.path().join("merged-input.pdf");
        let output_path = temp_dir.path().join("reordered.pdf");

        let mut merged_doc = merge_pdf_documents(&source_paths)?;
        save_pdf_document(&mut merged_doc, &merged_input_path)?;

        let mut reordered_doc = reordered_pdf_document(
            merged_input_path.to_string_lossy().as_ref(),
            &requested_order,
        )?;
        save_pdf_document(&mut reordered_doc, &output_path)?;

        let reordered_doc = load_pdf_document(&output_path)?;
        let reordered_pages = reordered_doc.get_pages();
        assert_eq!(reordered_pages.len(), requested_order.len());

        for (source_path, (_, reordered_page_id)) in expected_source_paths
            .iter()
            .zip(reordered_pages.into_iter())
        {
            let source_doc = load_pdf_document(source_path)?;
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

    #[test]
    fn reorder_rejects_duplicate_page_numbers() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let source_paths =
            create_merge_fixture_pdfs(temp_dir.path()).expect("fixture PDFs should be created");
        let merged_input_path = temp_dir.path().join("merged-input.pdf");
        let mut merged_doc =
            merge_pdf_documents(&source_paths).expect("merge fixture should be created");
        save_pdf_document(&mut merged_doc, &merged_input_path)
            .expect("merged fixture should be saved");

        let error =
            reordered_pdf_document(merged_input_path.to_string_lossy().as_ref(), &[1, 1, 2])
                .expect_err("duplicate page order should fail");

        assert!(
            error
                .to_string()
                .contains("permutation of every page number exactly once"),
            "unexpected error: {error}"
        );
    }
}
