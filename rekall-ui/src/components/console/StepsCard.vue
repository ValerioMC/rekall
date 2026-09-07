<script setup lang="ts">
import { computed } from 'vue'
import { stepIsComplete, type TaskStep } from '@/model/catalog'

const props = defineProps<{
  steps: readonly TaskStep[]
  selected: boolean
}>()

const emit = defineEmits<{ open: [] }>()

const done = computed(() => props.steps.filter((step) => step.state === 'DONE').length)
const claimed = computed(() => props.steps.filter((step) => step.state === 'CLAIMED').length)
const running = computed(() => props.steps.find((step) => step.state === 'RUNNING') ?? null)
const next = computed(() => props.steps.find((step) => !stepIsComplete(step.state)) ?? null)
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
      selected && 'selected-card'
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
      <span class="eyebrow">
        Steps
      </span>
      <span
        v-if="running"
        class="inline-flex items-center gap-1 rounded-full bg-accent-soft px-1.5 py-px text-[9.5px] font-semibold tracking-[0.02em] text-accent"
        data-testid="steps-card-running"
      >
        <span class="relative grid size-1.5 place-items-center" aria-hidden="true">
          <span class="absolute inline-flex size-1.5 animate-ping rounded-full bg-accent/60" />
          <span class="relative inline-flex size-1 rounded-full bg-accent" />
        </span>
        running
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
      <span class="mt-2 flex h-[4px] gap-[3px]" aria-hidden="true">
        <span
          v-for="step in steps"
          :key="step.id"
          class="h-full flex-1 rounded-full transition-colors duration-300"
          :class="
            step.state === 'DONE'
              ? 'bg-accent'
              : step.state === 'CLAIMED'
                ? 'bg-accent/60'
                : step.state === 'RUNNING'
                  ? 'bg-accent/50 animate-pulse'
                  : step.id === next?.id
                    ? 'bg-accent/25'
                    : 'bg-border-strong'
          "
        />
      </span>
      <span
        v-if="running"
        class="mt-1.5 block truncate text-[12px] leading-relaxed text-accent"
        data-testid="steps-card-line"
      >
        Running: {{ running.title }}
      </span>
      <span
        v-else-if="next"
        class="mt-1.5 block truncate text-[12px] leading-relaxed text-text-muted"
        data-testid="steps-card-line"
      >
        Next: {{ next.title }}
      </span>
      <span
        v-else-if="claimed > 0"
        class="mt-1.5 block text-[12px] leading-relaxed text-accent"
        data-testid="steps-card-line"
      >
        {{ claimed }} step{{ claimed > 1 ? 's' : '' }} awaiting your review.
      </span>
      <span
        v-else
        class="mt-1.5 block text-[12px] leading-relaxed text-safe"
        data-testid="steps-card-line"
      >
        Every step is done.
      </span>
    </template>
    <span v-else class="mt-1 block text-[11.5px] leading-relaxed text-text-muted">
      No steps yet. Open it to break this task into what is left to do.
    </span>
  </button>
</template>
