<script setup lang="ts">
import { SEARCH_HIT_LABEL } from '@/model/search'
import type { SearchHit } from '@/model/search'

defineProps<{
  hits: readonly SearchHit[]
  activeIndex: number
  searching: boolean
  failed: boolean
  term: string
}>()
const emit = defineEmits<{ pick: [hit: SearchHit]; hover: [index: number] }>()
</script>

<template>
  <div
    class="absolute left-0 top-full z-(--z-overlay) mt-1.5 w-[min(620px,calc(100vw-2rem))] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
    data-testid="search-hits"
  >
    <p class="border-b border-border px-3.5 py-2 text-[11px] text-text-subtle">
      <template v-if="failed">The text search did not answer. The filter above still works on titles.</template>
      <template v-else-if="searching && hits.length === 0">Searching descriptions, steps, wrapups and notes…</template>
      <template v-else-if="hits.length === 0">Nothing in any description, step, wrapup or note says “{{ term }}”.</template>
      <template v-else>In the text · ↑↓ to choose, Enter to open</template>
    </p>
    <ul v-if="hits.length" role="listbox" aria-label="Found in the text" class="max-h-[420px] overflow-y-auto py-1">
      <li
        v-for="(hit, index) in hits"
        :key="`${hit.kind}-${hit.stepId ?? hit.documentId ?? hit.taskId}`"
        role="option"
        :aria-selected="index === activeIndex"
        class="cursor-pointer px-3.5 py-2 transition-colors"
        :class="index === activeIndex ? 'bg-accent-soft' : 'hover:bg-surface-raised'"
        data-testid="search-hit"
        @mousedown.prevent="emit('pick', hit)"
        @mousemove="emit('hover', index)"
      >
        <span class="flex items-baseline gap-2">
          <span
            class="shrink-0 rounded-full border border-border px-1.5 text-[10px] font-semibold uppercase tracking-[0.04em] text-text-subtle"
          >
            {{ SEARCH_HIT_LABEL[hit.kind] }}
          </span>
          <span class="min-w-0 truncate text-[12.5px] font-medium text-text">{{ hit.title }}</span>
          <span class="ml-auto shrink-0 truncate font-mono text-[10.5px] text-anchor">{{ hit.where }}</span>
        </span>
        <span class="mt-0.5 block truncate text-[11.5px] text-text-muted">{{ hit.excerpt }}</span>
      </li>
    </ul>
  </div>
</template>
