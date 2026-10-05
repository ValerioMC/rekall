<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, useId, watch } from 'vue'
import PathPickerRow from '@/components/ui/path-picker/PathPickerRow.vue'
import PathPickerStatus from '@/components/ui/path-picker/PathPickerStatus.vue'
import PathPickerTopBar from '@/components/ui/path-picker/PathPickerTopBar.vue'
import { useCaretPlacement, type CaretBox } from '@/composables/useCaretPlacement'
import { useDirectoryBrowser } from '@/composables/useDirectoryBrowser'
import type { DirectoryEntry } from '@/model/filesystem'

export type { CaretBox }

/**
 * The path picker: a small file browser hung under the writing caret, so a path goes into the
 * markdown without a trip to Finder and a paste.
 *
 * It opens in the project's folder (or wherever it was last left in this editor) and can go
 * anywhere on the machine. Two ways through it, both always live:
 *
 * - Browse: arrows move, → or Tab opens a folder, ← or ⌫ on an empty filter goes up, the
 *   breadcrumb jumps to any folder above, and the two buttons beside it go to the project or home.
 * - Type: the filter reads like a shell prompt. A name filters the folder in view; anything up
 *   to a slash names a folder to list first (`src/comp`, `~/Downl`, `/etc/`), resolved by the
 *   server against the folder in view, so nothing here joins or normalises a path.
 *
 * ↵ inserts the highlighted entry, ⌘↵ the folder in view. The footer shows exactly the text that
 * will land, formatted by the editor, so what is inserted is never a surprise.
 */
const props = defineProps<{
  /** The project's folder: where the picker opens and what the project button returns to. */
  root: string | null
  /** Where to open instead of the root: the folder the picker was left in last time. */
  start: string | null
  /** Where the path will land, in viewport coordinates; null when it cannot be measured. */
  locate: () => CaretBox | null
  /** The exact text a pick inserts, so the footer can preview it. */
  format: (path: string, directory: boolean) => string
}>()

const emit = defineEmits<{
  pick: [path: string, directory: boolean]
  /** The folder now in view, so the editor can reopen the picker there. */
  visit: [folder: string]
  close: []
}>()

const uid = useId()
const listId = `${uid}-entries`
const rowId = (index: number): string => `${uid}-entry-${index}`

const panel = ref<HTMLElement | null>(null)
const filterField = ref<HTMLInputElement | null>(null)
const listEl = ref<HTMLElement | null>(null)
const previewEl = ref<HTMLElement | null>(null)

const browser = useDirectoryBrowser({
  root: () => props.root,
  start: props.start,
  onVisit: (folder) => emit('visit', folder),
  onNavigate: () => filterField.value?.focus(),
  onHighlightMoved: () => scrollHighlightedIntoView()
})
const { query, parsed, listing, failure, notice, slow, showHidden, highlighted, rows, hiddenCount, current, crumbs, atProject, atHome } =
  browser

const panelStyle = useCaretPlacement(props.locate, () => emit('close'))

const preview = computed<string>(() => {
  const entry = current.value
  if (entry) return props.format(entry.path, entry.directory)
  return listing.value && !failure.value ? props.format(listing.value.path, true) : ''
})

function insert(entry: DirectoryEntry): void {
  emit('pick', entry.path, entry.directory)
}

function insertFolderInView(): void {
  if (listing.value && !failure.value) emit('pick', listing.value.path, true)
}

/** A click opens a folder and inserts a file. */
function activate(entry: DirectoryEntry): void {
  if (entry.directory) browser.open(entry)
  else insert(entry)
}

function move(delta: number): void {
  browser.move(delta)
  scrollHighlightedIntoView()
}

function caretAtEnd(): boolean {
  const field = filterField.value
  return !field || (field.selectionStart === field.value.length && field.selectionEnd === field.value.length)
}

