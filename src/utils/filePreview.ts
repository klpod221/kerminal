import type { FileEntry } from "../types/sftp";

/**
 * Set of file extensions that can be previewed in the built-in file viewer.
 * Office formats (doc, xls, ppt, …) are included but only show metadata,
 * not actual content.
 */
export const PREVIEWABLE_EXTENSIONS = new Set([
  // Images
  "jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico",
  // Video
  "mp4", "webm", "ogg", "avi", "mov", "wmv", "flv", "mkv",
  // Documents / markup
  "html", "htm", "md", "markdown", "pdf",
  // Plain text & config
  "txt", "text", "log",
  "json", "yaml", "yml", "xml", "csv", "tsv",
  "ini", "conf", "config", "cfg", "properties", "env",
  "gitignore", "gitattributes", "dockerfile", "makefile",
  // Shell scripts
  "sh", "bash", "zsh", "fish", "ps1", "bat", "cmd",
  // Web
  "js", "ts", "jsx", "tsx", "vue", "svelte",
  "css", "scss", "sass", "less", "styl", "stylus",
  // Programming languages
  "py", "java", "c", "cpp", "cc", "h", "hpp",
  "cs", "go", "rs", "rb", "php", "swift", "kt", "scala",
  "clj", "hs", "ml", "sql", "r", "m", "pl", "pm", "lua", "vim",
  // Office (metadata-only preview)
  "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp",
]);

/**
 * Returns true when the file has an extension supported by the built-in
 * file preview modal.
 *
 * @param file - The remote or local file entry to check.
 */
export function canPreviewFile(file: FileEntry): boolean {
  if (file.fileType !== "file") return false;
  const ext = file.name.split(".").pop()?.toLowerCase();
  return ext ? PREVIEWABLE_EXTENSIONS.has(ext) : false;
}
