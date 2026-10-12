import hljs from 'highlight.js/lib/common'
import mermaid from 'mermaid'
import { config } from 'md-editor-v3'
import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { languages } from '@codemirror/language-data'
import 'md-editor-v3/lib/style.css'
import 'highlight.js/styles/github-dark.css'
import '@/assets/markdown-editor.css'
import '@/assets/live-preview.css'
import { markAnchorCode } from '@/common/config/anchor-code'
import { markComponentReferences } from '@/common/markdown/component-reference'
import { livePreview } from '@/common/live-preview/live-preview'

config({
  // Local instances so md-editor-v3 does not lazy-fetch highlight.js or mermaid from a CDN.
  editorExtensions: { highlight: { instance: hljs }, mermaid: { instance: mermaid } },
  markdownItConfig: (md) => {
    // Off, or every source newline in a hard-wrapped brief renders as a permanent <br>.
    md.set({ breaks: false })
    markAnchorCode(md)
    markComponentReferences(md)
  },
  // linkShortener turns long slash-prefixed runs into an uneditable widget, which eats file paths.
  // The library parses CommonMark only; live preview needs GFM to see tables, tasks and strikethrough.
  codeMirrorExtensions: (extensions) => [
    ...extensions
      .filter((extension) => extension.type !== 'linkShortener')
      .map((extension) =>
        extension.type === 'markdown'
          ? { ...extension, extension: markdown({ base: markdownLanguage, codeLanguages: languages }) }
          : extension
      ),
    { type: 'livePreview', extension: livePreview() }
  ]
})
