<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute, useRouter } from 'vue-router'
import ReviewDecision from '@/features/review/components/ReviewDecision.vue'
import ReviewEmpty from '@/features/review/components/ReviewEmpty.vue'
import ReviewQueueList from '@/features/review/components/ReviewQueueList.vue'
import { keyAfterLeaving } from '@/features/review/lib/review-selection'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { useConsoleStore } from '@/stores/console.store'

/**
 * Review at `/review/:key?`: every claimed step and every task handed in whole, longest waiting
 * first, beside the decision on the one in view. The queue is the console's, so its length is the
 * Needs me count, the menu badge and the header segment. A decided item leaves it and the next opens.
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

function show(key: string | null): void {
  const target = key === null ? { name: 'review' } : { name: 'review', params: { key } }
  void router.replace(target)
}

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
  <div class="flex min-h-0 min-w-0 flex-1 flex-col" data-testid="review-page">
    <div v-if="isLoading" class="px-6 pt-6" aria-hidden="true">
      <div class="skeleton h-7 w-40 rounded-control" />
    </div>
    <ReviewEmpty v-else-if="items.length === 0" />
    <div v-else class="grid min-h-0 flex-1 grid-cols-[minmax(0,22rem)_minmax(0,1fr)]">
      <ReviewQueueList :items="items" :current="current?.key ?? null" />
      <div class="min-h-0 overflow-y-auto px-8 py-6">
        <div class="mx-auto max-w-[44rem]">
          <ReviewDecision v-if="current" :key="current.key" :item="current" />
          <p v-else class="pt-16 text-center text-body text-text-muted" data-testid="review-pick-next">
            Decided. Pick the next item from the queue.
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
