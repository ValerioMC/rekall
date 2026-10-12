<script setup lang="ts">
import { computed } from 'vue'
import { findingCount, type MemoryAnalysis } from '@/model/memory-analysis'

/** The pinned way into the memory review, with how many suggestions the last reading found. */
const props = defineProps<{ analysis: MemoryAnalysis | null; active: boolean }>()
defineEmits<{ open: [] }>()

const count = computed(() => (props.analysis ? findingCount(props.analysis) : null))
</script>

<template>
  <button
    type="button"
    class="focus-ring flex w-full cursor-pointer items-center gap-2.5 rounded-control px-2.5 py-1.5 text-left transition-colors"
    :class="active ? 'selected-row text-text' : 'text-text-muted hover:bg-surface-raised hover:text-text'"
    :aria-current="active ? 'true' : undefined"
    data-testid="review-entry"
    @click="$emit('open')"
  >
    <svg class="size-4 shrink-0 text-accent" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path d="M3 3.5h10M3 8h6M3 12.5h4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      <path d="m10.5 11.5 1.3 1.3 2.7-3" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
    <span class="min-w-0 flex-1">
      <span class="block truncate text-body font-medium leading-[1.35]">Memory review</span>
      <span class="block truncate text-caption leading-[1.4] text-text-subtle">Repeated and already-covered memories</span>
    </span>
    <span
      v-if="count !== null"
      class="shrink-0 rounded-key px-1.5 py-px font-mono text-micro tabular-nums"
      :class="count > 0 ? 'bg-accent-soft text-accent' : 'bg-surface-raised text-text-subtle'"
      :aria-label="`${count} suggestions`"
      data-testid="review-count"
    >
      {{ count }}
    </span>
  </button>
</template>
