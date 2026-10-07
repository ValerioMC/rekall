import MarkdownIt from 'markdown-it'
import { markAnchorCode } from '@/common/config/anchor-code'

// The read view's inline rules (anchor-coloured code, no soft breaks), without raw html.
const renderer = new MarkdownIt({ html: false, breaks: false })
markAnchorCode(renderer)

/** A table cell's markdown as the html the read view would give it. */
export function renderInlineMarkdown(text: string): string {
  return renderer.renderInline(text)
}
