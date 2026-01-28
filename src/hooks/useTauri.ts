import { invoke } from '@tauri-apps/api/core';
import { save, open } from '@tauri-apps/plugin-dialog';

export interface PdfInfo {
  page_count: number;
  file_size: number;
  title: string | null;
  author: string | null;
}

export interface ProcessResult {
  success: boolean;
  message: string;
  output_path: string | null;
}

export function useTauri() {
  const getPdfInfo = async (filePath: string): Promise<PdfInfo> => {
    return await invoke('get_pdf_info', { filePath });
  };

  const mergePdfs = async (
    filePaths: string[],
    outputPath: string
  ): Promise<ProcessResult> => {
    return await invoke('merge_pdfs', { filePaths, outputPath });
  };

  const splitPdf = async (
    filePath: string,
    ranges: [number, number][],
    outputDir: string
  ): Promise<ProcessResult> => {
    return await invoke('split_pdf', { filePath, ranges, outputDir });
  };

  const deletePages = async (
    filePath: string,
    pagesToDelete: number[],
    outputPath: string
  ): Promise<ProcessResult> => {
    return await invoke('delete_pages', { filePath, pagesToDelete, outputPath });
  };

  const extractPages = async (
    filePath: string,
    pagesToExtract: number[],
    outputPath: string
  ): Promise<ProcessResult> => {
    return await invoke('extract_pages', { filePath, pagesToExtract, outputPath });
  };

  const compressPdf = async (
    filePath: string,
    outputPath: string,
    quality: number
  ): Promise<ProcessResult> => {
    return await invoke('compress_pdf', { filePath, outputPath, quality });
  };

  const pdfToImages = async (
    filePath: string,
    outputDir: string,
    format: string
  ): Promise<ProcessResult> => {
    return await invoke('pdf_to_images', { filePath, outputDir, format });
  };

  const imagesToPdf = async (
    imagePaths: string[],
    outputPath: string
  ): Promise<ProcessResult> => {
    return await invoke('images_to_pdf', { imagePaths, outputPath });
  };

  const rotatePages = async (
    filePath: string,
    pages: number[],
    rotation: number,
    outputPath: string
  ): Promise<ProcessResult> => {
    return await invoke('rotate_pages', { filePath, pages, rotation, outputPath });
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
    selectPdfFiles,
    selectImageFiles,
    selectOutputFile,
    selectOutputDir,
  };
}
