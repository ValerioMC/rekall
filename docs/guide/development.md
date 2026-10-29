# Development

## Frontend stack

| Concern | Choice |
|---|---|
| Framework | Vue 3, Composition API, `<script setup lang="ts">` |
| Shell | One surface, three panes. No router |
| Build | Vite 5 |
| Styling | Tailwind CSS 4, semantic tokens in `src/assets/main.css` |
| Fonts | Fira Sans and Fira Code, bundled |
| State | Pinia setup stores; `console.store` holds the working set, one slice per concern, each in its feature (`features/<feature>/store/`) |
| HTTP | `ofetch`, timeout, retry on 5xx only, correlation id per request |
| Validation | Zod on every response |
| Markdown | `md-editor-v3` wrapped by `AppMarkdownEditor`, highlight.js and mermaid passed in as local instances. Write mode is a CodeMirror live preview (`common/live-preview/`) on md-editor-v3's editor, with no side preview; read mode is md-editor-v3's preview. A ```` ```mermaid ```` fence is drawn as a diagram in both |
| Tests | Vitest + Vue Test Utils |

```
src/
├── model/        domain types and branded ids
├── api/          http calls and Zod schemas, no state
├── stores/       Pinia
├── composables/  reusable logic without UI
└── components/
    ├── ui/       atomic and presentational
    └── console/  the three panes, the anchor bar, the editors
```

## Debugging

`RUST_LOG=rekall=debug make run` turns on debug logging. The server is a plain binary, so any Rust debugger attaches to it (`rust-lldb target/debug/rekall-server`, or the CodeLLDB extension in VS Code).

