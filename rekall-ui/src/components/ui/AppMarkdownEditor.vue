<script setup lang="ts">
import { computed, onBeforeUnmount, ref, useId } from 'vue'
import { MdEditor, MdPreview, type ExposeParam, type ToolbarNames } from 'md-editor-v3'
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
    /**
     * Whether the editor is split with a live preview beside it.
     *
     * On a full pane the split is the point: the markdown and what it becomes, together. Inside
     * a card that is half a pane wide it is two cramped columns instead of one usable one, and
     * the surface it sits on already has a read state of its own.
     */
    showPreview?: boolean
  }>(),
  { readonly: false, height: '420px', placeholder: '# Contesto', showPreview: true }
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

const body = computed({
  get: () => props.modelValue,
  set: (value: string) => emit('update:modelValue', value)
})

/**
 * The editor is unmounted every time a pane is swapped, and md-editor-v3 6.5.6 builds its
 * CodeMirror view in `onMounted` without ever destroying it: the view keeps its document-level
 * listeners, and through them the whole detached editor DOM. Roughly three thousand nodes and a
 * megabyte per swap in Chrome, five megabytes in the WebKit view the macOS app runs in, which
 * is how a window left open all day reaches hundreds of megabytes. Until the library does it,
 * the view is destroyed here.
 */
const editor = ref<ExposeParam | null>(null)

onBeforeUnmount(() => editor.value?.getEditorView()?.destroy())

/**
 * One id per instance, the way the library assigns them when none is given.
 *
 * The id is what md-editor-v3 keys its global event bus on and what it puts on the root
 * element. Two previews on screen at once, which the steps pane does whenever more than one
 * step is open in read mode, shared one id: duplicate ids in the document, and unmounting any
 * one of them cleared the bus entries of the others, because the library's unmount clears by id.
 */
const editorId = `rekall-md-${useId()}`

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

/*
 * The editor scrolls its own text, so the toolbar stays where the hand expects it. Without a
 * resolved height the `height: 100%` handed to the editor falls back to auto, the editor grows
 * with the note, and the pane scrolls the formatting buttons off the top.
 */
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

/*
 * The library's github theme breaks lines mid-word: a rule that suits CJK, and one that turns Latin
 * prose into what looks like a typo ("e lo pubbl / ica su S3").
 */
.rekall-md .md-editor-preview {
  word-break: normal;
  overflow-wrap: break-word;
}

/* Tailwind's reset strips list markers, and half of any brief is bullets. */
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

/* Read as a document, not as a boxed field: nothing frames the text but the pane it is in. */
.rekall-md--readonly .md-editor-previewOnly {
  background: transparent;
  border: none;
}
</style>
