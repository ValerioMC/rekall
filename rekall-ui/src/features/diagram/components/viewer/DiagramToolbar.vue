<script setup lang="ts">
import { nextTick, ref } from 'vue'

/**
 * The canvas controls in one capsule: a search that dims what it does not match, and the zoom.
 * Every control has a key, and its title says which.
 */
const props = defineProps<{ query: string; scale: number; searchable: boolean }>()

const emit = defineEmits<{ 'update:query': [query: string]; zoomIn: []; zoomOut: []; resetZoom: []; fit: [] }>()

const search = ref<HTMLInputElement | null>(null)
const searching = ref(false)

async function openSearch(): Promise<void> {
  if (!props.searchable) return
  searching.value = true
  await nextTick()
  search.value?.focus()
  search.value?.select()
}

function closeSearch(): void {
  emit('update:query', '')
  searching.value = false
}

function onSearchKey(event: KeyboardEvent): void {
  if (event.key !== 'Escape') return
  event.stopPropagation()
  closeSearch()
}

function onBlur(): void {
  if (!props.query) searching.value = false
}

defineExpose({ openSearch })

const KEY = 'focus-ring grid size-8 place-items-center rounded-card text-text-muted transition-colors hover:bg-surface-raised hover:text-text'
</script>

<template>
  <div class="glass flex h-10 items-center gap-1 rounded-card border border-border px-1.5 shadow-lift" role="toolbar" aria-label="Canvas">
    <template v-if="searchable">
      <button v-if="!searching && !query" :class="KEY" title="Find an element (/)" aria-label="Find an element" data-testid="diagram-search-open" @click="openSearch">
        <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" aria-hidden="true">
          <circle cx="7" cy="7" r="4.2" />
          <path d="m10.2 10.2 3.3 3.3" />
        </svg>
      </button>
      <label v-else class="relative flex items-center">
        <span class="sr-only">Find an element</span>
        <input
          ref="search"
          :value="query"
          type="search"
          placeholder="Find an element"
          class="field h-7 w-44 rounded-control pl-2.5 pr-7 text-small text-text"
          data-testid="diagram-search"
          @input="emit('update:query', ($event.target as HTMLInputElement).value)"
          @keydown="onSearchKey"
          @blur="onBlur"
        />
        <button class="focus-ring hit-target absolute right-1 grid size-5 place-items-center rounded-key text-text-subtle hover:text-text" aria-label="Clear the search" @click="closeSearch">
          <svg viewBox="0 0 12 12" class="size-3" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" aria-hidden="true"><path d="m3 3 6 6M9 3 3 9" /></svg>
        </button>
      </label>
      <span class="mx-0.5 h-5 w-px bg-border" aria-hidden="true" />
    </template>

    <button :class="KEY" title="Zoom out (−)" aria-label="Zoom out" @click="emit('zoomOut')">
      <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true"><path d="M4 8h8" /></svg>
    </button>
    <button
      class="focus-ring h-8 min-w-[3.25rem] rounded-card px-1.5 font-mono text-caption tabular-nums text-text-muted transition-colors hover:bg-surface-raised hover:text-text"
      title="Actual size (0)"
      aria-label="Zoom to actual size"
      data-testid="diagram-zoom-readout"
      @click="emit('resetZoom')"
    >
      {{ Math.round(scale * 100) }}%
    </button>
    <button :class="KEY" title="Zoom in (+)" aria-label="Zoom in" @click="emit('zoomIn')">
      <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true"><path d="M4 8h8M8 4v8" /></svg>
    </button>
    <button :class="KEY" title="Fit the sheet (F)" aria-label="Fit the sheet" data-testid="diagram-fit" @click="emit('fit')">
      <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M2.5 5.5v-3h3M10.5 2.5h3v3M13.5 10.5v3h-3M5.5 13.5h-3v-3" />
      </svg>
    </button>
  </div>
</template>
