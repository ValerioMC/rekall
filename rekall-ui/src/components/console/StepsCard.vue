<script setup lang="ts">
import { computed } from 'vue'
import { stepIsComplete, type TaskStep } from '@/model/catalog'

const props = defineProps<{
  steps: readonly TaskStep[]
  selected: boolean
}>()

const emit = defineEmits<{ open: [] }>()

const checklist = computed(() => props.steps.filter((step) => step.state !== 'DRAFT'))
const draftCount = computed(() => props.steps.length - checklist.value.length)
const done = computed(() => checklist.value.filter((step) => step.state === 'DONE').length)
const claimed = computed(() => checklist.value.filter((step) => step.state === 'CLAIMED').length)
const running = computed(() => checklist.value.find((step) => step.state === 'RUNNING') ?? null)
const next = computed(() => checklist.value.find((step) => !stepIsComplete(step.state)) ?? null)
const hasSteps = computed(() => checklist.value.length > 0)
const hasBody = computed(() => hasSteps.value || draftCount.value > 0)
</script>

<template>
  <button
    data-testid="steps-card"
    class="dossier-section"
    :class="selected && 'dossier-section-selected'"
    :aria-current="selected"
    @click="emit('open')"
  >
    <span
      class="dossier-node"
      :class="selected ? 'dossier-node-lit' : !hasBody && 'dossier-node-empty'"
      aria-hidden="true"
    >
      <svg class="size-[9px]" viewBox="0 0 12 12" fill="none">
        <path d="M1.2 3.4 2.6 4.8l2.4-2.6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        <path d="M6.8 3.6h4M6.8 8.4h4M1.4 8.4h3.4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      </svg>
    </span>
    <span class="flex min-h-[17px] items-center gap-2">
      <span
        class="text-[11.5px] font-semibold tracking-[0.005em] transition-colors"
        :class="selected ? 'text-text' : 'text-text-muted'"
      >
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
        class="ml-auto shrink-0 font-mono text-[10.5px] tabular-nums"
        :class="done === checklist.length ? 'text-safe' : 'text-text-muted'"
        data-testid="steps-progress"
      >
        {{ done }}/{{ checklist.length }}
      </span>
    </span>

    <template v-if="hasSteps">
      <span class="mt-2 flex h-[4px] gap-[3px]" aria-hidden="true">
        <span
          v-for="step in checklist"
          :key="step.id"
          class="h-full flex-1 rounded-full transition-colors duration-300"
          :class="
            step.state === 'DONE'
              ? done === checklist.length
                ? 'bg-safe/80'
                : 'bg-accent'
              : step.state === 'CLAIMED'
                ? 'ledger-claimed'
                : step.state === 'RUNNING'
                  ? 'ledger-running'
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
      <span
        v-if="draftCount > 0"
        class="mt-1 flex items-center gap-1 text-[11px] leading-relaxed text-text-subtle"
        data-testid="steps-card-drafts"
      >
        <span class="size-[7px] rounded-[2px] border border-dashed border-current" aria-hidden="true" />
        {{ draftCount }} in draft
      </span>
    </template>
    <span v-else class="mt-1 block text-[11.5px] leading-relaxed text-text-muted">
      {{
        draftCount > 0
          ? `${draftCount} step${draftCount > 1 ? 's' : ''} in draft. Open it to promote them.`
          : 'No steps yet. Open it to break this task into what is left to do.'
      }}
    </span>
  </button>
</template>
