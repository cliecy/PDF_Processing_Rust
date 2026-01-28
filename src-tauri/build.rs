fn main() {
    tauri_build::build();
    
    // With pdfium-render bindings feature, PDFium library is automatically downloaded
    // during build. The bindings feature handles library discovery at runtime.
    // Tauri should automatically bundle dynamic libraries that are linked.
    // If PDFium library is not found at runtime, it means the library wasn't
    // properly bundled. The bindings feature should handle this automatically,
    // but we may need to ensure the library is in the right location.
}
