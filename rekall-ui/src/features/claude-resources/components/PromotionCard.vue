<script setup lang="ts">
import MemoryTypeChip from './MemoryTypeChip.vue'
import { TARGET_LABEL, type PromotionCandidate } from '@/features/claude-resources/model/memory-analysis'

/** The same lesson in several projects: where it would be written once. */
defineProps<{ candidate: PromotionCandidate; groupNames: ReadonlyMap<string, string> }>()
defineEmits<{ open: [path: string] }>()
</script>

<template>
  <article class="rounded-card border border-border bg-surface p-4" data-testid="promotion-card">
    <header class="flex flex-wrap items-center gap-2">
      <span class="inline-flex items-center gap-1.5 rounded-control bg-accent-soft px-2 py-0.5 text-caption font-medium text-accent ring-1 ring-inset ring-accent/25">
        <svg class="size-3" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path v-if="candidate.target === 'skill'" d="M8 2.5 9.6 6.4l4 .3-3 2.6 1 4-3.6-2.2-3.6 2.2 1-4-3-2.6 4-.3z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
          <path v-else d="M3 3.5h10M3 8h10M3 12.5h6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
        {{ TARGET_LABEL[candidate.target] }}
      </span>
      <span class="text-caption text-text-subtle">in {{ candidate.groupCount }} projects</span>
      <ul class="ml-auto flex min-w-0 flex-wrap justify-end gap-1" aria-label="What they are about">
        <li v-for="word in candidate.topic" :key="word" class="rounded-key bg-surface-raised px-1.5 py-px font-mono text-micro text-text-muted">{{ word }}</li>
      </ul>
    </header>

    <p class="mt-2 max-w-prose text-small leading-[1.5] text-text-muted">{{ candidate.reason }}</p>

    <ul class="mt-3 divide-y divide-border rounded-control border border-border">
      <li v-for="memory in candidate.memories" :key="memory.path">
        <button
          type="button"
          class="focus-ring flex w-full cursor-pointer items-center gap-2 px-2.5 py-1.5 text-left transition-colors hover:bg-surface-raised"
          data-testid="review-open-memory"
          @click="$emit('open', memory.path)"
        >
          <span class="min-w-0 flex-1 truncate text-small font-medium text-text">{{ memory.name }}</span>
          <MemoryTypeChip :type="memory.memoryType" />
          <span class="max-w-[40%] shrink-0 truncate font-mono text-micro text-text-subtle">{{ groupNames.get(memory.groupId) ?? memory.groupId }}</span>
        </button>
      </li>
    </ul>
  </article>
</template>
