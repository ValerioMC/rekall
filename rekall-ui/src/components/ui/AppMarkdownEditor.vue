<script setup lang="ts">
import { computed } from 'vue'
import { MdEditor, MdPreview, type ToolbarNames } from 'md-editor-v3'
import '@/common/config/markdown-editor'

/**
 * The markdown surface for document bodies, in both of its states.
 *
 * `readonly` renders the same markdown pipeline as the editor's own preview pane, so what a
 * note looks like while it is being written is what it looks like once saved. The alternative,
 * a separate renderer for the read state, is two implementations of one thing that drift.
 */
const props = withDefaults(
  defineProps<{
    modelValue: string
    readonly?: boolean
    /** CSS length. Ignored in readonly, where the preview grows with its content. */
    height?: string
    placeholder?: string
  }>(),
  { readonly: false, height: '420px', placeholder: '# Contesto' }
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

const body = computed({
  get: () => props.modelValue,
  set: (value: string) => emit('update:modelValue', value)
})

/**
 * Everything here is satisfied by code bundled with the application.
 *
 * The omissions are deliberate rather than a matter of taste: `fullscreen` needs screenfull,
 * `prettier` needs prettier, `mermaid` and `katex` need their own renderers, and each is
 * fetched from unpkg on first use. `pageFullscreen` is the same gesture implemented in CSS,
 * so the one button worth keeping is kept.
 */
const TOOLBARS: ToolbarNames[] = [
  'bold',
  'italic',
  'strikeThrough',
  '-',
  'title',
  'quote',
  'unorderedList',
  'orderedList',
  'task',
  '-',
  'codeRow',
  'code',
  'link',
  'image',
  'table',
  '-',
  'revoke',
  'next',
  '=',
  'pageFullscreen',
  'preview',
  'catalog'
]
</script>

<template>
  <div class="rekall-md" :class="{ 'rekall-md--readonly': readonly }">
    <MdPreview
      v-if="readonly"
      :model-value="modelValue"
      editor-id="rekall-preview"
      theme="dark"
      preview-theme="github"
      code-theme="atom"
      no-katex
      no-mermaid
      no-echarts
      no-img-zoom-in
    />
    <MdEditor
      v-else
      v-model="body"
      editor-id="rekall-editor"
      language="en-US"
      theme="dark"
      preview-theme="github"
      code-theme="atom"
      :toolbars="TOOLBARS"
      :footers="['markdownTotal', '=', 'scrollSwitch']"
      :style="{ height }"
      :placeholder="placeholder"
      no-katex
      no-mermaid
      no-echarts
      no-prettier
      no-upload-img
    />
  </div>
</template>

<style>
/**
 * The library ships its own dark palette, which is a different dark from this application's.
 * Rather than restyle its internals selector by selector, its theme variables are pointed at
 * the design tokens: one mapping, and the editor follows the rest of the UI from then on.
 */
.rekall-md .md-editor,
.rekall-md .md-editor-preview-wrapper {
  --md-bk-color: var(--color-surface);
  --md-color: var(--color-text);
  --md-border-color: var(--color-border);
  --md-hover-color: var(--color-text);
  --md-bk-color-outstand: var(--color-surface-raised);
  --md-bk-hover-color: var(--color-surface-hover);
  --md-scrollbar-bg-color: var(--color-canvas);
  --md-scrollbar-thumb-color: var(--color-border-strong);
  --md-scrollbar-thumb-hover-color: var(--color-text-subtle);
  --md-scrollbar-thumb-active-color: var(--color-text-subtle);
}

.rekall-md .md-editor {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-control);
  overflow: hidden;
}

.rekall-md .md-editor-toolbar-item:hover {
  color: var(--color-accent);
}

.rekall-md .md-editor-preview {
  font-size: 13.5px;
  line-height: 1.7;
  color: var(--color-text-muted);
}

.rekall-md .md-editor-preview h1,
.rekall-md .md-editor-preview h2,
.rekall-md .md-editor-preview h3,
.rekall-md .md-editor-preview h4 {
  color: var(--color-text);
  border-bottom: none;
}

.rekall-md .md-editor-preview a {
  color: var(--color-accent);
}

.rekall-md .md-editor-preview code {
  color: var(--color-accent);
}

.rekall-md .md-editor-preview pre code {
  color: inherit;
}

.rekall-md .md-editor-preview table th {
  background: var(--color-surface-raised);
  color: var(--color-text);
}

.rekall-md--readonly .md-editor-preview {
  padding: 4px 18px !important;
}

.rekall-md--readonly .md-editor-previewOnly {
  background: transparent;
}
</style>
