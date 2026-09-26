<script setup lang="ts">
import { computed, onBeforeUnmount, ref, useId, watch } from 'vue'
import { MdEditor, MdPreview, NormalToolbar, type ExposeParam, type ToolbarNames } from 'md-editor-v3'
import '@/common/config/markdown-editor'
import PathPicker, { type CaretBox } from '@/components/ui/PathPicker.vue'
import { formatPathForMarkdown } from '@/common/path/path-query'
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
    /**
     * Read inside something else (a step's detail under its title) rather than as the pane: the
     * page sits flush left at the container's width with no reading-column controls, since the
     * width and alignment they set belong to a pane-sized page, not to a card.
     */
    compact?: boolean
    /**
     * The folder the path picker opens in: the project's, when the text belongs to one. Without
     * it the picker opens in the home folder, and it can reach anywhere on the machine either way.
     */
    pathRoot?: string | null
  }>(),
  {
    readonly: false,
    height: '420px',
    placeholder: '# Contesto',
    showPreview: true,
    compact: false,
    pathRoot: null
  }
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

type EditorView = NonNullable<ReturnType<ExposeParam['getEditorView']>>

function editorView(): EditorView | null {
  return editor.value?.getEditorView() ?? null
}

/*
 * The path picker. It is opened over the selection as it stands (⌘⇧P, or the toolbar's folder)
 * and writes into exactly that range when a path is chosen, through CodeMirror, so the insert is
 * one undo step and the v-model hears about it the way it hears about typing.
 */
const pickerOpen = ref(false)
/** The folder the picker was last left in, so a second path from the same place is one keystroke. */
const pickerStart = ref<string | null>(null)
let pickerRange: { from: number; to: number } | null = null
let pickerTextBefore = ''

watch(
  () => props.pathRoot,
  () => {
    pickerStart.value = null
  }
)

function openPicker(): void {
  const view = editorView()
  if (!view || props.readonly) return
  const { from, to } = view.state.selection.main
  pickerRange = { from, to }
  pickerTextBefore = view.state.sliceDoc(0, from)
  pickerOpen.value = true
}

function closePicker(): void {
  pickerOpen.value = false
  editorView()?.focus()
}

function togglePicker(): void {
  if (pickerOpen.value) closePicker()
  else openPicker()
}

/** Where the path will land, held inside the writing area when that point has scrolled out of it. */
function locateCaret(): CaretBox | null {
  const view = editorView()
  if (!view || !pickerRange) return null
  const frame = view.scrollDOM.getBoundingClientRect()
  const coords = view.coordsAtPos(pickerRange.from)
  if (!coords) return { left: frame.left + 18, top: frame.top, bottom: frame.top + 20 }
  const top = Math.min(Math.max(coords.top, frame.top), frame.bottom - 20)
  return { left: coords.left, top, bottom: top + (coords.bottom - coords.top) }
}

function formatPick(path: string, directory: boolean): string {
  return formatPathForMarkdown(path, directory, pickerTextBefore)
}

function insertPath(path: string, directory: boolean): void {
  const view = editorView()
  const range = pickerRange
  closePicker()
  if (!view || !range) return
  const text = formatPick(path, directory)
  view.dispatch({
    changes: { from: range.from, to: range.to, insert: text },
    selection: { anchor: range.from + text.length },
    scrollIntoView: true,
    userEvent: 'input.paste'
  })
}

function onEditorKeydown(event: KeyboardEvent): void {
  if (props.readonly || !(event.metaKey || event.ctrlKey) || !event.shiftKey || event.code !== 'KeyP') return
  event.preventDefault()
  event.stopPropagation()
  togglePicker()
}

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
  0,
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
    :class="{ 'rekall-md--readonly': readonly, 'rekall-md--compact': readonly && compact }"
    :style="readonly && !compact ? readStyle : undefined"
    :data-align="readonly && !compact ? readAlign : undefined"
    @keydown.capture="onEditorKeydown"
  >
    <div v-if="readonly && !compact" class="rekall-read-controls">
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
      language="en-US"
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
    >
      <template #defToolbars>
        <NormalToolbar
          title="Insert a path  ⌘⇧P"
          aria-label="Insert a path"
          :aria-expanded="pickerOpen"
          data-path-picker-trigger
          data-testid="path-picker-trigger"
          :class="{ 'md-editor-toolbar-active': pickerOpen }"
          @on-click="togglePicker"
        >
          <!-- A folder with the slash of a path through it: this puts a path, not a file, in the text. -->
          <svg class="md-editor-icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              d="M3 6.5A1.5 1.5 0 0 1 4.5 5h4.4l2 2.2h8.6A1.5 1.5 0 0 1 21 8.7v9.8a1.5 1.5 0 0 1-1.5 1.5h-15A1.5 1.5 0 0 1 3 18.5v-12Z"
              stroke="currentColor"
              stroke-width="1.6"
              stroke-linejoin="round"
            />
            <path d="m13.6 10.4-3.2 6.2" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
          </svg>
        </NormalToolbar>
      </template>
    </MdEditor>

    <Teleport to="body">
      <Transition name="popover">
        <PathPicker
          v-if="pickerOpen"
          :root="pathRoot"
          :start="pickerStart"
          :locate="locateCaret"
          :format="formatPick"
          @pick="insertPath"
          @visit="pickerStart = $event"
          @close="closePicker"
        />
      </Transition>
    </Teleport>
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

