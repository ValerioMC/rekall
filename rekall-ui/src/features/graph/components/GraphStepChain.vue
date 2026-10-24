<script setup lang="ts">
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import type { ChainLink } from '@/features/graph/lib/graph-chain'

/** A graph's steps in order, each with the logo of what it reaches, joined by arrows. */
defineProps<{ links: readonly ChainLink[]; loops?: boolean }>()
</script>

<template>
  <ol class="flex flex-wrap items-center gap-x-1.5 gap-y-1 text-caption text-text-muted" data-testid="graph-chain">
    <li v-for="(link, index) in links" :key="index" class="flex items-center gap-1.5">
      <span v-if="index > 0" class="text-text-subtle" aria-hidden="true">&rarr;</span>
      <span class="inline-flex items-center gap-1 rounded-full border border-border bg-surface-raised py-0.5 pl-1.5 pr-2">
        <ConnectionKindIcon v-if="link.kind" :kind="link.kind" :size="11" />
        <span v-else class="size-[7px] rounded-full bg-border-strong" aria-hidden="true" />
        <span class="whitespace-nowrap">{{ link.title }}</span>
      </span>
    </li>
    <li v-if="loops" class="flex items-center gap-1 text-text-subtle" title="A failure goes back to an earlier step">
      <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path d="M13 8a5 5 0 1 1-1.6-3.7M13 2.5V5h-2.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      <span>loops back on failure</span>
    </li>
  </ol>
</template>
