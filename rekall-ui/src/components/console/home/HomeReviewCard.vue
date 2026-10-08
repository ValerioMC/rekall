<script setup lang="ts">
/** The work sessions claimed and nobody has reviewed: the console's most frequent next action, so the one amber card. */
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { relativeTime } from '@/common/format/relative-time'

const SHOWN = 5

const store = useConsoleStore()
const { reviewQueue } = storeToRefs(store)

const shown = computed(() => reviewQueue.value.slice(0, SHOWN))
const hidden = computed(() => Math.max(0, reviewQueue.value.length - SHOWN))
</script>

<template>
  <section
    class="flex flex-col rounded-card border bg-surface p-1.5 shadow-(--shadow-raised)"
    :class="reviewQueue.length ? 'border-accent-deep' : 'border-border'"
    aria-labelledby="home-review-title"
    data-testid="home-review"
  >
    <header class="flex items-baseline justify-between gap-3 px-2.5 pb-1.5 pt-2">
      <h2
        id="home-review-title"
        class="eyebrow uppercase tracking-[0.12em]"
        :class="reviewQueue.length > 0 && 'text-accent'"
      >
        Waiting for your review
      </h2>
      <span class="text-caption text-text-subtle">longest-waiting first</span>
    </header>

    <ul v-if="reviewQueue.length" class="flex flex-col">
      <li v-for="item in shown" :key="item.key">
        <button
          type="button"
          class="focus-ring flex w-full items-center gap-3 rounded-control px-2.5 py-2 text-left transition-colors duration-(--motion-quick) hover:bg-surface-raised"
          data-testid="home-review-item"
          @click="store.openReviewItem(item)"
        >
          <span
            class="size-5 shrink-0 rounded-full border-[1.5px] border-dashed border-accent"
            aria-hidden="true"
          />
          <span class="min-w-0 flex-1">
            <span class="block truncate text-body text-text">{{ item.stepTitle ?? item.taskTitle }}</span>
            <span class="mt-0.5 block truncate font-mono text-caption text-anchor">{{ item.anchor }}</span>
          </span>
          <span v-if="item.claimedAt" class="shrink-0 text-caption text-text-subtle">
            {{ relativeTime(item.claimedAt) }}
          </span>
        </button>
      </li>
      <li v-if="hidden" class="px-2.5 pb-1.5 pt-1 text-caption text-text-subtle">
        and {{ hidden }} more in the review list above
      </li>
    </ul>

    <p v-else class="px-2.5 pb-3 pt-1 text-small leading-relaxed text-text-muted">
      Nothing to review. A step a session claims lands here until you tick it.
    </p>
  </section>
</template>
