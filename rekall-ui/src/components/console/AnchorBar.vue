<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import HeaderStatus from '@/components/console/HeaderStatus.vue'
import SearchHitList from '@/components/console/SearchHitList.vue'
import SettingsTrigger from '@/components/console/SettingsTrigger.vue'
import { useConsoleStore } from '@/stores/console.store'
import { searchText } from '@/api/search.api'
import { isTextQuery } from '@/model/search'
import type { SearchHit } from '@/model/search'

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
  selectedTaskId,
  selectedTask,
  selectedDocument,
  scopeAnchor
} = storeToRefs(store)

const input = ref<HTMLInputElement | null>(null)
const isFocused = ref(false)

const placeholder = computed(() =>
  scopePath.value.length === 0
    ? // Short enough to show whole in the narrowest bar: a hint cut off mid-word reads as a bug.
      'project:vega task:setup'
    : `Find in ${scopeName.value}`
)

/** Where the reader is, as the anchors a session would be handed: the note, the task, or the scope. */
const location = computed<string[]>(() => {
  const anchors =
    navMode.value === 'notes' && selectedDocument.value
      ? selectedDocument.value.anchor
      : (selectedTask.value?.anchor ?? scopeAnchor.value)
  return anchors.split(' ').filter((part) => part.length > 0)
})

/** At rest the bar is a location bar; the moment it is focused or holds a query it is a search. */
const showsLocation = computed(() => !isFocused.value && !filter.value && location.value.length > 0)

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
  <Teleport to="#app-header-extra">
    <div data-header-search class="relative min-w-0">
      <span
        class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 font-mono text-small text-anchor"
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
        class="field h-9 w-full rounded-card pl-11 pr-16 font-mono text-small text-text max-[1099px]:pr-3"
        :placeholder="showsLocation ? '' : placeholder"
        aria-label="Find a task or a note"
        :aria-describedby="showsLocation ? 'anchor-location' : undefined"
        @focus="isFocused = true"
        @blur="isFocused = false"
        @keydown.enter="onEnter"
        @keydown.down.prevent="moveHit(1)"
        @keydown.up.prevent="moveHit(-1)"
        @keydown.esc="clear"
      />
      <span
        v-if="showsLocation"
        id="anchor-location"
        class="pointer-events-none absolute inset-y-0 left-11 right-16 flex min-w-0 items-center gap-1.5 overflow-hidden max-[1099px]:right-3"
        data-testid="anchor-location"
      >
        <span class="sr-only">Showing</span>
        <template v-for="(part, index) in location" :key="part">
          <span
            v-if="index > 0"
            class="text-text-subtle max-[1279px]:hidden"
            aria-hidden="true"
          >
            &#8250;
          </span>
          <span
            class="anchor-chip shrink-0 truncate px-1.5 text-caption leading-5"
            :class="index === location.length - 1 ? 'text-anchor-strong' : 'max-[1279px]:hidden'"
          >
            {{ part }}
          </span>
        </template>
      </span>
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
        class="pointer-events-none absolute right-2 top-1/2 flex -translate-y-1/2 items-center gap-1 max-[1099px]:hidden"
        aria-hidden="true"
      >
        <template v-if="filter.trim()">
          <span class="font-mono text-caption text-text-subtle">{{ resultCount }} found</span>
        </template>
        <kbd
          v-else
          class="rounded-key border border-b-2 border-border-strong bg-surface-raised px-1.5 font-mono text-micro leading-4 text-text-subtle"
        >
          &#8984;K
        </kbd>
      </span>
    </div>

    <div class="flex min-w-0 items-center gap-4">
      <div class="flex flex-1 justify-center">
        <HeaderStatus />
      </div>

      <div class="header-capsule" role="group" aria-label="Tools">
        <!-- The dot's colour carries the save state; the word stays for screen readers and as the tooltip. -->
        <p
          class="grid size-5 place-items-center text-small"
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
          <span class="sr-only">{{ SAVE_LABEL[saveState] }}</span>
        </p>
        <button
          type="button"
          class="header-key focus-ring"
          data-testid="tags-trigger"
          aria-label="Tags"
          title="Tags"
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
        <SettingsTrigger @open="emit('openSettings')" />
        <span class="mx-0.5 h-4 w-px bg-border" aria-hidden="true" />
        <span class="group relative inline-block">
          <AppButton
            variant="primary"
            size="sm"
            :disabled="needsTaskForNote"
            data-testid="new-note"
            title="New note"
            @click="emit('newNote')"
          >
            <svg class="size-3.5 min-[1280px]:hidden" viewBox="0 0 12 12" fill="none" aria-hidden="true">
              <path d="M6 2v8M2 6h8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
            </svg>
            <span class="max-[1279px]:sr-only">New note</span>
          </AppButton>
          <span
            v-if="needsTaskForNote"
            class="pointer-events-none absolute right-0 top-full z-(--z-sticky) mt-2 whitespace-nowrap rounded-control border border-border-strong bg-surface-raised px-2.5 py-1.5 text-small text-text opacity-0 shadow-lift transition-opacity duration-(--motion-quick) group-hover:opacity-100"
          >
            Select a task to create a new note on it
          </span>
        </span>
      </div>
    </div>
  </Teleport>
</template>