/*
 * The page.
 *
 * Everything Rekall hands a session is read here first: a brief, a wrapup, a note, a step's
 * detail. So it is typeset rather than dropped into the library's GitHub theme: the interface
 * face instead of the system stack, a measure held by the reading column, a scale that steps by
 * a fifth, and headings that hang a short tick into the left margin so a long wrapup can be
 * scanned by its sections without a rule across every one of them.
 */
.rekall-md .md-editor,
.rekall-md .md-editor-previewOnly,
.rekall-md .md-editor-preview {
  font-family: var(--font-sans);
}

.rekall-md .md-editor-preview {
  --rk-prose: color-mix(in srgb, var(--color-text) 72%, var(--color-text-muted));
  font-size: 13.5px;
  line-height: 1.7;
  color: var(--rk-prose);
  word-break: normal;
  overflow-wrap: break-word;
  font-feature-settings: 'kern', 'liga', 'tnum' 0;
}

.rekall-md .md-editor-preview > :first-child {
  margin-top: 0;
}

.rekall-md .md-editor-preview h1,
.rekall-md .md-editor-preview h2,
.rekall-md .md-editor-preview h3,
.rekall-md .md-editor-preview h4 {
  position: relative;
  color: var(--color-text);
  border-bottom: none;
  padding-bottom: 0;
  font-weight: 600;
  line-height: 1.3;
}

.rekall-md .md-editor-preview h1 {
  font-size: 1.62em;
  letter-spacing: -0.018em;
  margin: 0 0 0.7em;
}

.rekall-md .md-editor-preview h2 {
  font-size: 1.2em;
  letter-spacing: -0.008em;
  margin: 2.1em 0 0.6em;
}

.rekall-md .md-editor-preview h3 {
  font-size: 1em;
  margin: 1.7em 0 0.45em;
}

.rekall-md .md-editor-preview h4 {
  font-size: 0.93em;
  color: var(--color-text-muted);
  margin: 1.5em 0 0.4em;
}

/* The section tick: a short bar hung in the margin, level with the heading's x-height. */
.rekall-md .md-editor-preview h2::before,
.rekall-md .md-editor-preview h3::before {
  content: '';
  position: absolute;
  top: 0.62em;
  left: -22px;
  height: 2px;
  width: 12px;
  border-radius: 2px;
  background: var(--color-border-strong);
}

.rekall-md .md-editor-preview h3::before {
  width: 7px;
  left: -17px;
}

.rekall-md .md-editor-preview p,
.rekall-md .md-editor-preview ul,
.rekall-md .md-editor-preview ol,
.rekall-md .md-editor-preview blockquote,
.rekall-md .md-editor-preview table,
.rekall-md .md-editor-preview pre {
  margin: 0 0 1em;
}

.rekall-md .md-editor-preview strong {
  color: var(--color-text);
  font-weight: 600;
}

.rekall-md .md-editor-preview ul {
  list-style: none;
  padding-left: 1.25em;
}

/* A short dash rather than a bullet: it sits on the x-height and reads as a list of statements. */
.rekall-md .md-editor-preview ul > li {
  position: relative;
}

.rekall-md .md-editor-preview ul > li::before {
  content: '';
  position: absolute;
  top: 0.82em;
  left: -1.05em;
  width: 0.5em;
  height: 1.5px;
  border-radius: 1px;
  background: var(--color-text-subtle);
}

.rekall-md .md-editor-preview ul.contains-task-list > li::before,
.rekall-md .md-editor-preview ul > li.task-list-item::before {
  display: none;
}

.rekall-md .md-editor-preview ol {
  list-style: decimal;
  padding-left: 1.4em;
}

.rekall-md .md-editor-preview ol > li::marker {
  color: var(--color-text-subtle);
  font-family: var(--font-mono);
  font-size: 0.86em;
}

.rekall-md .md-editor-preview li + li {
  margin-top: 0.3em;
}

.rekall-md .md-editor-preview li > ul,
.rekall-md .md-editor-preview li > ol {
  margin: 0.3em 0 0;
}

.rekall-md .md-editor-preview input[type='checkbox'] {
  accent-color: var(--color-accent);
  margin-right: 0.45em;
  translate: 0 1px;
}

.rekall-md .md-editor-preview a {
  color: var(--color-accent-strong);
  text-decoration: underline;
  text-decoration-color: color-mix(in srgb, var(--color-accent) 40%, transparent);
  text-underline-offset: 3px;
  text-decoration-thickness: 1px;
  transition: text-decoration-color 120ms ease;
}

.rekall-md .md-editor-preview a:hover {
  text-decoration-color: var(--color-accent-strong);
}

