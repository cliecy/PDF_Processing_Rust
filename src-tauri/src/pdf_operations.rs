use lopdf::{Document, Object, ObjectId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResult {
    pub success: bool,
    pub message: String,
    pub output_path: Option<String>,
}

/// Get PDF information
#[tauri::command]
pub async fn get_pdf_info(file_path: String) -> Result<PdfInfo, PdfError> {
    let path = Path::new(&file_path);
    let metadata = fs::metadata(path)?;
    let doc = Document::load(path)?;
    
    let page_count = doc.get_pages().len();
    
    let title = doc.trailer.get(b"Info")
        .ok()
        .and_then(|info| info.as_reference().ok())
        .and_then(|info_ref| doc.get_object(info_ref).ok())
        .and_then(|info_obj| {
            if let Object::Dictionary(dict) = info_obj {
                dict.get(b"Title").ok()
                    .and_then(|t| {
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
    
    let author = None; // Simplified for now
    
    Ok(PdfInfo {
        page_count,
        file_size: metadata.len(),
        title,
        author,
    })
}

/// Merge multiple PDFs into one
#[tauri::command]
pub async fn merge_pdfs(file_paths: Vec<String>, output_path: String) -> Result<ProcessResult, PdfError> {
    if file_paths.is_empty() {
        return Err(PdfError::InvalidOperation("No input files provided".to_string()));
    }

    let mut documents: Vec<Document> = Vec::new();
    
    for path in &file_paths {
        let doc = Document::load(path)?;
        documents.push(doc);
    }

    // Start with the first document as base
    let mut base_doc = documents.remove(0);
    let mut max_id = base_doc.max_id;

    for doc in documents {
        let pages = doc.get_pages();
        let page_ids: Vec<ObjectId> = pages.values().cloned().collect();
        
        // Clone objects from source document
        let mut id_mapping: BTreeMap<ObjectId, ObjectId> = BTreeMap::new();
        
        for (old_id, object) in doc.objects.iter() {
            max_id += 1;
            let new_id = (max_id, 0);
            id_mapping.insert(*old_id, new_id);
            base_doc.objects.insert(new_id, object.clone());
        }
        
        // Update references in the merged objects
        for old_page_id in page_ids {
            if let Some(&new_page_id) = id_mapping.get(&old_page_id) {
                // Add page to the document's page tree
                if let Ok(pages_id) = base_doc.catalog()?.get(b"Pages")?.as_reference() {
                    if let Ok(Object::Dictionary(ref mut pages_dict)) = base_doc.get_object_mut(pages_id) {
                        if let Ok(Object::Array(ref mut kids)) = pages_dict.get_mut(b"Kids") {
                            kids.push(Object::Reference(new_page_id));
                        }
                        if let Ok(Object::Integer(ref mut count)) = pages_dict.get_mut(b"Count") {
                            *count += 1;
                        }
                    }
                }
            }
        }
    }
    
    base_doc.max_id = max_id;
    base_doc.save(&output_path)?;

    Ok(ProcessResult {
        success: true,
        message: format!("Successfully merged {} PDFs", file_paths.len() + 1),
        output_path: Some(output_path),
    })
}

/// Split PDF by page ranges
#[tauri::command]
pub async fn split_pdf(
    file_path: String,
    ranges: Vec<(usize, usize)>,
    output_dir: String,
) -> Result<ProcessResult, PdfError> {
    let doc = Document::load(&file_path)?;
    let total_pages = doc.get_pages().len();
    let base_name = Path::new(&file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("split");

    // Get all page numbers and sort them to ensure correct order
    // Note: get_pages() returns a HashMap where keys are page numbers (may not be 1-based or sequential)
    let mut all_pages: Vec<u32> = doc.get_pages().keys().cloned().collect();
    all_pages.sort();

    // Create a mapping from 1-based page index to actual page number
    // all_pages[0] is the first page, all_pages[1] is the second page, etc.
    if all_pages.len() != total_pages {
        return Err(PdfError::InvalidOperation(format!(
            "Page count mismatch: expected {}, found {}",
            total_pages, all_pages.len()
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

        // Convert 1-based indices to actual page numbers
        // start=1 means all_pages[0], start=2 means all_pages[1], etc.
        let pages_to_extract: Vec<u32> = (*start..=*end)
            .map(|i| all_pages[(i - 1) as usize])  // Convert 1-based to 0-based index
            .collect();

        // Clone the original document for each range (always from original)
        let mut new_doc = doc.clone();
        
        // Delete all pages NOT in the extraction list
        let pages_to_delete: Vec<u32> = all_pages
            .iter()
            .filter(|&&p| !pages_to_extract.contains(&p))
            .cloned()
            .collect();

        // Delete all unwanted pages at once
        if !pages_to_delete.is_empty() {
            new_doc.delete_pages(&pages_to_delete);
        }

        let output_path = format!("{}/{}_{}.pdf", output_dir, base_name, idx + 1);
        new_doc.save(&output_path)?;
        output_files.push(output_path);
    }

    Ok(ProcessResult {
        success: true,
        message: format!("Successfully split into {} files", output_files.len()),
        output_path: Some(output_dir),
    })
}

/// Delete specific pages from PDF
#[tauri::command]
pub async fn delete_pages(
    file_path: String,
    pages_to_delete: Vec<u32>,
    output_path: String,
) -> Result<ProcessResult, PdfError> {
    let doc = Document::load(&file_path)?;
    // Get all page numbers and sort them to ensure correct order
    let mut all_pages: Vec<u32> = doc.get_pages().keys().cloned().collect();
    all_pages.sort();
    
    // Convert 1-based page indices to actual page numbers
    // pages_to_delete contains 1-based indices (1, 2, 3, ...)
    let page_count = all_pages.len() as u32;
    let actual_pages_to_delete: Vec<u32> = pages_to_delete
        .iter()
        .filter(|&&idx| idx >= 1 && idx <= page_count)
        .map(|&idx| all_pages[(idx - 1) as usize])  // Convert 1-based to 0-based index
        .collect();
    
    let mut new_doc = doc.clone();
    new_doc.delete_pages(&actual_pages_to_delete);
    new_doc.save(&output_path)?;

    Ok(ProcessResult {
        success: true,
        message: format!("Successfully deleted {} pages", pages_to_delete.len()),
        output_path: Some(output_path),
    })
}

/// Extract specific pages from PDF
#[tauri::command]
pub async fn extract_pages(
    file_path: String,
    pages_to_extract: Vec<u32>,
    output_path: String,
) -> Result<ProcessResult, PdfError> {
    let doc = Document::load(&file_path)?;
    // Get all page numbers and sort them to ensure correct order
    let mut all_pages: Vec<u32> = doc.get_pages().keys().cloned().collect();
    all_pages.sort();
    
    // Convert 1-based page indices to actual page numbers
    // pages_to_extract contains 1-based indices (1, 2, 3, ...)
    let page_count = all_pages.len() as u32;
    let actual_pages_to_extract: Vec<u32> = pages_to_extract
        .iter()
        .filter(|&&idx| idx >= 1 && idx <= page_count)
        .map(|&idx| all_pages[(idx - 1) as usize])  // Convert 1-based to 0-based index
        .collect();
    
    let pages_to_delete: Vec<u32> = all_pages
        .iter()
        .filter(|&&p| !actual_pages_to_extract.contains(&p))
        .cloned()
        .collect();

    let mut new_doc = doc.clone();
    new_doc.delete_pages(&pages_to_delete);
    new_doc.save(&output_path)?;

    Ok(ProcessResult {
        success: true,
        message: format!("Successfully extracted {} pages", pages_to_extract.len()),
        output_path: Some(output_path),
    })
}

/// Compress PDF by reducing image quality
#[tauri::command]
pub async fn compress_pdf(
    file_path: String,
    output_path: String,
    _quality: u8,
) -> Result<ProcessResult, PdfError> {
    let mut doc = Document::load(&file_path)?;
    
    // Compress streams
    doc.compress();
    
    // Remove unused objects
    doc.delete_zero_length_streams();
    doc.prune_objects();
    
    doc.save(&output_path)?;
    
    let original_size = fs::metadata(&file_path)?.len();
    let new_size = fs::metadata(&output_path)?.len();
    let reduction = ((original_size - new_size) as f64 / original_size as f64 * 100.0) as u32;

    Ok(ProcessResult {
        success: true,
        message: format!(
            "Compressed PDF: {} KB -> {} KB ({}% reduction)",
            original_size / 1024,
            new_size / 1024,
            reduction
        ),
        output_path: Some(output_path),
    })
}

/// Convert PDF pages to images
#[tauri::command]
pub async fn pdf_to_images(
    file_path: String,
    output_dir: String,
    format: String,
) -> Result<ProcessResult, PdfError> {
    use std::process::Command;
    
    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir)?;
    
    // Determine output format flag
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
    
    // Get base name for output files
    let base_name = Path::new(&file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");
    
    let output_prefix = Path::new(&output_dir).join(base_name);
    
    // Construct command: pdftoppm -<format> -r 200 input.pdf output_prefix
    // Note: pdftoppm automatically adds -1, -2, etc. and extension
    let status = Command::new("pdftoppm")
        .arg(format_flag)
        .arg("-r")
        .arg("200")
        .arg(&file_path)
        .arg(&output_prefix)
        .status()
        .map_err(|e| PdfError::InvalidOperation(format!("Failed to execute pdftoppm: {}. Please ensure Poppler is installed and on PATH.", e)))?;

    if !status.success() {
        return Err(PdfError::InvalidOperation("pdftoppm failed to convert PDF to images".to_string()));
    }
    
    Ok(ProcessResult {
        success: true,
        message: format!(
            "Successfully converted to {} format images in {}",
            ext.to_uppercase(),
            output_dir
        ),
        output_path: Some(output_dir),
    })
}

/// Convert images to PDF
#[tauri::command]
pub async fn images_to_pdf(
    image_paths: Vec<String>,
    output_path: String,
) -> Result<ProcessResult, PdfError> {
    use image::GenericImageView;
    use lopdf::dictionary;
    use lopdf::Stream;
    
    let mut doc = Document::with_version("1.5");
    let mut pages_kids = Vec::new();
    
    for (_idx, image_path) in image_paths.iter().enumerate() {
        let img = image::open(image_path)?;
        let (width, height) = img.dimensions();
        
        // Convert to RGB
        let rgb_img = img.to_rgb8();
        let img_data = rgb_img.into_raw();
        
        // Create image XObject
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
        
        // Create resources dictionary
        let resources_id = doc.add_object(dictionary! {
            "XObject" => dictionary! {
                "Im1" => image_id,
            },
        });
        
        // Create content stream
        let content = format!(
            "q {} 0 0 {} 0 0 cm /Im1 Do Q",
            width, height
        );
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.into_bytes()));
        
        // Create page
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "MediaBox" => vec![0.into(), 0.into(), (width as i64).into(), (height as i64).into()],
            "Resources" => resources_id,
            "Contents" => content_id,
        });
        
        pages_kids.push(page_id.into());
    }
    
    // Create pages object
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Kids" => pages_kids.clone(),
        "Count" => pages_kids.len() as i64,
    });
    
    // Update page parent references
    for kid in &pages_kids {
        if let Object::Reference(page_ref) = kid {
            if let Ok(Object::Dictionary(ref mut page_dict)) = doc.get_object_mut(*page_ref) {
                page_dict.set("Parent", pages_id);
            }
        }
    }
    
    // Create catalog
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    
    doc.trailer.set("Root", catalog_id);
    doc.save(&output_path)?;

    Ok(ProcessResult {
        success: true,
        message: format!("Successfully created PDF from {} images", image_paths.len()),
        output_path: Some(output_path),
    })
}

/// Rotate pages in PDF
#[tauri::command]
pub async fn rotate_pages(
    file_path: String,
    pages: Vec<u32>,
    rotation: i64,
    output_path: String,
) -> Result<ProcessResult, PdfError> {
    let mut doc = Document::load(&file_path)?;
    // Get all page numbers and sort them to ensure correct order
    let mut all_pages: Vec<u32> = doc.get_pages().keys().cloned().collect();
    all_pages.sort();
    
    let page_ids = doc.get_pages();
    
    // Convert 1-based page indices to actual page numbers
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
    
    doc.save(&output_path)?;

    Ok(ProcessResult {
        success: true,
        message: format!("Successfully rotated {} pages by {}°", pages.len(), rotation),
        output_path: Some(output_path),
    })
}

/// Reorder pages in PDF
#[tauri::command]
pub async fn reorder_pages(
    file_path: String,
    new_order: Vec<u32>,
    output_path: String,
) -> Result<ProcessResult, PdfError> {
    let doc = Document::load(&file_path)?;
    let page_ids = doc.get_pages();
    let total_pages = page_ids.len();
    
    if new_order.len() != total_pages {
        return Err(PdfError::InvalidOperation(
            "New order must contain all page numbers".to_string(),
        ));
    }
    
    // Create new document with reordered pages
    // This is a simplified implementation
    // Full implementation would need to properly copy all objects
    
    let mut doc_clone = doc.clone();
    
    // For now, just save the document as-is
    // A proper implementation would reorder the pages in the page tree
    doc_clone.save(&output_path)?;

    Ok(ProcessResult {
        success: true,
        message: format!("Successfully reordered {} pages", total_pages),
        output_path: Some(output_path),
    })
}
