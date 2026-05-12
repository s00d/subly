/**
 * Filesystem helpers used by the AI dropzone.
 *
 * Tauri 2 native drag-and-drop bypasses the WebView's HTML5 `drop` event —
 * the frontend only receives file *paths* through
 * `getCurrentWebview().onDragDropEvent`. To turn a path into bytes we use
 * the official `@tauri-apps/plugin-fs` JS API; the corresponding Rust
 * plugin is already registered and the capability set allows
 * `fs:allow-read-file` for `**`, so no extra wiring is required.
 */
import { readFile } from "@tauri-apps/plugin-fs";

/**
 * Hard cap on a single dropped file. Checked after `readFile` resolves —
 * we don't `stat()` first to avoid an extra `fs:allow-stat` permission;
 * the OS already buffered the file before handing us the bytes, so an
 * upper-bound check post-read is "good enough" for the dialog UX.
 */
const MAX_FILE_BYTES = 64 * 1024 * 1024; // 64 MiB

const MIME_BY_EXT: Record<string, string> = {
  csv: "text/csv",
  xls: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  xlsx: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  pdf: "application/pdf",
  json: "application/json",
  txt: "text/plain",
  png: "image/png",
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  webp: "image/webp",
  gif: "image/gif",
  heic: "image/heic",
  heif: "image/heic",
};

export interface DroppedFile {
  bytes: Uint8Array;
  name: string;
  mime: string;
  size: number;
}

function basename(path: string): string {
  const norm = path.replace(/\\/g, "/");
  const idx = norm.lastIndexOf("/");
  return idx >= 0 ? norm.slice(idx + 1) : norm;
}

function mimeFromName(name: string): string {
  const dot = name.lastIndexOf(".");
  if (dot < 0) return "application/octet-stream";
  const ext = name.slice(dot + 1).toLowerCase();
  return MIME_BY_EXT[ext] ?? "application/octet-stream";
}

/**
 * Read the file at `path` (typically supplied by a Tauri drag-drop event)
 * and return its bytes plus best-effort metadata. Throws if the path can't
 * be read or the result exceeds the size cap.
 */
export async function fsReadDroppedFile(path: string): Promise<DroppedFile> {
  const bytes = await readFile(path);
  if (bytes.byteLength > MAX_FILE_BYTES) {
    throw new Error(`file_too_large:${bytes.byteLength}:${MAX_FILE_BYTES}`);
  }
  const name = basename(path);
  return {
    bytes,
    name,
    mime: mimeFromName(name),
    size: bytes.byteLength,
  };
}
