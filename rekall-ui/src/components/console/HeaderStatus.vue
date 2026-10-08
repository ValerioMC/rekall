<script setup lang="ts">
/**
 * The header's live readings in one capsule: Claude usage, the run queue, and the review count.
 * Side by side on purpose, since the queue's ceiling is read off the meter next to it. Review is
 * the only amber segment, because it is the only one asking you to act.
 */
import { storeToRefs } from 'pinia'
import ClaudeUsageMeter from '@/components/console/ClaudeUsageMeter.vue'
import QueueBeacon from '@/components/queue/QueueBeacon.vue'
import ReviewQueueButton from '@/components/console/ReviewQueueButton.vue'
import { useConsoleStore } from '@/stores/console.store'

const { reviewQueue } = storeToRefs(useConsoleStore())
</script>

<template>
  <div
    class="header-capsule"
    role="group"
    aria-label="Live status"
    data-testid="header-status"
  >
    <ClaudeUsageMeter />
    <span class="h-4 w-px bg-border" aria-hidden="true" />
    <QueueBeacon />
    <template v-if="reviewQueue.length">
      <span class="h-4 w-px bg-border" aria-hidden="true" />
      <ReviewQueueButton />
    </template>
  </div>
</template>
