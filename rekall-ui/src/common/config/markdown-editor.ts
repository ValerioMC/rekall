import hljs from 'highlight.js/lib/common'
import { config } from 'md-editor-v3'
import 'md-editor-v3/lib/style.css'
import 'highlight.js/styles/github-dark.css'

/**
 * Editor configuration, imported for its side effect by the component that mounts it.
 *
 * md-editor-v3 lazily pulls highlight.js from unpkg the first time it renders a fenced block.
 * Rekall runs on localhost against a file database and is meant to work with no network at
 * all, so a syntax-highlighted note would silently fall back to plain text on a train. Passing
 * a local instance suppresses both the script tag and the stylesheet link: the library injects
 * them only when `instance` is unset.
 *
 * `highlight.js/lib/common` carries the ~40 languages that appear in notes rather than the
 * full set of 190, which is most of the difference in what the bundle costs.
 *
 * A single newline is not a line break. The library turns markdown-it's `breaks` on, so every
 * newline in the source becomes a `<br>`. A brief written by an agent or pasted from an editor
 * is hard-wrapped at eighty columns, and with that option on it renders wrapped at eighty
 * columns forever: the text stops mid-pane and starts a new line while the width sits empty.
 * Off, a paragraph is a paragraph and reflows to whatever the pane gives it. A deliberate break
 * is still two spaces or a blank line, which is what markdown means by one everywhere else.
 *
 * The `linkShortener` extension is dropped. It exists so a pasted three-hundred-character URL
 * does not eat half the editor, and for URLs that is a fair trade. But one of the four patterns
 * it matches is any run of non-space characters starting with a single slash, which is every
 * absolute path a technical note ever mentions. Over thirty characters the path is swapped for
 * an uneditable `...` widget: the cursor cannot enter it, so correcting a typo halfway along
 * `/Users/.../application.yaml` means deleting the leading slash to dissolve the decoration
 * first. A path in a note is prose, not a link, and prose has to stay editable.
 */
config({
  editorExtensions: { highlight: { instance: hljs } },
  markdownItConfig: (md) => {
    md.set({ breaks: false })
  },
  codeMirrorExtensions: (extensions) =>
    extensions.filter((extension) => extension.type !== 'linkShortener')
})
