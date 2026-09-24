<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, useId, watch } from 'vue'
import { fetchDirectory } from '@/api/filesystem.api'
import { ApiError } from '@/api/client'
import { filterEntries, splitByRanges, splitPathQuery, type PathRow } from '@/common/path/path-query'
import { pathPickerShowsHidden, setPathPickerShowsHidden } from '@/common/config/path-picker'
import type { DirectoryEntry, DirectoryListing } from '@/model/filesystem'

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
export interface CaretBox {
  readonly left: number
  readonly top: number
  readonly bottom: number
}

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
const crumbsEl = ref<HTMLElement | null>(null)
const previewEl = ref<HTMLElement | null>(null)

const folder = ref<string | null>(props.start ?? props.root)
const query = ref('')
const parsed = computed(() => splitPathQuery(query.value))
const listing = ref<DirectoryListing | null>(null)
const failure = ref<string | null>(null)
const notice = ref<string | null>(null)
const loading = ref(false)
/** Loading for long enough to be worth showing: a cached or local listing never flashes a bar. */
const slow = ref(false)
const showHidden = ref(pathPickerShowsHidden())
const highlighted = ref(0)
/** The project folder as the server spells it, once known, to mark it in the breadcrumb. */
const projectPath = ref<string | null>(null)
/** After going up, the folder just left is highlighted, the way Finder keeps your place. */
let focusAfterLoad: string | null = null

const cache = new Map<string, DirectoryListing>()
let ticket = 0
let slowTimer: ReturnType<typeof setTimeout> | undefined
let typeTimer: ReturnType<typeof setTimeout> | undefined

const SLOW_AFTER_MS = 140
const TYPE_SETTLE_MS = 90

async function load(base: string | null, typed: string): Promise<void> {
  const mine = ++ticket
  const key = `${base ?? ''}\u0000${typed}`
  const cached = cache.get(key)
  if (cached) {
    apply(cached)
    return
  }
  loading.value = true
  clearTimeout(slowTimer)
  slowTimer = setTimeout(() => {
    if (mine === ticket && loading.value) slow.value = true
  }, SLOW_AFTER_MS)
  try {
    const next = await fetchDirectory(base, typed)
    cache.set(key, next)
    if (mine === ticket) apply(next)
  } catch (error) {
    if (mine !== ticket) return
    const message = error instanceof ApiError ? error.message : 'Rekall could not list that folder.'
    if (!typed && listing.value === null && base !== null) {
      fallBackFrom(base)
      return
    }
    failure.value = message
    settle()
  }
}

/** The folder it opened on is gone (moved, unmounted): try the project, then home, and say so. */
function fallBackFrom(base: string): void {
  const next = base !== props.root && props.root ? props.root : null
  notice.value = `${base} is not there any more, so this opened ${next === null ? 'in your home folder' : 'in the project folder'}.`
  folder.value = next
}

function apply(next: DirectoryListing): void {
  listing.value = next
  failure.value = null
  settle()
  emit('visit', next.path)
  if (props.root && !projectPath.value && folder.value === props.root && !parsed.value.folder) {
    projectPath.value = next.path
  }
}

function settle(): void {
  loading.value = false
  slow.value = false
  clearTimeout(slowTimer)
}

watch(
  () => [folder.value, parsed.value.folder] as const,
  ([base, typed], previous) => {
    clearTimeout(typeTimer)
    const onlyTyping = previous !== undefined && previous[0] === base
    if (onlyTyping) typeTimer = setTimeout(() => void load(base, typed), TYPE_SETTLE_MS)
    else void load(base, typed)
  },
  { immediate: true }
)

const rows = computed<PathRow[]>(() =>
  listing.value && !failure.value ? filterEntries(listing.value.entries, parsed.value.needle, showHidden.value) : []
)

const hiddenCount = computed(() => listing.value?.entries.filter((entry) => entry.hidden).length ?? 0)

watch(rows, (next) => {
  const target = focusAfterLoad
  focusAfterLoad = null
  const found = target ? next.findIndex((row) => row.entry.path === target) : -1
  highlighted.value = found >= 0 ? found : 0
  void nextTick(scrollHighlightedIntoView)
})

const current = computed<DirectoryEntry | null>(() => rows.value[highlighted.value]?.entry ?? null)

interface Crumb {
  readonly label: string
  readonly path: string
  readonly project: boolean
  readonly home: boolean
}

