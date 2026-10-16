/**
 * A Mermaid label as plain text: line breaks become spaces, markup and entities are dropped,
 * since the console draws text nodes and never HTML. Mermaid's `~T~` generics read as `<T>`.
 */
export function plainLabel(raw: string | undefined | null): string {
  if (!raw) return ''
  return raw
    .replace(/<br\s*\/?>/gi, ' ')
    .replace(/<[^>]*>/g, '')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&amp;/g, '&')
    .replace(/~([^~]+)~/g, '<$1>')
    .replace(/\s+/g, ' ')
    .trim()
}
