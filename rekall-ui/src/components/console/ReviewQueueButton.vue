<script setup lang="ts">
import { onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { relativeTime } from '@/common/format/relative-time'
import type { ReviewItem } from '@/model/review'

const store = useConsoleStore()
const { reviewQueue } = storeToRefs(store)

const open = ref(false)
const host = ref<HTMLElement | null>(null)

function pick(item: ReviewItem): void {
  open.value = false
  store.openReviewItem(item)
}

function onPointerDown(event: MouseEvent): void {
  if (host.value && !host.value.contains(event.target as Node)) open.value = false
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') open.value = false
}

watch(open, (value) => {
  if (value) {
    window.addEventListener('mousedown', onPointerDown, true)
    window.addEventListener('keydown', onKeydown, true)
  } else {
    window.removeEventListener('mousedown', onPointerDown, true)
    window.removeEventListener('keydown', onKeydown, true)
  }
})

// The popover has nothing to list once the queue empties under it.
watch(
  () => reviewQueue.value.length,
  (count) => {
    if (count === 0) open.value = false
  }
)

onUnmounted(() => {
  window.removeEventListener('mousedown', onPointerDown, true)
  window.removeEventListener('keydown', onKeydown, true)
})
</script>

<template>
  <div v-if="reviewQueue.length" ref="host" class="relative">
    <button
      type="button"
      class="focus-ring inline-flex h-8 shrink-0 items-center gap-1 rounded-[var(--radius-control)] border border-accent/60 bg-accent-soft px-2 text-accent transition-colors hover:border-accent"
      :title="`Review: ${reviewQueue.length} waiting for you, across every task`"
      :aria-label="`Review, ${reviewQueue.length} waiting`"
      aria-haspopup="dialog"
      :aria-expanded="open"
      data-testid="review-queue-open"
      @click="open = !open"
    >
      <svg class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <path d="M1.2 3.4 2.6 4.8l2.4-2.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
        <path d="M6.8 3.6h4M6.8 8.4h4M1.4 8.4h3.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      </svg>
      <span class="rounded-full bg-accent px-1.5 font-mono text-[10px] tabular-nums leading-[15px] text-accent-ink" data-testid="review-queue-count">
        {{ reviewQueue.length }}
      </span>
    </button>

    <Transition name="popover">
      <div
        v-if="open"
        class="absolute right-0 top-full z-(--z-overlay) mt-1.5 w-[380px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
        role="dialog"
        aria-label="Waiting for review"
        data-testid="review-queue"
      >
        <p class="border-b border-border px-3.5 py-2 text-[11px] text-text-subtle">
          Claimed by a session, waiting for you · longest-waiting first
        </p>
        <ul class="max-h-[420px] overflow-y-auto py-1">
          <li v-for="item in reviewQueue" :key="item.key">
            <button
              type="button"
              class="focus-ring flex w-full flex-col items-start gap-0.5 px-3.5 py-2 text-left transition-colors hover:bg-surface-raised"
              data-testid="review-queue-item"
              @click="pick(item)"
            >
              <span class="w-full truncate text-[12.5px] font-medium text-text">
                {{ item.stepTitle ?? item.taskTitle }}
              </span>
              <span class="flex w-full items-baseline gap-2 text-[11px] text-text-muted">
                <span class="min-w-0 truncate">{{ item.stepTitle ? item.taskTitle : 'Whole task, no checklist' }}</span>
                <span v-if="item.claimedAt" class="ml-auto shrink-0">claimed {{ relativeTime(item.claimedAt) }}</span>
              </span>
            </button>
          </li>
        </ul>
      </div>
    </Transition>
  </div>
</template>
