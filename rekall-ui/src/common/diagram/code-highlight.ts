import hljs from 'highlight.js/lib/common'
import 'highlight.js/styles/github-dark.css'

/**
 * One line of code as coloured tokens rather than an HTML string, so the excerpt renders text
 * nodes only. highlight.js escapes the source; its markup is read back here, never injected.
 */
export interface CodeToken {
  readonly text: string
  readonly classes: string
}

export function highlightLine(text: string, language: string | null): CodeToken[] {
  if (!language || !hljs.getLanguage(language) || typeof DOMParser === 'undefined') return [{ text, classes: '' }]
  const html = hljs.highlight(text, { language, ignoreIllegals: true }).value
  const body = new DOMParser().parseFromString(`<body>${html}</body>`, 'text/html').body
  const tokens: CodeToken[] = []
  collect(body, [], tokens)
  return tokens
}

function collect(node: Node, classes: readonly string[], into: CodeToken[]): void {
  node.childNodes.forEach((child) => {
    if (child.nodeType === Node.TEXT_NODE) {
      into.push({ text: child.textContent ?? '', classes: classes.join(' ') })
      return
    }
    if (child instanceof Element) collect(child, [...classes, ...Array.from(child.classList)], into)
  })
}
