<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppLogo from '@/components/ui/AppLogo.vue'
import AppNavSwitcher from '@/components/console/AppNavSwitcher.vue'
import ClaudeUsageMeter from '@/components/console/ClaudeUsageMeter.vue'
import QueueBeacon from '@/components/queue/QueueBeacon.vue'
import SearchHitList from '@/components/console/SearchHitList.vue'
import ReviewQueueButton from '@/components/console/ReviewQueueButton.vue'
import WindowControls from '@/components/console/WindowControls.vue'
import { useConsoleStore } from '@/stores/console.store'
import { searchText } from '@/api/search.api'
import { isTextQuery } from '@/model/search'
import type { SearchHit } from '@/model/search'
import { isDesktopApp } from '@/common/native/desktop'

const isDesktop = isDesktopApp()

const emit = defineEmits<{ newNote: []; openSettings: []; openTags: [] }>()

const store = useConsoleStore()
const {
  filter,
  scopePath,
  scopeName,
  saveState,
  navMode,
  visibleTasks,
  visibleDocuments,
  selectedTaskId
} = storeToRefs(store)

const input = ref<HTMLInputElement | null>(null)
const isFocused = ref(false)

const placeholder = computed(() =>
  scopePath.value.length === 0
    ? // Short enough to show whole in the narrowest bar: a hint cut off mid-word reads as a bug.
      'project:vega task:setup'
    : `Find in ${scopeName.value}`
)

const resultCount = computed(() =>
  navMode.value === 'tasks' ? visibleTasks.value.length : visibleDocuments.value.length
)

/** Browsing tasks, a new note needs a task in view. Browsing notes, the composer asks for one. */
const needsTaskForNote = computed(() => navMode.value === 'tasks' && !selectedTaskId.value)

const SAVE_LABEL = { saved: 'Saved', unsaved: 'Unsaved', saving: 'Saving' } as const

const SAVE_DOT = {
  saved: 'bg-safe',
  unsaved: 'bg-warn',
  saving: 'bg-accent animate-pulse'
} as const

const justSaved = ref(false)
watch(saveState, (value, previous) => {
  if (value === 'saved' && previous !== 'saved') {
    justSaved.value = false
    requestAnimationFrame(() => {
      justSaved.value = true
      setTimeout(() => (justSaved.value = false), 340)
    })
  }
})

const query = computed<string>({
  get: () => filter.value,
  set: (value) => {
    filter.value = value.replace(/^\s*\/rk\s+/i, '')
  }
})

// The filter matches titles and labels as you type. For a phrase, the text behind them is searched
// on the server too: descriptions, steps, wrapups and notes, listed under the bar.
const SEARCH_DEBOUNCE_MS = 200
const hits = ref<SearchHit[]>([])
const activeHit = ref(-1)
const searching = ref(false)
const searchFailed = ref(false)
let searchTimer: ReturnType<typeof setTimeout> | null = null
let searchSequence = 0

const showHits = computed(() => isFocused.value && isTextQuery(filter.value))

watch(filter, (value) => {
  if (searchTimer) clearTimeout(searchTimer)
  activeHit.value = -1
  const sequence = ++searchSequence
  if (!isTextQuery(value)) {
    hits.value = []
    searching.value = false
    return
  }
  searching.value = true
  searchTimer = setTimeout(async () => {
    try {
      const found = await searchText(value.trim())
      if (sequence !== searchSequence) return
      hits.value = found
      searchFailed.value = false
    } catch {
      if (sequence !== searchSequence) return
      hits.value = []
      searchFailed.value = true
    } finally {
      if (sequence === searchSequence) searching.value = false
    }
  }, SEARCH_DEBOUNCE_MS)
})

function moveHit(delta: number): void {
  if (!showHits.value || hits.value.length === 0) return
  const count = hits.value.length
  activeHit.value = activeHit.value < 0 && delta < 0 ? count - 1 : (activeHit.value + delta + count) % count
}

function openHit(hit: SearchHit): void {
  store.openSearchHit(hit)
  filter.value = ''
  input.value?.blur()
}

function onEnter(): void {
  const hit = hits.value[activeHit.value]
  if (showHits.value && hit) {
    openHit(hit)
    return
  }
  openFirst()
}

function openFirst(): void {
  if (navMode.value === 'notes') {
    const first = visibleDocuments.value[0]
    if (first) store.selectDocument(first.id)
    return
  }
  const first = visibleTasks.value[0]
  if (first) store.selectTask(first.id)
}

function clear(): void {
  filter.value = ''
  input.value?.blur()
}

defineExpose({ focus: () => { input.value?.focus(); input.value?.select() } })
</script>

