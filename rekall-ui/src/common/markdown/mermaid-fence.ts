import type { config } from 'md-editor-v3'

type MarkdownIt = Parameters<NonNullable<Parameters<typeof config>[0]['markdownItConfig']>>[0]

export const MERMAID_FENCE_CLASS = 'rk-mermaid-fence'
export const MERMAID_FENCE_FAILED_CLASS = 'rk-mermaid-fence--failed'

/**
 * Renders a mermaid fence as an empty frame carrying its source, so a read view can draw it the
 * way the Diagram section does once the page is in the DOM (`useMermaidFences`).
 */
export function markMermaidFences(md: MarkdownIt): void {
  md.core.ruler.after('block', 'rk-mermaid-fence', (state) => {
    for (const token of state.tokens) {
      if (token.type !== 'fence' || token.info.trim() !== 'mermaid') continue
      const source = encodeURIComponent(token.content.trimEnd())
      token.type = 'html_block'
      token.tag = ''
      token.info = ''
      token.content = `<div class="${MERMAID_FENCE_CLASS}" data-source="${source}"></div>\n`
    }
    return true
  })
}

export function sourceOfMermaidFrame(frame: HTMLElement): string {
  return decodeURIComponent(frame.dataset['source'] ?? '')
}

export function describeMermaidFailure(failure: unknown): string {
  return `Mermaid could not draw this diagram: ${failure instanceof Error ? failure.message : String(failure)}`
}
