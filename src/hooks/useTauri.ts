import { invoke } from '@tauri-apps/api/core';
import { save, open } from '@tauri-apps/plugin-dialog';

export interface PdfInfo {
  page_count: number;
  file_size: number;
  title: string | null;
  author: string | null;
  pdf_version: string;
  is_encrypted: boolean;
}

interface ProcessOutput {
  message: string;
  output_path: string | null;
}

export interface ProcessResult extends ProcessOutput {
  success: boolean;
}

export function useTauri() {
  const invokeProcess = async (
    command: string,
    args: Record<string, unknown>
  ): Promise<ProcessResult> => {
    const output = await invoke<ProcessOutput>(command, args);
    return { success: true, ...output };
  };

  const getPdfInfo = async (filePath: string): Promise<PdfInfo> => {
    return await invoke('get_pdf_info', { filePath });
  };

  const mergePdfs = async (
    filePaths: string[],
    outputPath: string
  ): Promise<ProcessResult> => {
    return await invokeProcess('merge_pdfs', { filePaths, outputPath });
  };

  const splitPdf = async (
    filePath: string,
    ranges: [number, number][],
    outputDir: string
  ): Promise<ProcessResult> => {
    return await invokeProcess('split_pdf', { filePath, ranges, outputDir });
  };

  const deletePages = async (
    filePath: string,
    pagesToDelete: number[],
    outputPath: string
  ): Promise<ProcessResult> => {
    return await invokeProcess('delete_pages', { filePath, pagesToDelete, outputPath });
  };

  const extractPages = async (
    filePath: string,
    pagesToExtract: number[],
    outputPath: string
  ): Promise<ProcessResult> => {
    return await invokeProcess('extract_pages', { filePath, pagesToExtract, outputPath });
  };

  const compressPdf = async (
    filePath: string,
    outputPath: string,
    quality: number
  ): Promise<ProcessResult> => {
    return await invokeProcess('compress_pdf', { filePath, outputPath, quality });
  };

  const pdfToImages = async (
    filePath: string,
    outputDir: string,
    format: string,
    dpi?: number
  ): Promise<ProcessResult> => {
    return await invokeProcess('pdf_to_images', { filePath, outputDir, format, dpi });
  };

  const openFolder = async (path: string): Promise<void> => {
    await invoke('open_folder', { path });
  };

  const imagesToPdf = async (
    imagePaths: string[],
    outputPath: string
  ): Promise<ProcessResult> => {
    return await invokeProcess('images_to_pdf', { imagePaths, outputPath });
  };

  const rotatePages = async (
    filePath: string,
    pages: number[],
    rotation: number,
    outputPath: string
  ): Promise<ProcessResult> => {
    return await invokeProcess('rotate_pages', { filePath, pages, rotation, outputPath });
  };

  const reorderPages = async (
    filePath: string,
    newOrder: number[],
    outputPath: string
  ): Promise<ProcessResult> => {
    return await invokeProcess('reorder_pages', { filePath, newOrder, outputPath });
  };

  const selectPdfFiles = async (multiple = true): Promise<string[] | null> => {
    const result = await open({
      multiple,
      filters: [{ name: 'PDF', extensions: ['pdf'] }],
    });
    if (result === null) return null;
    return Array.isArray(result) ? result : [result];
  };

  const selectImageFiles = async (): Promise<string[] | null> => {
    const result = await open({
      multiple: true,
      filters: [{ name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'gif', 'webp'] }],
    });
    if (result === null) return null;
    return Array.isArray(result) ? result : [result];
  };

  const selectOutputFile = async (defaultName: string): Promise<string | null> => {
    return await save({
      defaultPath: defaultName,
      filters: [{ name: 'PDF', extensions: ['pdf'] }],
    });
  };

  const selectOutputDir = async (): Promise<string | null> => {
    const result = await open({
      directory: true,
    });
    return result as string | null;
  };

  // Format file size to human readable string
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  // Copy text to clipboard
  const copyToClipboard = async (text: string): Promise<boolean> => {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      return false;
    }
  };

  return {
    getPdfInfo,
    mergePdfs,
    splitPdf,
    deletePages,
    extractPages,
    compressPdf,
    pdfToImages,
    imagesToPdf,
    rotatePages,
    reorderPages,
    openFolder,
    selectPdfFiles,
    selectImageFiles,
    selectOutputFile,
    selectOutputDir,
    formatFileSize,
    copyToClipboard,
  };
}
