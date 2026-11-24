<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute, useRouter } from 'vue-router'
import ReviewClaimCard from '@/features/review/components/ReviewClaimCard.vue'
import ReviewEmpty from '@/features/review/components/ReviewEmpty.vue'
import { useReviewDecisions } from '@/features/review/composables/useReviewDecisions'
import { useReviewEvidence } from '@/features/review/composables/useReviewEvidence'
import { useReviewKeys } from '@/features/review/composables/useReviewKeys'
import { keyAfterLeaving } from '@/features/review/lib/review-selection'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { useConsoleStore } from '@/stores/console.store'

/**
 * Review at `/review/:key?`: every claimed step and every task handed in whole, longest waiting
 * first, each a card that carries its own evidence and keys, the one in view open. The queue is the
 * console's, so its length is the Needs me count, the menu badge and the header segment. A decided
 * item leaves it and the next opens.
 */
const store = useConsoleStore()
const preferences = usePreferencesStore()
const route = useRoute()
const router = useRouter()
const { reviewQueue: items, isLoading } = storeToRefs(store)

const keyInUrl = computed(() => (typeof route.params.key === 'string' ? route.params.key : null))
/** With auto-advance off, a decided item leaves nothing selected until the person picks the next one. */
const waitingForPick = ref(false)
const current = computed(
  () => items.value.find((item) => item.key === keyInUrl.value) ?? (waitingForPick.value ? null : (items.value[0] ?? null))
)

const decisions = useReviewDecisions()
useReviewEvidence(items)

function show(key: string | null): void {
  const target = key === null ? { name: 'review' } : { name: 'review', params: { key } }
  void router.replace(target)
}

function move(by: 1 | -1): void {
  const at = items.value.findIndex((item) => item.key === current.value?.key)
  const target = items.value[at === -1 ? 0 : Math.min(items.value.length - 1, Math.max(0, at + by))]
  if (target) show(target.key)
}

useReviewKeys({
  next: () => move(1),
  previous: () => move(-1),
  accept: () => current.value && void decisions.accept(current.value),
  'send-back': () => current.value && void decisions.sendBack(current.value)
})

// One watcher decides the key in the URL: the item that left is followed by the next, and a missing
// or stale key opens the first. Two watchers would race over the same replace.
watch(
  [items, isLoading, keyInUrl],
  ([now, loading, key], previous) => {
    if (loading) return
    if (key === null) {
      if (!waitingForPick.value) show(now[0]?.key ?? null)
      return
    }
    waitingForPick.value = false
    const left = !now.some((item) => item.key === key)
    if (left && !preferences.reviewAutoAdvance) {
      waitingForPick.value = true
      show(null)
      return
    }
    const target = keyAfterLeaving(previous?.[0] ?? now, now, key)
    if (target !== key) show(target)
  },
  { immediate: true }
)
</script>

<template>
  <div class="flex min-h-0 min-w-0 flex-1 flex-col overflow-y-auto" data-testid="review-page">
    <div v-if="isLoading" class="px-6 pt-6" aria-hidden="true">
      <div class="skeleton h-7 w-40 rounded-control" />
    </div>
    <ReviewEmpty v-else-if="items.length === 0" />
    <div v-else class="page-column mx-auto w-full max-w-[1320px] pb-24 pt-5">
      <header class="flex min-h-10 flex-wrap items-baseline gap-x-3 gap-y-1">
        <h1 class="text-heading font-medium tracking-[-0.01em] text-text">Review</h1>
        <span class="text-small text-text-subtle"><span data-testid="review-count">{{ items.length }}</span> waiting</span>
        <span class="text-small text-text-subtle">Longest waiting first</span>
        <ul class="ml-auto flex items-center gap-4 text-small text-text-subtle" aria-label="Keyboard shortcuts">
          <li class="inline-flex items-center gap-1.5"><kbd>J</kbd><kbd>K</kbd> move</li>
          <li class="inline-flex items-center gap-1.5"><kbd>A</kbd> accept</li>
          <li class="inline-flex items-center gap-1.5"><kbd>S</kbd> send back</li>
        </ul>
      </header>
      <section class="mt-3 flex flex-col gap-2" aria-label="Waiting for review" data-testid="review-list">
        <ReviewClaimCard
          v-for="item in items"
          :key="item.key"
          :item="item"
          :open="current?.key === item.key"
          @open="show(item.key)"
          @accept="decisions.accept(item)"
          @send-back="decisions.sendBack(item)"
        />
      </section>
    </div>
  </div>
</template>