/** From the filesystem root, or from `~` when the folder is under home: the part above is noise. */
const crumbs = computed<Crumb[]>(() => {
  const view = listing.value
  if (!view) return []
  const homeAt = view.segments.findIndex((segment) => segment.path === view.home)
  const from = homeAt >= 0 ? homeAt : 0
  return view.segments.slice(from).map((segment, index) => ({
    label: index === 0 && homeAt >= 0 ? '~' : segment.name,
    path: segment.path,
    project: segment.path === projectPath.value,
    home: index === 0 && homeAt >= 0
  }))
})

const atProject = computed(() => projectPath.value !== null && listing.value?.path === projectPath.value)
const atHome = computed(() => listing.value !== null && listing.value.path === listing.value.home)

const preview = computed<string>(() => {
  const entry = current.value
  if (entry) return props.format(entry.path, entry.directory)
  return listing.value && !failure.value ? props.format(listing.value.path, true) : ''
})

// --- navigation ---------------------------------------------------------------------------------

function goTo(path: string | null, highlight: string | null = null): void {
  focusAfterLoad = highlight
  query.value = ''
  folder.value = path
  filterField.value?.focus()
}

function open(entry: DirectoryEntry): void {
  if (entry.directory) goTo(entry.path)
}

function up(): void {
  const view = listing.value
  if (view?.parent) goTo(view.parent, view.path)
}

function jumpToCrumb(index: number): void {
  const crumb = crumbs.value[index]
  if (!crumb) return
  goTo(crumb.path, crumbs.value[index + 1]?.path ?? null)
}

function goProject(): void {
  if (props.root) goTo(projectPath.value ?? props.root)
}

function goHome(): void {
  goTo(listing.value?.home ?? null)
}

function toggleHidden(): void {
  showHidden.value = !showHidden.value
  setPathPickerShowsHidden(showHidden.value)
}

function insert(entry: DirectoryEntry): void {
  emit('pick', entry.path, entry.directory)
}

function insertFolderInView(): void {
  if (listing.value && !failure.value) emit('pick', listing.value.path, true)
}

function activate(entry: DirectoryEntry): void {
  if (entry.directory) open(entry)
  else insert(entry)
}

/** Tab on a highlighted name completes it into the filter's folder part, the way a shell does. */
function complete(entry: DirectoryEntry): void {
  if (!entry.directory) {
    query.value = `${parsed.value.folder}${entry.name}`
    return
  }
  open(entry)
}

// --- keyboard -----------------------------------------------------------------------------------

function move(delta: number): void {
  if (!rows.value.length) return
  highlighted.value = (highlighted.value + delta + rows.value.length) % rows.value.length
  scrollHighlightedIntoView()
}

function caretAtEnd(): boolean {
  const field = filterField.value
  return !field || (field.selectionStart === field.value.length && field.selectionEnd === field.value.length)
}

function onKeydown(event: KeyboardEvent): void {
  const mod = event.metaKey || event.ctrlKey
  const inFilter = event.target === filterField.value

  const handled = ((): boolean => {
    if (event.key === 'Escape') {
      // The first Escape clears a typed filter, the second closes: a stray Escape never loses a path.
      if (query.value) query.value = ''
      else emit('close')
      return true
    }
    // A focused button (a crumb, a toggle) keeps its own Enter and Space.
    if (!inFilter && (event.key === 'Enter' || event.key === ' ') && event.target instanceof HTMLButtonElement) {
      return false
    }
    if (mod && event.shiftKey && event.code === 'Period') {
      toggleHidden()
      return true
    }
    if (mod && event.shiftKey && event.code === 'KeyP') {
      emit('close')
      return true
    }
    if (mod && event.shiftKey && event.code === 'KeyH') {
      goHome()
      return true
    }
    if (mod && event.key === 'ArrowUp') {
      up()
      return true
    }
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      move(event.key === 'ArrowDown' ? 1 : -1)
      return true
    }
    if (event.key === 'PageDown' || event.key === 'PageUp') {
      move((event.key === 'PageDown' ? 1 : -1) * 8)
      return true
    }
    if (event.key === 'Enter') {
      if (mod) insertFolderInView()
      else if (current.value) insert(current.value)
      else insertFolderInView()
      return true
    }
    if (!inFilter) return false
    if (event.key === 'Tab' && !event.shiftKey && current.value) {
      complete(current.value)
      return true
    }
    if (event.key === 'ArrowRight' && caretAtEnd() && current.value?.directory) {
      open(current.value)
      return true
    }
    if ((event.key === 'ArrowLeft' || event.key === 'Backspace') && query.value === '') {
      up()
      return true
    }
    return false
  })()

  if (handled) event.preventDefault()
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

