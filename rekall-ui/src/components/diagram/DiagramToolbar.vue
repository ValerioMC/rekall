<script setup lang="ts">
import { nextTick, ref } from 'vue'
import type { Direction } from '@/common/diagram/graph-layout'
import type { Lens } from '@/common/diagram/graph-projection'

/**
 * The canvas controls in one capsule: the lens, the reading direction, a search that dims
 * everything it does not match, and the zoom. Every control has a key; the title says which.
 */
const props = defineProps<{ lens: Lens; direction: Direction; query: string; scale: number; hasCode: boolean }>()

const emit = defineEmits<{
  'update:lens': [lens: Lens]
  'update:direction': [direction: Direction]
  'update:query': [query: string]
  zoomIn: []
  zoomOut: []
  resetZoom: []
  fit: []
}>()

const search = ref<HTMLInputElement | null>(null)
const searching = ref(false)

async function openSearch(): Promise<void> {
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
  if (event.key === 'Escape') {
    event.stopPropagation()
    closeSearch()
  }
}

function onBlur(): void {
  if (!props.query) searching.value = false
}

defineExpose({ openSearch })

const LENSES: readonly { value: Lens; label: string; hint: string }[] = [
  { value: 'concept', label: 'Concept', hint: 'What the code does, without the functions that do it (L)' },
  { value: 'code', label: 'Code', hint: 'The same pieces framed by the functions that implement them (L)' }
]
</script>

<template>
  <div class="glass flex h-10 items-center gap-1 rounded-[12px] border border-border px-1.5 shadow-lift" role="toolbar" aria-label="Canvas">
    <div class="flex gap-0.5 rounded-[8px] bg-canvas p-0.5" role="group" aria-label="Lens">
      <button
        v-for="option in LENSES"
        :key="option.value"
        class="focus-ring flex h-7 items-center rounded-[6px] px-2.5 text-[12px] transition-colors disabled:cursor-not-allowed disabled:opacity-40"
        :class="lens === option.value ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]' : 'text-text-subtle hover:text-text'"
        :aria-pressed="lens === option.value"
        :title="option.hint"
        :disabled="option.value === 'code' && !hasCode"
        :data-testid="`diagram-lens-${option.value}`"
        @click="emit('update:lens', option.value)"
      >
        {{ option.label }}
      </button>
    </div>

    <span class="mx-0.5 h-5 w-px bg-border" aria-hidden="true" />

    <button
      class="focus-ring grid size-8 place-items-center rounded-[8px] text-text-muted transition-colors hover:bg-surface-raised hover:text-text"
      :title="direction === 'LR' ? 'Read top to bottom (D)' : 'Read left to right (D)'"
      :aria-label="direction === 'LR' ? 'Read top to bottom' : 'Read left to right'"
      data-testid="diagram-direction"
      @click="emit('update:direction', direction === 'LR' ? 'TB' : 'LR')"
    >
      <svg viewBox="0 0 16 16" class="size-4 transition-transform duration-200" :class="direction === 'TB' ? 'rotate-90' : ''" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <rect x="1.8" y="5.6" width="3.6" height="4.8" rx="1" />
        <rect x="10.6" y="5.6" width="3.6" height="4.8" rx="1" />
        <path d="M5.8 8h3.8M8.2 6.4 9.8 8 8.2 9.6" />
      </svg>
    </button>

    <div class="flex items-center">
      <button
        v-if="!searching && !query"
        class="focus-ring grid size-8 place-items-center rounded-[8px] text-text-muted transition-colors hover:bg-surface-raised hover:text-text"
        title="Find an element (/)"
        aria-label="Find an element"
        @click="openSearch"
      >
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
          class="field h-7 w-44 rounded-[7px] pl-2.5 pr-7 text-[12px] text-text"
          data-testid="diagram-search"
          @input="emit('update:query', ($event.target as HTMLInputElement).value)"
          @keydown="onSearchKey"
          @blur="onBlur"
        />
        <button class="focus-ring absolute right-1 grid size-5 place-items-center rounded text-text-subtle hover:text-text" aria-label="Clear the search" @click="closeSearch">
          <svg viewBox="0 0 12 12" class="size-3" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" aria-hidden="true"><path d="m3 3 6 6M9 3 3 9" /></svg>
        </button>
      </label>
    </div>

    <span class="mx-0.5 h-5 w-px bg-border" aria-hidden="true" />

    <button class="focus-ring grid size-8 place-items-center rounded-[8px] text-text-muted transition-colors hover:bg-surface-raised hover:text-text" title="Zoom out (−)" aria-label="Zoom out" @click="emit('zoomOut')">
      <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true"><path d="M4 8h8" /></svg>
    </button>
    <button
      class="focus-ring h-8 min-w-[3.25rem] rounded-[8px] px-1.5 font-mono text-[11.5px] tabular-nums text-text-muted transition-colors hover:bg-surface-raised hover:text-text"
      title="Actual size (0)"
      aria-label="Zoom to actual size"
      data-testid="diagram-zoom-readout"
      @click="emit('resetZoom')"
    >
      {{ Math.round(scale * 100) }}%
    </button>
    <button class="focus-ring grid size-8 place-items-center rounded-[8px] text-text-muted transition-colors hover:bg-surface-raised hover:text-text" title="Zoom in (+)" aria-label="Zoom in" @click="emit('zoomIn')">
      <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true"><path d="M4 8h8M8 4v8" /></svg>
    </button>
    <button class="focus-ring grid size-8 place-items-center rounded-[8px] text-text-muted transition-colors hover:bg-surface-raised hover:text-text" title="Fit the diagram (F)" aria-label="Fit the diagram" data-testid="diagram-fit" @click="emit('fit')">
      <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M2.5 5.5v-3h3M10.5 2.5h3v3M13.5 10.5v3h-3M5.5 13.5h-3v-3" />
      </svg>
    </button>
  </div>
</template>
