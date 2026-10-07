import { WidgetType } from '@codemirror/view'
import hljs from 'highlight.js/lib/common'

/** A fenced block drawn highlighted. */
export class CodeBlockWidget extends WidgetType {
  constructor(
    readonly code: string,
    readonly language: string
  ) {
    super()
  }

  override eq(other: CodeBlockWidget): boolean {
    return other.code === this.code && other.language === this.language
  }

  override toDOM(): HTMLElement {
    const frame = document.createElement('div')
    frame.className = 'cm-rk-codeblock'
    const pre = frame.appendChild(document.createElement('pre'))
    const code = pre.appendChild(document.createElement('code'))
    code.className = 'hljs'
    // highlight.js escapes the source it marks up.
    code.innerHTML = highlighted(this.code, this.language)
    return frame
  }
  // A click places the caret here, which shows the source.
  override ignoreEvent(): boolean {
    return false
  }
}

function highlighted(code: string, language: string): string {
  if (language && hljs.getLanguage(language))
    return hljs.highlight(code, { language, ignoreIllegals: true }).value
  return hljs.highlight(code, { language: 'plaintext' }).value
}