// --- placement ----------------------------------------------------------------------------------

const WIDTH = 480
const HEIGHT = 404
const MIN_BELOW = 260
const GUTTER = 8
const GAP = 6

const placement = ref<{ left: number; width: number; height: number; top?: number; bottom?: number }>({
  left: 0,
  width: WIDTH,
  height: HEIGHT
})

/**
 * Under the caret when there is room, over it when there is not. The height is fixed rather than
 * fitted to the rows, so the panel does not jump as a filter narrows the list under the cursor.
 */
function place(): void {
  const caret = props.locate()
  if (!caret) {
    emit('close')
    return
  }
  const width = Math.min(WIDTH, window.innerWidth - 2 * GUTTER)
  const left = Math.max(GUTTER, Math.min(caret.left - 22, window.innerWidth - width - GUTTER))
  const below = window.innerHeight - caret.bottom - GAP - GUTTER
  const above = caret.top - GAP - GUTTER
  if (below >= MIN_BELOW || below >= above) {
    placement.value = { left, width, height: Math.min(HEIGHT, below), top: caret.bottom + GAP }
  } else {
    placement.value = { left, width, height: Math.min(HEIGHT, above), bottom: window.innerHeight - caret.top + GAP }
  }
}

const panelStyle = computed(() => ({
  left: `${placement.value.left}px`,
  width: `${placement.value.width}px`,
  height: `${placement.value.height}px`,
  top: placement.value.top === undefined ? undefined : `${placement.value.top}px`,
  bottom: placement.value.bottom === undefined ? undefined : `${placement.value.bottom}px`,
  transformOrigin: placement.value.top === undefined ? 'bottom left' : 'top left'
}))

// --- scrolling the long things to their ends ----------------------------------------------------

function scrollHighlightedIntoView(): void {
  listEl.value?.querySelector(`#${CSS.escape(rowId(highlighted.value))}`)?.scrollIntoView({ block: 'nearest' })
}

/** A path is read from its end: the breadcrumb and the preview keep their last characters in view. */
function revealEnd(element: HTMLElement | null): void {
  if (!element) return
  element.scrollLeft = element.scrollWidth
  element.dataset.overflowing = String(element.scrollWidth > element.clientWidth + 1)
}

watch(crumbs, () => void nextTick(() => revealEnd(crumbsEl.value)))
watch(preview, () => void nextTick(() => revealEnd(previewEl.value)))

onMounted(async () => {
  place()
  window.addEventListener('keydown', onKeydown, true)
  window.addEventListener('pointerdown', onPointerDown, true)
  window.addEventListener('resize', place)
  window.addEventListener('scroll', place, true)
  // Opened somewhere other than the project: learn how the server spells the project folder, so
  // its crumb can be marked, without waiting for someone to go there.
  if (props.root && folder.value !== props.root) {
    fetchDirectory(props.root)
      .then((rootListing) => {
        cache.set(`${props.root}\u0000`, rootListing)
        projectPath.value = rootListing.path
      })
      .catch(() => undefined)
  }
  await nextTick()
  filterField.value?.focus()
})

