<script setup lang="ts">
import { computed } from 'vue'
import { excerpt as previewOf } from '@/common/format/excerpt'
import type { TaskStepState } from '@/model/catalog'

const props = defineProps<{
  description: string | null
  selected: boolean
  reviewState?: TaskStepState | null
}>()

const emit = defineEmits<{ open: [] }>()

const excerpt = computed(() => previewOf(props.description ?? ''))

const hasBody = computed(() => (props.description ?? '').trim().length > 0)

const running = computed(() => props.reviewState === 'RUNNING')
const claimed = computed(() => props.reviewState === 'CLAIMED')
const accepted = computed(() => props.reviewState === 'DONE')
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
      <span
        v-if="running"
        class="inline-flex items-center gap-1 rounded-full bg-accent-soft px-1.5 py-px text-[9.5px] font-semibold tracking-[0.02em] text-accent"
        data-testid="description-card-running"
      >
        <span class="relative grid size-1.5 place-items-center" aria-hidden="true">
          <span class="absolute inline-flex size-1.5 animate-ping rounded-full bg-accent/60" />
          <span class="relative inline-flex size-1 rounded-full bg-accent" />
        </span>
        running
      </span>
      <span
        v-else-if="claimed"
        class="rounded-full border border-accent/40 px-1.5 py-px text-[9.5px] font-semibold tracking-[0.02em] text-accent"
        data-testid="description-card-claimed"
      >
        Awaiting review
      </span>
    </span>

    <span v-if="hasBody" class="mt-1 line-clamp-2 block text-[12px] leading-relaxed text-text-muted">
      {{ excerpt }}
    </span>
    <span v-else class="mt-1 block text-[11.5px] leading-relaxed text-text-muted">
      No description yet. Open it to write the brief.
    </span>

    <span
      v-if="running"
      class="mt-1.5 block text-[12px] leading-relaxed text-text-muted"
      data-testid="description-card-line"
    >
      Session running. Open it to accept, or wait for the wrapup.
    </span>
    <span
      v-else-if="claimed"
      class="mt-1.5 block text-[12px] leading-relaxed text-accent"
      data-testid="description-card-line"
    >
      Awaiting your review.
    </span>
    <span
      v-else-if="accepted"
      class="mt-1.5 block text-[12px] leading-relaxed text-safe"
      data-testid="description-card-line"
    >
      Accepted.
    </span>
  </button>
</template>
