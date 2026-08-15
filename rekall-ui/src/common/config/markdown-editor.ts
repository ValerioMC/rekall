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
 */
config({
  editorExtensions: { highlight: { instance: hljs } }
})