/** The keys that work anywhere in the panel; `false` when the key is not the panel's. */
function handlePanelKey(event: KeyboardEvent, mod: boolean, inFilter: boolean): boolean {
  if (event.key === 'Escape') {
    // The first Escape clears a typed filter, the second closes: a stray Escape never loses a path.
    if (query.value) query.value = ''
    else emit('close')
    return true
  }
  // A focused button (a crumb, a toggle) keeps its own Enter and Space.
  if (!inFilter && (event.key === 'Enter' || event.key === ' ') && event.target instanceof HTMLButtonElement) return false
  if (mod && event.shiftKey && event.code === 'Period') browser.toggleHidden()
  else if (mod && event.shiftKey && event.code === 'KeyP') emit('close')
  else if (mod && event.shiftKey && event.code === 'KeyH') browser.goHome()
  else if (mod && event.key === 'ArrowUp') browser.up()
  else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') move(event.key === 'ArrowDown' ? 1 : -1)
  else if (event.key === 'PageDown' || event.key === 'PageUp') move((event.key === 'PageDown' ? 1 : -1) * 8)
  else if (event.key === 'Enter') {
    if (!mod && current.value) insert(current.value)
    else insertFolderInView()
  } else return inFilter && handleFilterKey(event)
  return true
}

/** The keys that only mean something while typing in the filter. */
function handleFilterKey(event: KeyboardEvent): boolean {
  if (event.key === 'Tab' && !event.shiftKey && current.value) browser.complete(current.value)
  else if (event.key === 'ArrowRight' && caretAtEnd() && current.value?.directory) browser.open(current.value)
  else if ((event.key === 'ArrowLeft' || event.key === 'Backspace') && query.value === '') browser.up()
  else return false
  return true
}

function onKeydown(event: KeyboardEvent): void {
  if (handlePanelKey(event, event.metaKey || event.ctrlKey, event.target === filterField.value)) event.preventDefault()
  // Nothing typed here reaches the console's shortcuts or the editor behind the panel.
  event.stopPropagation()
}

function onPointerDown(event: PointerEvent): void {
  const target = event.target as Element | null
  if (!target || panel.value?.contains(target)) return
  // The toolbar button toggles the picker itself; closing here as well would reopen it.
  if (target.closest?.('[data-path-picker-trigger]')) return
  emit('close')
}

function scrollHighlightedIntoView(): void {
  listEl.value?.querySelector(`#${CSS.escape(rowId(highlighted.value))}`)?.scrollIntoView({ block: 'nearest' })
}

/** A path is read from its end: the preview keeps its last characters in view. */
watch(preview, () =>
  void nextTick(() => {
    const element = previewEl.value
    if (!element) return
    element.scrollLeft = element.scrollWidth
    element.dataset.overflowing = String(element.scrollWidth > element.clientWidth + 1)
  })
)

onMounted(async () => {
  window.addEventListener('keydown', onKeydown, true)
  window.addEventListener('pointerdown', onPointerDown, true)
  browser.learnProjectPath()
  await nextTick()
  filterField.value?.focus()
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown, true)
  window.removeEventListener('pointerdown', onPointerDown, true)
})
</script>

