<script setup lang="ts">
import { computed, onBeforeUnmount, ref, useId, watch } from 'vue'
import { MdEditor, MdPreview, type ExposeParam, type ToolbarNames } from 'md-editor-v3'
import '@/common/config/markdown-editor'
import {
  READ_WIDTH_MAX,
  READ_WIDTH_MIN,
  readModeAlign,
  readModeWidth,
  setReadModeAlign,
  setReadModeWidth
} from '@/common/config/read-mode'

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

// The reading column's width and alignment, shared across every Read-mode pane via localStorage.
const readWidth = ref(readModeWidth())
const readAlign = ref(readModeAlign())
const readSettingsOpen = ref(false)

watch(readWidth, (value) => setReadModeWidth(value))
watch(readAlign, (value) => setReadModeAlign(value))

const readStyle = computed(() => ({ '--rk-read-width': `${readWidth.value}px` }))

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
  <div
    class="rekall-md"
    :class="{ 'rekall-md--readonly': readonly }"
    :style="readonly ? readStyle : undefined"
    :data-align="readonly ? readAlign : undefined"
  >
    <div v-if="readonly" class="rekall-read-controls">
      <div class="relative">
        <button
          type="button"
          class="focus-ring flex size-7 items-center justify-center rounded-full border border-border bg-surface text-[11px] font-semibold text-text-subtle shadow-sm transition-colors hover:text-text"
          :aria-expanded="readSettingsOpen"
          aria-label="Reading settings"
          data-testid="read-settings-toggle"
          @click="readSettingsOpen = !readSettingsOpen"
        >
          Aa
        </button>
        <div
          v-if="readSettingsOpen"
          class="absolute right-0 top-9 z-10 w-56 rounded-[10px] border border-border bg-surface-raised p-3 shadow-lg"
          data-testid="read-settings-panel"
        >
          <div class="flex items-center justify-between gap-2">
            <span class="text-[11px] font-medium text-text-subtle">Line width</span>
            <span class="text-[11px] tabular-nums text-text-subtle">{{ readWidth }}px</span>
          </div>
          <input
            v-model.number="readWidth"
            type="range"
            class="mt-1.5 w-full accent-accent"
            :min="READ_WIDTH_MIN"
            :max="READ_WIDTH_MAX"
            step="20"
            aria-label="Reading line width"
            data-testid="read-width-slider"
          />

          <p class="mt-3 text-[11px] font-medium text-text-subtle">
            Align
          </p>
          <div class="mt-1.5 flex gap-0.5 rounded-[7px] bg-canvas p-0.5">
            <button
              v-for="option in (['left', 'center'] as const)"
              :key="option"
              type="button"
              class="focus-ring h-6 flex-1 rounded-[5px] text-[11px] capitalize transition-colors"
              :class="readAlign === option ? 'bg-surface-raised text-text' : 'text-text-subtle hover:text-text'"
              :aria-pressed="readAlign === option"
              :data-testid="`read-align-${option}`"
              @click="readAlign = option"
            >
              {{ option }}
            </button>
          </div>
        </div>
      </div>
    </div>
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

/*
 * The writing caret, made worth watching: wider than CodeMirror's default hairline and lit with
 * a soft accent glow that breathes rather than the library's hard on/off blink, so the point
 * you're about to type at is never just a thin line disappearing and reappearing.
 */
.rekall-md:not(.rekall-md--readonly) .cm-cursor {
  border-left-width: 2px !important;
  border-left-color: var(--color-accent-strong) !important;
  animation: caret-glow 1.1s ease-in-out infinite;
}

@keyframes caret-glow {
  0%,
  100% {
    box-shadow: 0 0 3px 0 color-mix(in srgb, var(--color-accent) 45%, transparent);
  }
  50% {
    box-shadow: 0 0 8px 1px color-mix(in srgb, var(--color-accent) 85%, transparent);
  }
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

.rekall-md--readonly {
  display: flex;
  flex-direction: column;
}

.rekall-md--readonly .md-editor-preview {
  width: 100%;
  max-width: var(--rk-read-width, 680px);
  margin-inline: auto;
  padding: 40px 24px 56px !important;
  font-size: 14.5px !important;
  line-height: 1.75 !important;
  box-sizing: border-box;
  transition: max-width 0.12s ease-out;
}

.rekall-md--readonly[data-align='left'] .md-editor-preview {
  margin-inline: 0;
  margin-left: clamp(16px, 4vw, 56px);
}

.rekall-md--readonly .md-editor-previewOnly {
  background: transparent;
  border: none;
}

.rekall-read-controls {
  align-self: flex-end;
  position: sticky;
  top: 12px;
  z-index: 10;
  margin: 8px 12px 0 0;
}
</style>
