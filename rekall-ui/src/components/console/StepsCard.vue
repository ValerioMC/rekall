<script setup lang="ts">
import { computed } from 'vue'
import type { TaskStep } from '@/model/catalog'

/**
 * The checklist, on the way to the pane that holds it, reduced to the one thing a card can say:
 * how much is left, and what is next.
 *
 * It sits between the description and the wrapup because that is the order the three are asked
 * in: what is this task, what is left of it, what did it become. The middle question is the one
 * the other two were being made to answer between them.
 *
 * The next open step is named rather than counted. A number says how much work is left; the
 * title says what it is, which is what you came to this row to find out.
 */
const props = defineProps<{
  steps: readonly TaskStep[]
  selected: boolean
}>()

const emit = defineEmits<{ open: [] }>()

const done = computed(() => props.steps.filter((step) => step.done).length)
const next = computed(() => props.steps.find((step) => !step.done) ?? null)
const hasSteps = computed(() => props.steps.length > 0)
</script>

<template>
  <button
    data-testid="steps-card"
    class="focus-ring mb-2 block w-full rounded-[var(--radius-control)] border p-2.5 text-left transition-all"
    :class="[
      hasSteps
        ? 'border-border-strong bg-surface-raised hover:border-text-subtle'
        : 'border-dashed border-border-strong bg-transparent hover:border-accent hover:bg-accent-soft',
      selected && 'border-accent bg-surface-raised shadow-lift'
    ]"
    :aria-current="selected"
    @click="emit('open')"
  >
    <span class="flex items-center gap-2">
      <svg
        class="size-3 shrink-0 transition-colors"
        :class="selected ? 'text-accent' : hasSteps ? 'text-text-muted' : 'text-text-subtle'"
        viewBox="0 0 12 12"
        fill="none"
        aria-hidden="true"
      >
        <path d="M1.2 3.4 2.6 4.8l2.4-2.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
        <path d="M6.8 3.6h4M6.8 8.4h4M1.4 8.4h3.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      </svg>
      <span class="text-[10px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
        Steps
      </span>
      <span
        v-if="hasSteps"
        class="ml-auto shrink-0 font-mono text-[10.5px] tabular-nums text-text-muted"
        data-testid="steps-progress"
      >
        {{ done }}/{{ steps.length }}
      </span>
    </span>

    <template v-if="hasSteps">
      <!-- One segment per step rather than one bar at a percentage, and the same mark the pane
           carries. A proportion says how far along; segments say how many pieces the work was
           cut into, and which one is next. -->
      <span class="mt-2 flex h-[4px] gap-[3px]" aria-hidden="true">
        <span
          v-for="step in steps"
          :key="step.id"
          class="h-full flex-1 rounded-full transition-colors duration-300"
          :class="
            step.done ? 'bg-accent' : step.id === next?.id ? 'bg-accent/40' : 'bg-border-strong'
          "
        />
      </span>
      <span v-if="next" class="mt-1.5 block truncate text-[12px] leading-relaxed text-text-muted">
        Next: {{ next.title }}
      </span>
      <span v-else class="mt-1.5 block text-[12px] leading-relaxed text-safe">
        Every step is done.
      </span>
    </template>
    <span v-else class="mt-1 block text-[11.5px] leading-relaxed text-text-muted">
      No steps yet. Open it to break this task into what is left to do.
    </span>
  </button>
</template>
