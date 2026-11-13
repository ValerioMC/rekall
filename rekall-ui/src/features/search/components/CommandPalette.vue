<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AppDialog from '@/components/ui/AppDialog.vue'
import PaletteList from '@/features/search/components/PaletteList.vue'
import { useTextSearch } from '@/features/search/composables/useTextSearch'
import { flattenSections, optionId, paletteSections } from '@/features/search/lib/palette-entries'
import type { PaletteEntry } from '@/features/search/model/palette'
import { usePaletteStore } from '@/features/search/store/palette.store'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId } from '@/model/branded'

/**
 * Moves between screens, projects, tasks and notes from one field, and finds a phrase in the text
 * behind them. Opening a task goes to its page.
 */
const palette = usePaletteStore()
const store = useConsoleStore()
const router = useRouter()

const query = ref('')
const input = ref<HTMLInputElement | null>(null)
const activeIndex = ref(0)
const { hits, searching, failed } = useTextSearch(query)

const needsReview = computed(() => {
  const counts = new Map<TaskId, number>()
  for (const item of store.reviewQueue) counts.set(item.taskId, (counts.get(item.taskId) ?? 0) + 1)
  return counts
})

const sections = computed(() =>
  paletteSections(query.value, {
    projects: store.projects,
    tasks: store.tasks,
    documents: store.documents,
    needsReview: needsReview.value,
    textHits: hits.value
  })
)
const entries = computed(() => flattenSections(sections.value))
const active = computed<PaletteEntry | null>(() => entries.value[activeIndex.value] ?? null)

const emptyMessage = computed(() => {
  if (searching.value) return 'Searching descriptions, steps, wrapups and notes…'
  if (failed.value) return 'The text search did not answer. Titles and anchors still match.'
  return 'Nothing matches.'
})

function move(delta: number): void {
  if (entries.value.length === 0) return
  activeIndex.value = (activeIndex.value + delta + entries.value.length) % entries.value.length
}

function open(entry: PaletteEntry): void {
  palette.hide()
  if (entry.target.kind === 'hit') store.openSearchHit(entry.target.hit)
  void router.push(entry.target.to)
}

function openActive(): void {
  if (active.value) open(active.value)
}

function select(entry: PaletteEntry): void {
  activeIndex.value = entries.value.indexOf(entry)
}

watch(query, () => (activeIndex.value = 0))

watch(active, async (entry) => {
  if (!entry) return
  await nextTick()
  document.getElementById(optionId(entry))?.scrollIntoView({ block: 'nearest' })
})

onMounted(() => input.value?.focus())
</script>

<template>
  <AppDialog
    label="Search"
    panel-class="max-w-[620px] self-start mt-[12vh]"
    panel-testid="command-palette"
    @dismiss="palette.hide()"
  >
    <div class="flex items-center gap-3 border-b border-border px-4">
      <span class="font-mono text-small text-anchor" aria-hidden="true">/rk</span>
      <input
        ref="input"
        v-model="query"
        data-testid="palette-input"
        type="text"
        role="combobox"
        autocomplete="off"
        spellcheck="false"
        class="h-12 min-w-0 flex-1 bg-transparent font-mono text-small text-text outline-none placeholder:text-text-subtle"
        placeholder="Search tasks, projects, notes, screens, or an anchor"
        aria-label="Search"
        aria-expanded="true"
        aria-controls="palette-list"
        :aria-activedescendant="active ? optionId(active) : undefined"
        @keydown.down.prevent="move(1)"
        @keydown.up.prevent="move(-1)"
        @keydown.enter.prevent="openActive"
      />
    </div>
    <PaletteList
      v-if="entries.length > 0"
      :sections="sections"
      :active-key="active?.key ?? null"
      @pick="open"
      @hover="select"
    />
    <p v-else class="px-4 py-5 text-small text-text-muted" role="status" data-testid="palette-empty">
      {{ emptyMessage }}
    </p>
    <p class="flex gap-4 border-t border-border px-4 py-2 text-caption text-text-subtle" aria-hidden="true">
      <span><kbd class="font-mono">&#8593;</kbd> <kbd class="font-mono">&#8595;</kbd> walk</span>
      <span><kbd class="font-mono">&#9166;</kbd> open</span>
      <span><kbd class="font-mono">esc</kbd> close</span>
    </p>
  </AppDialog>
</template>
