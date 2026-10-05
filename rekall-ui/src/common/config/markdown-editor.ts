import hljs from 'highlight.js/lib/common'
import mermaid from 'mermaid'
import { config } from 'md-editor-v3'
import 'md-editor-v3/lib/style.css'
import 'highlight.js/styles/github-dark.css'
import { markAnchorCode } from '@/common/config/anchor-code'

config({
  // Local instances so md-editor-v3 does not lazy-fetch highlight.js or mermaid from a CDN.
  editorExtensions: { highlight: { instance: hljs }, mermaid: { instance: mermaid } },
  markdownItConfig: (md) => {
    // Off, or every source newline in a hard-wrapped brief renders as a permanent <br>.
    md.set({ breaks: false })
    markAnchorCode(md)
  },
  // linkShortener turns long slash-prefixed runs into an uneditable widget, which eats file paths.
  codeMirrorExtensions: (extensions) =>
    extensions.filter((extension) => extension.type !== 'linkShortener')
})
