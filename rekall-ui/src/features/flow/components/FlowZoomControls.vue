<script setup lang="ts">
/** The canvas keys: tidy the layout when steps were moved, zoom out and in with the level between, fit all. */
defineProps<{ percent: number; tidyable: boolean }>()

defineEmits<{
  tidy: []
  'zoom-out': []
  'zoom-in': []
  fit: []
}>()

const KEY =
  'focus-ring grid size-7 cursor-pointer place-items-center rounded-control text-text-muted transition-colors hover:bg-surface-raised hover:text-text'
</script>

<template>
  <div class="flex items-center gap-0.5 rounded-card border border-border bg-surface p-0.5 shadow-raised" data-testid="flow-zoom">
    <template v-if="tidyable">
      <button type="button" :class="KEY" title="Tidy the layout" aria-label="Tidy the layout" data-testid="flow-tidy" @click="$emit('tidy')">
        <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <rect x="2.5" y="2.5" width="4.5" height="4.5" rx="1" stroke="currentColor" stroke-width="1.3" />
          <rect x="9" y="9" width="4.5" height="4.5" rx="1" stroke="currentColor" stroke-width="1.3" />
          <path d="M7 4.7h2.5A1.5 1.5 0 0 1 11 6.2V9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
        </svg>
      </button>
      <span class="mx-0.5 h-4 w-px bg-border" aria-hidden="true" />
    </template>
    <button type="button" :class="KEY" title="Zoom out" aria-label="Zoom out" data-testid="flow-zoom-out" @click="$emit('zoom-out')">
      <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="M4 8h8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" /></svg>
    </button>
    <span class="w-10 text-center font-mono text-micro tabular-nums text-text-subtle" data-testid="flow-zoom-level">{{ percent }}%</span>
    <button type="button" :class="KEY" title="Zoom in" aria-label="Zoom in" data-testid="flow-zoom-in" @click="$emit('zoom-in')">
      <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="M4 8h8M8 4v8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" /></svg>
    </button>
    <button type="button" :class="KEY" title="Fit to view" aria-label="Fit to view" data-testid="flow-fit" @click="$emit('fit')">
      <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path d="M2.5 6V2.5H6M10 2.5h3.5V6M13.5 10v3.5H10M6 13.5H2.5V10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>
  </div>
</template>
