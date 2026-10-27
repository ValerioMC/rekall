<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import { matchesEpicQuery } from '@/features/catalog/model/epic'
import { TASK_STATUS_LABEL, type Task } from '@/features/catalog/model/catalog'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId } from '@/model/branded'

/**
 * The epic a task belongs to, or none, chosen from the epics of its project. Closed, it is a card
 * naming the current epic; open, a searchable list unfolds in place. `null` in the list is "No epic".
 */
const epicId = defineModel<TaskId | null>({ required: true })

const props = defineProps<{ epics: readonly Task[] }>()

const emit = defineEmits<{ 'open-change': [open: boolean] }>()

const store = useConsoleStore()
const isOpen = ref(false)
const query = ref('')
const highlighted = ref(0)
const searchField = ref<HTMLInputElement | null>(null)
const list = ref<HTMLElement | null>(null)

const current = computed(() => props.epics.find((epic) => epic.id === epicId.value) ?? null)
const matching = computed(() => props.epics.filter((epic) => matchesEpicQuery(epic, query.value)))
const options = computed<(Task | null)[]>(() => (query.value.trim() ? matching.value : [null, ...matching.value]))

watch(isOpen, (open) => emit('open-change', open))
watch(query, () => {
  highlighted.value = 0
})

function taskCount(epic: Task): number {
  return store.tasks.filter((task) => task.epicId === epic.id).length
}

async function open(): Promise<void> {
  query.value = ''
  highlighted.value = Math.max(0, options.value.findIndex((option) => (option?.id ?? null) === epicId.value))
  isOpen.value = true
  await nextTick()
  searchField.value?.focus()
  revealHighlighted()
}

function close(): void {
  isOpen.value = false
}

function pick(option: Task | null): void {
  epicId.value = option?.id ?? null
  close()
}

function revealHighlighted(): void {
  const row = list.value?.querySelector<HTMLElement>(`[data-index="${highlighted.value}"]`)
  row?.scrollIntoView?.({ block: 'nearest' })
}

function move(step: number): void {
  if (options.value.length === 0) return
  highlighted.value = (highlighted.value + step + options.value.length) % options.value.length
  void nextTick(revealHighlighted)
}

function onSearchKeydown(event: KeyboardEvent): void {
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    move(1)
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    move(-1)
  } else if (event.key === 'Enter' && !event.metaKey && !event.ctrlKey) {
    event.preventDefault()
    event.stopPropagation()
    if (highlighted.value < options.value.length) pick(options.value[highlighted.value] ?? null)
  } else if (event.key === 'Escape') {
    event.preventDefault()
    close()
  }
}
</script>

<template>
  <label for="record-epic" class="mb-1.5 mt-5 block eyebrow text-caption">Epic</label>
  <div
    class="overflow-hidden rounded-control border bg-canvas transition-colors"
    :class="isOpen ? 'border-accent' : 'border-border-strong'"
    data-testid="record-epic-picker"
  >
    <button
      id="record-epic"
      type="button"
      class="focus-ring flex w-full cursor-pointer items-center gap-3 px-3 py-2 text-left transition-colors hover:bg-surface"
      data-testid="epic-field-trigger"
      aria-haspopup="listbox"
      :aria-expanded="isOpen"
      @click="isOpen ? close() : open()"
    >
      <span
        class="grid size-7 shrink-0 place-items-center rounded-key border"
        :class="current ? 'border-epic-line bg-epic-soft' : 'border-border-strong text-text-subtle'"
        aria-hidden="true"
      >
        <EpicGlyph v-if="current" :size="15" />
        <svg v-else width="13" height="13" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="5.5" stroke="currentColor" stroke-width="1.4" />
          <path d="m4.2 11.8 7.6-7.6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
      </span>
      <span class="min-w-0 flex-1">
        <template v-if="current">
          <span class="block truncate text-body font-medium text-text" data-testid="epic-field-title">{{ current.title }}</span>
          <span class="mt-0.5 flex items-center gap-2 text-caption text-text-subtle">
            <span class="anchor-chip shrink-0 px-1.5 py-px text-micro leading-[14px]">{{ current.anchor }}</span>
            <span class="truncate">{{ taskCount(current) }} tasks</span>
          </span>
        </template>
        <span v-else class="text-body text-text-subtle" data-testid="epic-field-title">
          {{ epics.length ? 'No epic' : 'No epic in this project yet' }}
        </span>
      </span>
      <span class="shrink-0 eyebrow text-micro text-accent">{{ isOpen ? 'Close' : 'Change' }}</span>
    </button>

    <div v-if="isOpen" class="border-t border-border">
      <div class="flex items-center gap-2 border-b border-border px-3">
        <svg class="size-3.5 shrink-0 text-text-subtle" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <circle cx="11" cy="11" r="6.5" stroke="currentColor" stroke-width="2" />
          <path d="m16 16 4.5 4.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
        </svg>
        <input
          ref="searchField"
          v-model="query"
          type="text"
          data-testid="epic-field-search"
          class="h-10 min-w-0 flex-1 bg-transparent text-body text-text outline-none placeholder:text-text-subtle"
          placeholder="Find an epic by title or label"
          aria-label="Find an epic"
          spellcheck="false"
          autocomplete="off"
          role="combobox"
          aria-expanded="true"
          aria-controls="epic-field-list"
          @keydown="onSearchKeydown"
        />
        <span class="shrink-0 font-mono text-micro text-text-subtle">{{ matching.length }}/{{ epics.length }}</span>
      </div>

      <div id="epic-field-list" ref="list" class="max-h-[220px] overflow-y-auto p-1.5" role="listbox">
        <button
          v-for="(option, index) in options"
          :key="option?.id ?? 'none'"
          type="button"
          role="option"
          data-testid="epic-field-option"
          :data-index="index"
          :aria-selected="(option?.id ?? null) === epicId"
          class="focus-ring flex w-full cursor-pointer items-center gap-2.5 rounded-control px-2 py-1.5 text-left transition-colors"
          :class="[
            index === highlighted ? 'bg-surface-raised' : 'hover:bg-surface',
            (option?.id ?? null) === epicId && 'selected-row'
          ]"
          @mousemove="highlighted = index"
          @click="pick(option)"
        >
          <template v-if="option">
            <EpicGlyph :size="16" />
            <span class="min-w-0 flex-1">
              <span class="block truncate text-body" :class="option.id === epicId ? 'text-text' : 'text-text-muted'">{{ option.title }}</span>
              <span class="block truncate font-mono text-micro text-anchor/70">{{ option.anchor }}</span>
            </span>
            <span v-if="option.status === 'DONE'" class="shrink-0 rounded-key border border-border-strong px-1 py-px text-micro text-text-subtle">
              {{ TASK_STATUS_LABEL[option.status] }}
            </span>
            <span class="shrink-0 font-mono text-caption text-text-subtle">{{ taskCount(option) }}</span>
          </template>
          <span v-else class="py-1 text-body" :class="epicId === null ? 'text-text' : 'text-text-muted'">No epic</span>
        </button>
        <p v-if="matching.length === 0" class="px-2 py-3 text-small text-text-subtle" data-testid="epic-field-empty">
          <template v-if="query.trim()">No epic matches “{{ query.trim() }}”.</template>
          <template v-else>No epic in this project yet. Create a task of kind Epic to gather tasks under it.</template>
        </p>
      </div>
    </div>
  </div>
</template>