/*
 * Code is neutral and anchors are cyan, the split the tokens in main.css set out: amber is where
 * you are and what you are about to do, so it has no business on every identifier in a wrapup.
 */
.rekall-md .md-editor-preview code {
  font-family: var(--font-mono);
  font-size: 0.86em;
  color: var(--color-text);
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border);
  border-radius: 5px;
  padding: 0.1em 0.38em;
  white-space: break-spaces;
}

.rekall-md .md-editor-preview code.rk-anchor {
  color: var(--color-anchor);
  background: var(--color-anchor-soft);
  border-color: var(--color-anchor-line);
}

.rekall-md .md-editor-preview pre {
  background: var(--color-canvas);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-control);
}

.rekall-md .md-editor-preview pre code {
  color: inherit;
  background: transparent;
  border: none;
  padding: 0;
  font-size: 12.5px;
  white-space: pre;
}

/* md-editor-v3's default row-number gutter sits at `top: 1em`, a push-down it needs only in the
   bundled preview themes (vuepress, cyanosis, ...) that stack a header row inside the same <pre>.
   Ours (github) doesn't, so the un-patched 1em default leaves every gutter number a full line
   below the code line it labels. */
.rekall-md .md-editor-preview pre code span[rn-wrapper] {
  top: 0;
}

.rekall-md .md-editor-preview blockquote {
  border-left: 2px solid var(--color-border-strong);
  padding: 0.1em 0 0.1em 1em;
  color: var(--color-text-muted);
}

.rekall-md .md-editor-preview blockquote > :last-child,
.rekall-md .md-editor-preview li > p:last-child {
  margin-bottom: 0;
}

.rekall-md .md-editor-preview hr {
  height: 0;
  border: none;
  border-top: 1px solid var(--color-border);
  margin: 2em 0;
  background: none;
}

.rekall-md .md-editor-preview table {
  display: table;
  width: auto;
  border-collapse: separate;
  border-spacing: 0;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-control);
  overflow: hidden;
  font-size: 0.93em;
}

.rekall-md .md-editor-preview table th,
.rekall-md .md-editor-preview table td {
  border: none;
  border-bottom: 1px solid var(--color-border);
  padding: 0.45em 0.9em;
  text-align: left;
}

.rekall-md .md-editor-preview table tr:last-child td {
  border-bottom: none;
}

.rekall-md .md-editor-preview table tr {
  background: transparent;
  border: none;
}

.rekall-md .md-editor-preview table th {
  background: var(--color-surface-raised);
  color: var(--color-text);
  font-weight: 600;
}

/*
 * The writing surface: a quiet toolbar that lights only what is on, source set in the code face
 * at the same size the page is read at, and a footer that counts in the chrome's voice rather
 * than the library's.
 */
.rekall-md .md-editor-toolbar-wrapper {
  padding: 6px 8px;
  border-bottom: 1px solid var(--color-border);
}

.rekall-md .md-editor-toolbar-item {
  color: var(--color-text-subtle);
  border-radius: 6px;
  transition:
    color 120ms ease,
    background-color 120ms ease;
}

.rekall-md .md-editor-toolbar-item:hover {
  color: var(--color-text);
  background: var(--color-surface-hover);
}

.rekall-md .md-editor-toolbar-active {
  color: var(--color-accent);
  background: var(--color-accent-soft);
}

.rekall-md .md-editor-toolbar-divider {
  background: var(--color-border);
}

.rekall-md .md-editor .cm-scroller {
  font-family: var(--font-mono);
  font-size: 13px;
  line-height: 1.75;
}

.rekall-md .md-editor .cm-content {
  padding-block: 18px;
}

.rekall-md .md-editor .cm-line {
  padding-inline: 18px;
}

.rekall-md .md-editor-footer {
  height: 28px;
  font-family: var(--font-mono);
  font-size: 10.5px;
  color: var(--color-text-subtle);
  border-color: var(--color-border);
  background: var(--color-surface);
}

.rekall-md .md-editor-preview-wrapper .md-editor-preview {
  padding: 18px 26px 28px 30px;
}

.rekall-md--readonly {
  display: flex;
  flex-direction: column;
}

.rekall-md--readonly .md-editor-preview {
  width: 100%;
  max-width: var(--rk-read-width, 680px);
  margin-inline: auto;
  padding: 32px 28px 56px !important;
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

/* Compact: the same typesetting at card scale, flush with the title above it. The section ticks
   hang into the card's own padding, so the page is indented by exactly their reach. */
.rekall-md--compact .md-editor-preview {
  max-width: none;
  margin-inline: 0 !important;
  padding: 2px 4px 4px 22px !important;
  font-size: 13px !important;
  line-height: 1.7 !important;
}

/* Held over the page's top-right corner rather than given a row of its own, so opening a read
   view does not start with a strip of nothing above the first line. */
.rekall-read-controls {
  align-self: flex-end;
  position: sticky;
  top: 12px;
  z-index: 10;
  height: 0;
  margin-right: 12px;
  overflow: visible;
}
</style>
