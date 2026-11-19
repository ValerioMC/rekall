<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { RouterLink } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'

/** The header's review segment: how many claims wait, and the way to the Review screen that lists them. */
const { reviewQueue } = storeToRefs(useConsoleStore())
</script>

<template>
  <RouterLink
    v-if="reviewQueue.length"
    :to="{ name: 'review' }"
    class="focus-ring inline-flex h-7 shrink-0 items-center gap-1.5 rounded-control bg-accent-soft px-2.5 text-small font-medium text-accent-strong transition-colors duration-(--motion-quick) hover:bg-accent/25"
    :title="`Review: ${reviewQueue.length} waiting for you, across every task`"
    :aria-label="`Review, ${reviewQueue.length} waiting`"
    data-testid="review-queue-open"
  >
    <svg class="size-3.5 text-accent" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <path d="M1.2 3.4 2.6 4.8l2.4-2.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M6.8 3.6h4M6.8 8.4h4M1.4 8.4h3.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
    </svg>
    <span class="font-mono tabular-nums" data-testid="review-queue-count">{{ reviewQueue.length }}</span>
    <span class="@max-[34rem]:sr-only">to review</span>
  </RouterLink>
</template>
