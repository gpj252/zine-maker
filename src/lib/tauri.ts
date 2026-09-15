import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { Project, ImpositionResult } from "../types";

export function getImposition(project: Project): Promise<ImpositionResult> {
  return invoke("get_imposition", { project });
}

export function exportProjectPdf(project: Project, outputPath: string): Promise<void> {
  return invoke("export_project_pdf", { project, outputPath });
}

const IMAGE_EXTENSIONS = ["png", "jpg", "jpeg", "webp", "tif", "tiff", "bmp"];

/** Returns the absolute paths the user picked, or null if they cancelled. */
export async function pickImages(): Promise<string[] | null> {
  const result = await open({
    multiple: true,
    filters: [{ name: "Images", extensions: IMAGE_EXTENSIONS }],
  });
  if (result == null) return null;
  return Array.isArray(result) ? result : [result];
}

export function pickSaveLocation(defaultName: string): Promise<string | null> {
  return save({
    defaultPath: defaultName,
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
}

/** Local file paths can't be used as <img src> directly in the Tauri webview — this converts
 *  them to the special asset:// URL scheme that can. */
export function assetUrl(path: string): string {
  return convertFileSrc(path);
}
