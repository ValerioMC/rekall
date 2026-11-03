<script setup lang="ts">
import { computed, watch } from 'vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useWrapupFreshnessStore } from '@/features/wrapup/store/wrapup-freshness.store'
import type { TaskId } from '@/model/branded'

/**
 * Marks a wrapup whose named files changed in commits made after it was written. A guess from the
 * paths between backticks, so it says "possibly" and lists what it saw; it shows nothing otherwise.
 */
const props = defineProps<{ taskId: TaskId; wrapupUpdatedAt: string }>()

const store = useWrapupFreshnessStore()
const { run } = useAsyncAction()

const freshness = computed(() => store.freshnessOf(props.taskId))

watch(
  () => [props.taskId, props.wrapupUpdatedAt],
  () => void run(() => store.load(props.taskId)),
  { immediate: true }
)

function shortCommit(hash: string): string {
  return hash.slice(0, 7)
}
</script>

<template>
  <details
    v-if="freshness.state === 'possiblyStale'"
    class="rounded-control bg-warn-soft px-2 py-0.5 text-caption text-warn"
    data-testid="wrapup-freshness"
  >
    <summary class="focus-ring flex cursor-pointer list-none items-center gap-1.5 rounded-key">
      <span class="size-[5px] shrink-0 rounded-full bg-warn" aria-hidden="true" />
      Possibly stale &middot; {{ freshness.changedPaths.length }} named
      file{{ freshness.changedPaths.length === 1 ? '' : 's' }} changed in
      {{ freshness.commits.length }} commit{{ freshness.commits.length === 1 ? '' : 's' }} since it was written
    </summary>
    <ul class="mt-1 flex flex-col gap-0.5 font-mono text-micro" data-testid="wrapup-freshness-paths">
      <li v-for="path in freshness.changedPaths" :key="path">{{ path }}</li>
    </ul>
    <p class="mt-1 font-mono text-micro" data-testid="wrapup-freshness-commits">
      {{ freshness.commits.map(shortCommit).join(' ') }}
    </p>
  </details>
</template>
