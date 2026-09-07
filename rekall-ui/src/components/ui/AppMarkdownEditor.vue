<script setup lang="ts">
import { computed, onBeforeUnmount, ref, useId } from 'vue'
import { MdEditor, MdPreview, type ExposeParam, type ToolbarNames } from 'md-editor-v3'
import '@/common/config/markdown-editor'

const props = withDefaults(
  defineProps<{
    modelValue: string
    readonly?: boolean
    height?: string
    placeholder?: string
    showPreview?: boolean
  }>(),
  { readonly: false, height: '420px', placeholder: '# Contesto', showPreview: true }
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

const body = computed({
  get: () => props.modelValue,
  set: (value: string) => emit('update:modelValue', value)
})

const editor = ref<ExposeParam | null>(null)

// md-editor-v3 6.5.6 never destroys its CodeMirror view on unmount; without this each pane swap leaks the detached editor DOM.
onBeforeUnmount(() => editor.value?.getEditorView()?.destroy())

// Unique per instance: the library keys its global event bus on this id, and two previews sharing one clear each other's entries on unmount.
const editorId = `rekall-md-${useId()}`

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
      :editor-id="editorId"
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
      ref="editor"
      v-model="body"
      :editor-id="editorId"
      language="en-US"
      theme="dark"
      preview-theme="github"
      code-theme="atom"
      :toolbars="TOOLBARS"
      :preview="showPreview"
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

.rekall-md:not(.rekall-md--readonly) {
  height: 100%;
  min-height: 0;
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

.rekall-md .md-editor-preview {
  word-break: normal;
  overflow-wrap: break-word;
}

.rekall-md .md-editor-preview ul {
  list-style: disc;
  padding-left: 1.35em;
}

.rekall-md .md-editor-preview ol {
  list-style: decimal;
  padding-left: 1.35em;
}

.rekall-md .md-editor-preview li::marker {
  color: var(--color-text-subtle);
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
  border: none;
}
</style>