onUnmounted(() => {
  clearTimeout(slowTimer)
  clearTimeout(typeTimer)
  window.removeEventListener('keydown', onKeydown, true)
  window.removeEventListener('pointerdown', onPointerDown, true)
  window.removeEventListener('resize', place)
  window.removeEventListener('scroll', place, true)
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
    <!-- Where you are: the breadcrumb, and the two places you most often want to be instead. -->
    <div class="flex h-9 shrink-0 items-center gap-1 border-b border-border pl-1.5 pr-1.5">
      <nav
        ref="crumbsEl"
        class="path-crumbs min-w-0 flex-1 overflow-x-auto"
        aria-label="Folder in view"
        data-testid="path-picker-crumbs"
      >
        <ol class="flex w-max items-center font-mono text-[11px] leading-none">
          <li v-for="(crumb, index) in crumbs" :key="crumb.path" class="flex items-center">
            <span
              v-if="index > 0 && !/[\\/]$/.test(crumbs[index - 1]!.label)"
              class="px-px text-text-subtle/50"
              aria-hidden="true"
              >/</span
            >
            <button
              type="button"
              class="focus-ring flex h-6 items-center gap-1 rounded-[6px] px-1.5 transition-colors"
              :class="
                index === crumbs.length - 1
                  ? 'text-text'
                  : 'text-text-subtle hover:bg-surface-hover hover:text-text'
              "
              :aria-current="index === crumbs.length - 1 ? 'location' : undefined"
              :title="crumb.path"
              :data-project="crumb.project || undefined"
              data-testid="path-picker-crumb"
              @click="jumpToCrumb(index)"
            >
              <!-- The project's own folder carries the accent dot: the place this picker is about. -->
              <span v-if="crumb.project" class="crumb-project-dot" aria-hidden="true" />
              {{ crumb.label }}
            </button>
          </li>
        </ol>
      </nav>

      <button
        v-if="root"
        type="button"
        class="focus-ring grid size-7 shrink-0 place-items-center rounded-[7px] transition-colors"
        :class="atProject ? 'bg-accent-soft text-accent' : 'text-text-subtle hover:bg-surface-hover hover:text-text'"
        :aria-pressed="atProject"
        title="Project folder"
        aria-label="Go to the project folder"
        data-testid="path-picker-project"
        @click="goProject"
      >
        <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path
            d="M1.75 4.25c0-.55.45-1 1-1h3.1l1.4 1.5h6c.55 0 1 .45 1 1v6.5c0 .55-.45 1-1 1H2.75c-.55 0-1-.45-1-1v-8Z"
            stroke="currentColor"
            stroke-width="1.25"
            stroke-linejoin="round"
          />
          <circle cx="8" cy="8.9" r="1.35" fill="currentColor" />
        </svg>
      </button>
      <button
        type="button"
        class="focus-ring grid size-7 shrink-0 place-items-center rounded-[7px] transition-colors"
        :class="atHome ? 'bg-accent-soft text-accent' : 'text-text-subtle hover:bg-surface-hover hover:text-text'"
        :aria-pressed="atHome"
        title="Home folder  ⌘⇧H"
        aria-label="Go to your home folder"
        data-testid="path-picker-home"
        @click="goHome"
      >
        <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path
            d="M2.5 7.1 8 2.6l5.5 4.5M4 6v6.9c0 .3.2.5.5.5h2.4V10h2.2v3.4h2.4c.3 0 .5-.2.5-.5V6"
            stroke="currentColor"
            stroke-width="1.25"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
      <button
        type="button"
        class="focus-ring grid h-7 shrink-0 place-items-center rounded-[7px] px-1.5 font-mono text-[11px] transition-colors"
        :class="showHidden ? 'bg-accent-soft text-accent' : 'text-text-subtle hover:bg-surface-hover hover:text-text'"
        :aria-pressed="showHidden"
        :title="showHidden ? 'Hide hidden files  ⌘⇧.' : 'Show hidden files  ⌘⇧.'"
        aria-label="Show hidden files"
        data-testid="path-picker-hidden"
        @click="toggleHidden"
      >
        .*
      </button>
    </div>

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

      <div v-else-if="failure" class="path-message" data-testid="path-picker-failure">
        <p class="text-text-muted">{{ failure }}</p>
        <p class="text-text-subtle">Keep typing, or <kbd>esc</kbd> to clear the filter.</p>
      </div>

      <div v-else-if="listing && !listing.readable" class="path-message" data-testid="path-picker-unreadable">
        <p class="text-text-muted">Rekall is not allowed to read this folder.</p>
        <p class="text-text-subtle">
          macOS keeps some folders private to the apps that own them. <kbd>⌘↵</kbd> still inserts its path.
        </p>
      </div>

      <div v-else-if="!rows.length" class="path-message" data-testid="path-picker-empty">
        <template v-if="parsed.needle">
          <p class="text-text-muted">
            Nothing here matches <span class="font-mono text-text">{{ parsed.needle }}</span>.
          </p>
          <p v-if="!showHidden && hiddenCount" class="text-text-subtle">
            Hidden files are left out.
            <button type="button" class="focus-ring text-accent hover:underline" @click="toggleHidden">
              Show them
            </button>
          </p>
        </template>
        <template v-else-if="hiddenCount">
          <p class="text-text-muted">Only hidden files in here.</p>
          <p class="text-text-subtle">
            <button type="button" class="focus-ring text-accent hover:underline" @click="toggleHidden">
              Show them
            </button>
            <kbd class="ml-1">⌘⇧.</kbd>, or <kbd>↵</kbd> to insert the folder itself.
          </p>
        </template>
        <template v-else>
          <p class="text-text-muted">This folder is empty.</p>
          <p class="text-text-subtle"><kbd>↵</kbd> inserts the folder itself.</p>
        </template>
      </div>

      <ul
        v-else
        :id="listId"
        role="listbox"
        aria-label="Folders and files"
        :class="{ 'opacity-60 transition-opacity': slow }"
      >
        <li
          v-for="(row, index) in rows"
          :id="rowId(index)"
          :key="row.entry.path"
          role="option"
          :aria-selected="index === highlighted"
          class="path-row group/row flex h-[30px] cursor-pointer items-center gap-2.5 rounded-[7px] pl-3.5 pr-1.5"
          :class="[index === highlighted ? 'selected-row' : 'hover:bg-surface-hover', row.entry.hidden && 'opacity-60']"
          :data-directory="row.entry.directory"
          data-testid="path-picker-row"
          @mousemove="highlighted = index"
          @click="activate(row.entry)"
        >
          <svg
            v-if="row.entry.directory"
            class="size-3.5 shrink-0"
            :class="index === highlighted ? 'text-accent' : 'text-text-muted'"
            viewBox="0 0 16 16"
            aria-hidden="true"
          >
            <path
              d="M1.75 4.25c0-.55.45-1 1-1h3.1l1.4 1.5h6c.55 0 1 .45 1 1v6.5c0 .55-.45 1-1 1H2.75c-.55 0-1-.45-1-1v-8Z"
              fill="currentColor"
              fill-opacity="0.16"
              stroke="currentColor"
              stroke-width="1.25"
              stroke-linejoin="round"
            />
          </svg>
          <svg v-else class="size-3.5 shrink-0 text-text-subtle" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path
              d="M4 1.75h5.1L12.25 5v8.25c0 .55-.45 1-1 1H4c-.55 0-1-.45-1-1V2.75c0-.55.45-1 1-1Z"
              stroke="currentColor"
              stroke-width="1.25"
              stroke-linejoin="round"
            />
            <path d="M9 1.9V5.1h3.1" stroke="currentColor" stroke-width="1.25" stroke-linejoin="round" />
          </svg>

          <span
            class="min-w-0 flex-1 truncate font-mono text-[12px]"
            :class="index === highlighted ? 'text-text' : 'text-text-muted'"
            :title="row.entry.name"
          >
            <template v-for="(part, partIndex) in splitByRanges(row.entry.name, row.ranges)" :key="partIndex">
              <mark v-if="part.matched" class="path-match">{{ part.text }}</mark>
              <template v-else>{{ part.text }}</template>
            </template>
          </span>

          <!-- On the highlighted folder, both of its moves, since a click only does one. -->
          <span
            v-if="row.entry.directory && index === highlighted"
            class="flex shrink-0 items-center gap-1 text-[10.5px] text-text-subtle"
          >
            <button
              type="button"
              tabindex="-1"
              class="rounded-[5px] px-1.5 py-0.5 transition-colors hover:bg-accent-soft hover:text-accent"
              data-testid="path-picker-insert-folder"
              @click.stop="insert(row.entry)"
            >
              <kbd>↵</kbd> insert
            </button>
          </span>
          <svg
            v-if="row.entry.directory"
            class="size-3 shrink-0 transition-colors"
            :class="index === highlighted ? 'text-text-muted' : 'text-text-subtle/50'"
            viewBox="0 0 12 12"
            fill="none"
            aria-hidden="true"
          >
            <path d="m4.5 2.75 3.25 3.25-3.25 3.25" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </li>
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
.path-picker kbd {
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

/* The breadcrumb scrolls sideways under a fade rather than wrapping: the deep end is what matters. */
.path-crumbs {
  scrollbar-width: none;
}

.path-crumbs::-webkit-scrollbar {
  display: none;
}

.path-crumbs[data-overflowing='true'],
.path-preview[data-overflowing='true'] {
  mask-image: linear-gradient(90deg, transparent 0, #000 22px);
}

.crumb-project-dot {
  width: 5px;
  height: 5px;
  border-radius: 999px;
  background: var(--color-accent);
  box-shadow: 0 0 6px color-mix(in srgb, var(--color-accent) 70%, transparent);
}

/* The matched letters: lit, not boxed, so a scattered match still reads as one word. */
.path-match {
  background: none;
  color: var(--color-accent-strong);
  font-weight: 600;
}

.path-message {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 14px 14px 10px;
  font-size: 12px;
  line-height: 1.5;
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