<template>
  <header
    :data-tauri-drag-region="isDesktop ? true : undefined"
    class="glass sticky top-0 z-(--z-sticky) flex h-(--spacing-header) shrink-0 items-center gap-3.5 border-b border-border px-4"
  >
    <WindowControls v-if="isDesktop" />

    <div class="flex w-(--spacing-nav) shrink-0 items-center gap-2.5 pr-2.5">
      <AppLogo :size="32" class="halo rounded-[7px]" />
      <span class="min-w-0">
        <span class="block text-[14.5px] font-semibold leading-tight tracking-[-0.015em] text-text">
          Rekall
        </span>
        <span class="block font-mono text-[10px] leading-tight text-text-subtle">
          context, anchored
        </span>
      </span>
    </div>

    <AppNavSwitcher />

    <div class="relative max-w-[640px] flex-1">
      <span
        class="pointer-events-none absolute left-3.5 top-1/2 -translate-y-1/2 font-mono text-[13px] text-anchor"
        aria-hidden="true"
      >
        /rk
      </span>
      <input
        ref="input"
        v-model="query"
        data-testid="anchor-input"
        type="text"
        autocomplete="off"
        spellcheck="false"
        class="field h-10 w-full rounded-[var(--radius-control)] pl-[48px] pr-[92px] font-mono text-[13px] text-text"
        :placeholder="placeholder"
        aria-label="Find a task or a note"
        @focus="isFocused = true"
        @blur="isFocused = false"
        @keydown.enter="onEnter"
        @keydown.down.prevent="moveHit(1)"
        @keydown.up.prevent="moveHit(-1)"
        @keydown.esc="clear"
      />
      <SearchHitList
        v-if="showHits"
        :hits="hits"
        :active-index="activeHit"
        :searching="searching"
        :failed="searchFailed"
        :term="filter.trim()"
        @pick="openHit"
        @hover="activeHit = $event"
      />
      <span
        class="pointer-events-none absolute right-2.5 top-1/2 flex -translate-y-1/2 items-center gap-1"
        aria-hidden="true"
      >
        <template v-if="filter.trim()">
          <span class="font-mono text-[11px] text-text-subtle">{{ resultCount }} found</span>
        </template>
        <template v-else>
          <kbd
            class="rounded border border-border bg-surface-raised px-1.5 py-0.5 font-mono text-[10px] text-text-subtle"
          >
            &#8984;
          </kbd>
          <kbd
            class="rounded border border-border bg-surface-raised px-1.5 py-0.5 font-mono text-[10px] text-text-subtle"
          >
            K
          </kbd>
        </template>
      </span>
    </div>

    <!-- Side by side on purpose: the queue's ceiling is read off the meter next to it. -->
    <ClaudeUsageMeter class="shrink-0" />
    <QueueBeacon />

    <div class="ml-auto flex items-center gap-3">
      <ReviewQueueButton />
      <!-- Under 1440px the bar has no room for the word: the dot's colour carries it, and the word
           stays for screen readers and as the tooltip. -->
      <p
        class="flex min-w-[92px] items-center gap-2 text-[12px] max-[1440px]:min-w-0"
        :class="saveState === 'unsaved' ? 'text-warn' : 'text-text-subtle'"
        :title="SAVE_LABEL[saveState]"
        role="status"
        aria-live="polite"
        data-testid="save-state"
      >
        <span
          class="size-1.5 rounded-full transition-colors"
          :class="[SAVE_DOT[saveState], justSaved && 'settle']"
          aria-hidden="true"
        />
        <span class="max-[1440px]:sr-only">{{ SAVE_LABEL[saveState] }}</span>
      </p>
      <a
        href="/api/export"
        download
        data-testid="export-link"
        class="focus-ring inline-flex h-8 items-center gap-1.5 rounded-[var(--radius-control)] border border-border-strong bg-surface-raised px-3 text-[12.5px] text-text transition-colors hover:border-accent hover:bg-surface-hover"
        title="Download every company, project, task and note as a folder tree"
      >
        <svg class="size-3.5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path
            d="M12 3v12m0 0l-4-4m4 4l4-4M4 19h16"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        Export
      </a>
      <button
        type="button"
        data-testid="tags-trigger"
        aria-label="Tags"
        title="Tags"
        class="focus-ring grid size-8 shrink-0 place-items-center rounded-[var(--radius-control)] border border-border-strong bg-surface-raised text-text-subtle transition-colors hover:border-accent hover:bg-surface-hover hover:text-text"
        @click="emit('openTags')"
      >
        <!-- Drawn in the chrome's grey like the cog beside it: a lit crimson glyph here was the
             only coloured icon in the bar, and colour up here means state, not a menu. -->
        <svg class="size-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path
            d="M3.5 12.1V4.6a1.1 1.1 0 0 1 1.1-1.1h7.5a1.1 1.1 0 0 1 .78.32l7.8 7.8a1.1 1.1 0 0 1 0 1.56l-7.5 7.5a1.1 1.1 0 0 1-1.56 0l-7.8-7.8a1.1 1.1 0 0 1-.32-.78Z"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linejoin="round"
          />
          <circle cx="8.2" cy="8.2" r="1.5" fill="currentColor" />
        </svg>
      </button>
      <button
        type="button"
        data-testid="settings-trigger"
        aria-label="Settings"
        title="Settings"
        class="focus-ring grid size-8 shrink-0 place-items-center rounded-[var(--radius-control)] border border-border-strong bg-surface-raised text-text-subtle transition-colors hover:border-accent hover:bg-surface-hover hover:text-text"
        @click="emit('openSettings')"
      >
        <svg class="size-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path
            d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
      <span class="group relative inline-block">
        <AppButton
          variant="primary"
          size="sm"
          :disabled="needsTaskForNote"
          data-testid="new-note"
          @click="emit('newNote')"
        >
          New note
        </AppButton>
        <span
          v-if="needsTaskForNote"
          class="pointer-events-none absolute right-0 top-full z-(--z-sticky) mt-2 whitespace-nowrap rounded-[var(--radius-control)] border border-border-strong bg-surface-raised px-2.5 py-1.5 text-xs text-text opacity-0 shadow-lift transition-opacity duration-100 group-hover:opacity-100"
        >
          Select a task to create a new note on it
        </span>
      </span>
    </div>
  </header>
</template>
