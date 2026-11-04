<script setup lang="ts">
import { computed } from 'vue'
import type { TaskDeviation } from '@/features/deviations/model/deviation'

/**
 * The dossier's line for where the work left the plan: one link per step that has a deviation,
 * so a task that deviated often costs one row, not one card each. Past `LISTED` links the rest
 * collapse into a count that opens the first one hidden.
 */
const props = defineProps<{
  deviations: readonly TaskDeviation[]
  /** The step whose deviation is in the main pane, if one is. */
  openStep: number | null
}>()

const emit = defineEmits<{ open: [stepNumber: number] }>()

const LISTED = 6

const listed = computed(() => props.deviations.slice(0, LISTED))
const hidden = computed(() => props.deviations.slice(LISTED))
const isOpen = computed(() => props.openStep !== null)

function describe(deviation: TaskDeviation): string {
  return deviation.stepTitle ? `Step ${deviation.stepNumber}: ${deviation.stepTitle}` : `Step ${deviation.stepNumber}`
}
</script>

<template>
  <div
    v-if="deviations.length"
    class="dossier-seam relative z-[1] pb-2.5 pl-[38px] pr-3 pt-2.5"
    data-testid="deviations-card"
  >
    <span class="dossier-node top-[9px]" :class="isOpen && 'dossier-node-lit'" aria-hidden="true">
      <svg class="size-[9px]" viewBox="0 0 12 12" fill="none">
        <path d="M2.2 1.6v3.2c0 1.2 1 2 2.2 2H7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        <path d="M2.2 4.8v5.6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        <path d="M6.6 4.4 9.4 6.8 6.6 9.2" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </span>

    <div class="flex items-center gap-2">
      <span class="text-caption font-semibold tracking-[0.005em]" :class="isOpen ? 'text-text' : 'text-text-muted'">
        Deviations
      </span>
      <span class="font-mono text-micro tabular-nums text-text-subtle" data-testid="deviations-count">
        {{ deviations.length }}
      </span>
    </div>

    <ul class="mt-1.5 flex flex-wrap gap-x-2.5 gap-y-1" aria-label="Steps the work deviated on">
      <li v-for="deviation in listed" :key="deviation.id">
        <button
          type="button"
          class="focus-ring rounded-key px-1 text-caption transition-colors"
          :class="
            openStep === deviation.stepNumber
              ? 'bg-accent-soft font-semibold text-accent'
              : 'text-accent underline decoration-accent/40 underline-offset-2 hover:decoration-accent'
          "
          :title="describe(deviation)"
          :aria-current="openStep === deviation.stepNumber"
          data-testid="deviation-link"
          @click="emit('open', deviation.stepNumber)"
        >
          Step {{ deviation.stepNumber }}
        </button>
      </li>
      <li v-if="hidden.length">
        <button
          type="button"
          class="focus-ring rounded-key px-1 text-caption text-text-muted underline decoration-text-subtle/50 underline-offset-2 hover:text-text"
          :title="`${hidden.length} more, from step ${hidden[0]!.stepNumber}`"
          data-testid="deviation-more"
          @click="emit('open', hidden[0]!.stepNumber)"
        >
          +{{ hidden.length }} more
        </button>
      </li>
    </ul>
  </div>
</template>