<template>
  <div
    ref="panel"
    class="path-picker fixed z-(--z-overlay) flex flex-col overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-lift"
    :style="panelStyle"
    role="dialog"
    aria-label="Insert a path"
    data-testid="path-picker"
  >
    <PathPickerTopBar
      :crumbs="crumbs"
      :has-project="!!root"
      :at-project="atProject"
      :at-home="atHome"
      :show-hidden="showHidden"
      @crumb="browser.jumpToCrumb"
      @project="browser.goProject"
      @home="browser.goHome"
      @toggle-hidden="browser.toggleHidden"
    />

    <!-- The filter, which is also a prompt: a name filters, a slash goes somewhere. -->
    <div class="relative shrink-0 border-b border-border px-2 py-2">
      <label class="sr-only" :for="`${uid}-filter`">Filter this folder, or type a path</label>
      <input
        :id="`${uid}-filter`"
        ref="filterField"
        v-model="query"
        type="text"
        role="combobox"
        aria-autocomplete="list"
        aria-expanded="true"
        :aria-controls="listId"
        :aria-activedescendant="current ? rowId(highlighted) : undefined"
        autocomplete="off"
        autocapitalize="off"
        spellcheck="false"
        placeholder="Filter, or type a path: src/  ~/  /"
        class="field h-8 w-full rounded-[var(--radius-control)] px-2.5 font-mono text-[12px] text-text"
        :aria-invalid="failure ? 'true' : undefined"
        data-testid="path-picker-filter"
      />
      <span v-if="slow" class="path-progress" role="status" aria-label="Loading folder" />
    </div>

    <div ref="listEl" class="relative min-h-0 flex-1 overflow-y-auto px-1 py-1" data-testid="path-picker-list">
      <p
        v-if="notice"
        class="mx-2 mb-1 mt-0.5 rounded-[7px] bg-warn-soft px-2.5 py-1.5 text-[11.5px] leading-snug text-warn"
        data-testid="path-picker-notice"
      >
        {{ notice }}
      </p>

      <div v-if="!listing && !failure" role="status" aria-live="polite" aria-busy="true">
        <span class="sr-only">Loading</span>
        <div v-for="index in 7" :key="index" class="flex h-[30px] items-center gap-2.5 pl-4">
          <div class="skeleton size-3.5" />
          <div class="skeleton h-2.5" :style="{ width: `${90 + ((index * 37) % 110)}px` }" />
        </div>
      </div>

      <PathPickerStatus
        v-else-if="failure || !listing?.readable || !rows.length"
        :failure="failure"
        :unreadable="!!listing && !listing.readable"
        :needle="parsed.needle"
        :show-hidden="showHidden"
        :hidden-count="hiddenCount"
        @show-hidden="browser.toggleHidden"
      />

      <ul v-else :id="listId" role="listbox" aria-label="Folders and files" :class="{ 'opacity-60 transition-opacity': slow }">
        <PathPickerRow
          v-for="(row, index) in rows"
          :id="rowId(index)"
          :key="row.entry.path"
          :row="row"
          :highlighted="index === highlighted"
          @hover="highlighted = index"
          @activate="activate(row.entry)"
          @insert="insert(row.entry)"
        />
      </ul>
    </div>

    <!-- What lands in the markdown, exactly, and the keys that put it there. -->
    <div class="shrink-0 border-t border-border bg-canvas px-3 pb-2 pt-1.5">
      <div class="flex min-w-0 items-baseline gap-2">
        <span class="shrink-0 text-[10.5px] text-text-subtle">Inserts</span>
        <code
          ref="previewEl"
          class="path-preview min-w-0 flex-1 overflow-hidden whitespace-nowrap font-mono text-[11px] text-text-muted"
          data-testid="path-picker-preview"
          >{{ preview }}</code
        >
      </div>
      <div class="mt-1 flex flex-wrap items-center gap-x-2.5 gap-y-0.5 text-[10.5px] text-text-subtle">
        <span><kbd>↵</kbd> insert</span>
        <span><kbd>→</kbd> open</span>
        <span><kbd>⌫</kbd> up</span>
        <span><kbd>⌘↵</kbd> this folder</span>
        <span v-if="listing?.truncated" class="text-warn" data-testid="path-picker-truncated">
          first {{ listing.entries.length.toLocaleString() }} shown
        </span>
        <span class="ml-auto"><kbd>esc</kbd></span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.path-picker :deep(kbd) {
  display: inline-block;
  min-width: 1.35em;
  padding: 0 0.3em;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  font-size: 9.5px;
  line-height: 1.5;
  text-align: center;
  color: var(--color-text-muted);
}

.path-preview[data-overflowing='true'] {
  mask-image: linear-gradient(90deg, transparent 0, #000 22px);
}

/*
 * A folder that is slow to list (a network mount, a huge directory) draws a hairline sweeping the
 * bottom edge of the filter. It exists only while that load does, and a fast one never shows it.
 */
.path-progress {
  position: absolute;
  left: 0;
  right: 0;
  bottom: -1px;
  height: 1px;
  overflow: hidden;
}

.path-progress::after {
  content: '';
  position: absolute;
  inset: 0;
  width: 34%;
  background: linear-gradient(90deg, transparent, var(--color-accent), transparent);
  animation: path-sweep 900ms ease-in-out infinite;
}

@keyframes path-sweep {
  from {
    transform: translateX(-100%);
  }
  to {
    transform: translateX(300%);
  }
}
</style>
