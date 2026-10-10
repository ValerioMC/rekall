import type { SourceLocation } from '@/model/diagram'

/** `L12–30`, `L12` or `whole file`: the short form a source is listed under. */
export function spanLabel(source: SourceLocation): string {
  if (source.startLine === null) return 'whole file'
  if (source.endLine === null || source.endLine === source.startLine) return `L${source.startLine}`
  return `L${source.startLine}–${source.endLine}`
}

/** `src/pty/manager.rs` → `src/pty/`: shown dimmed and cut first, so the file name survives. */
export function folderOf(path: string): string {
  const at = path.lastIndexOf('/')
  return at < 0 ? '' : path.slice(0, at + 1)
}

export function baseOf(path: string): string {
  return path.slice(path.lastIndexOf('/') + 1)
}
