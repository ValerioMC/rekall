<script setup lang="ts">
import { PALETTE_SCOPES } from '@/features/search/model/palette'
import type { PaletteCounts, PaletteScope } from '@/features/search/model/palette'

/**
 * The groups of the results as tabs with what each found, so the list can be narrowed to one
 * of them. The palette's Tab key walks them; a press picks one.
 */
defineProps<{ scope: PaletteScope; counts: PaletteCounts }>()
const emit = defineEmits<{ select: [scope: PaletteScope] }>()

function countOf(counts: PaletteCounts, scope: PaletteScope): number | null {
  return scope === 'all' ? null : counts[scope]
}
</script>

<template>
  <div role="tablist" aria-label="Narrow the results" class="flex gap-1 border-b border-border px-3 py-2">
    <button
      v-for="option in PALETTE_SCOPES"
      :key="option"
      type="button"
      role="tab"
      tabindex="-1"
      :aria-selected="option === scope"
      class="flex h-7 cursor-pointer items-center gap-1.5 whitespace-nowrap rounded-control px-2.5 text-small font-medium transition-colors duration-(--motion-quick)"
      :class="[
        option === scope ? 'bg-surface-raised text-text ring-1 ring-inset ring-border-strong' : 'text-text-muted hover:bg-surface-raised hover:text-text',
        countOf(counts, option) === 0 && option !== scope && 'opacity-60'
      ]"
      data-testid="palette-scope"
      @mousedown.prevent="emit('select', option)"
    >
      {{ option === 'all' ? 'All' : option }}
      <span v-if="countOf(counts, option) !== null" class="text-caption font-normal tabular-nums text-text-subtle">{{ countOf(counts, option) }}</span>
    </button>
  </div>
</template>
