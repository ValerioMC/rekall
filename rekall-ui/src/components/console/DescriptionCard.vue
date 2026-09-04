<script setup lang="ts">
import { computed } from 'vue'
import { excerpt as previewOf } from '@/common/format/excerpt'

/**
 * The brief of the task in view, on the way to the pane that holds it.
 *
 * It sits above the wrapup because it is the older of the two answers: what the work is, then
 * where the work got to. The same card in both states, and the glyph carries the difference —
 * a written page against an empty one — so a task nobody has described is visible as a fact
 * rather than as a missing row.
 */
const props = defineProps<{
  description: string | null
  selected: boolean
}>()

const emit = defineEmits<{ open: [] }>()

const excerpt = computed(() => previewOf(props.description ?? ''))

const hasBody = computed(() => (props.description ?? '').trim().length > 0)
</script>

<template>
  <button
    data-testid="description-card"
    class="focus-ring mb-2 block w-full rounded-[var(--radius-control)] border p-2.5 text-left transition-all"
    :class="[
      hasBody
        ? 'border-border-strong bg-surface-raised hover:border-text-subtle'
        : 'border-dashed border-border-strong bg-transparent hover:border-accent hover:bg-accent-soft',
      selected && 'selected-card'
    ]"
    :aria-current="selected"
    @click="emit('open')"
  >
    <span class="flex items-center gap-2">
      <svg
        class="size-3 shrink-0 transition-colors"
        :class="selected ? 'text-accent' : hasBody ? 'text-text-muted' : 'text-text-subtle'"
        viewBox="0 0 12 12"
        fill="none"
        aria-hidden="true"
      >
        <path
          d="M2.6 1.1h4.1l2.7 2.7v7.1H2.6z"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linejoin="round"
        />
        <path
          v-if="hasBody"
          d="M4.5 6.1h3.2M4.5 8.2h2"
          stroke="currentColor"
          stroke-width="1.1"
          stroke-linecap="round"
        />
      </svg>
      <span class="eyebrow">
        Description
      </span>
    </span>

    <span v-if="hasBody" class="mt-1 line-clamp-2 block text-[12px] leading-relaxed text-text-muted">
      {{ excerpt }}
    </span>
    <span v-else class="mt-1 block text-[11.5px] leading-relaxed text-text-muted">
      No description yet. Open it to write the brief.
    </span>
  </button>
</template>
