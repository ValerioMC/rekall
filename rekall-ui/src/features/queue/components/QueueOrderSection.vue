<script setup lang="ts">
/**
 * The order of the queue: a relay of numbered beads, the part already run drawn solid and the
 * part still to run in hairline. Only waiting tasks move or come off; the running one is
 * stopped, not removed, and a finished one stays until cleared. It asks, the panel acts.
 */
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import QueueOrderRow from '@/features/queue/components/QueueOrderRow.vue'
import QueueTaskPicker from '@/features/queue/components/QueueTaskPicker.vue'
import { isSettled, type RunProgress, type RunQueue, type RunQueueItem } from '@/features/queue/model/runQueue'
import type { TaskId } from '@/model/branded'

const props = defineProps<{ queue: RunQueue; progress: RunProgress; busy: boolean }>()

const emit = defineEmits<{
  add: [taskId: TaskId]
  /** `index` is the item's new place among the waiting ones. */
  move: [item: RunQueueItem, index: number]
  remove: [item: RunQueueItem]
  clear: []
  watch: [item: RunQueueItem]
}>()

const items = computed(() => props.queue.items)
const waitingIds = computed(() => items.value.filter((item) => item.state === 'QUEUED').map((item) => item.id))
const excluded = computed<ReadonlySet<TaskId>>(
  () => new Set(items.value.filter((item) => !isSettled(item.state)).map((item) => item.taskId))
)
const hasSettled = computed(() => items.value.some((item) => isSettled(item.state)))

function waitingIndex(item: RunQueueItem): number {
  return waitingIds.value.indexOf(item.id)
}

function moveBy(item: RunQueueItem, delta: number): void {
  const index = waitingIndex(item) + delta
  if (index < 0 || index >= waitingIds.value.length) return
  emit('move', item, index)
}
</script>

<template>
  <section class="flex min-h-0 flex-col" aria-labelledby="queue-order-heading">
    <div class="flex items-center gap-2 px-6 pb-2 pt-4">
      <h3 id="queue-order-heading" class="section-label">Order</h3>
      <span class="font-mono text-caption tabular-nums text-text-subtle">
        {{ progress.waiting }} waiting<template v-if="progress.settled">, {{ progress.settled }} run</template>
      </span>
      <AppButton
        v-if="hasSettled"
        class="ml-auto"
        variant="ghost"
        size="sm"
        :disabled="busy"
        data-testid="run-queue-clear"
        @click="emit('clear')"
      >
        Clear finished
      </AppButton>
    </div>

    <div class="min-h-[180px] flex-1 overflow-y-auto px-6 pb-3">
      <div
        v-if="items.length === 0"
        class="flex flex-col items-start gap-1.5 rounded-card border border-dashed border-border px-5 py-8"
        data-testid="run-queue-empty"
      >
        <p class="text-body font-medium text-text">Nothing queued</p>
        <p class="max-w-[46ch] text-small leading-relaxed text-text-muted">
          Add the tasks you want run, in the order you want them. Each one gets a Claude
          session that works its open steps and claims them, then the next one starts.
        </p>
      </div>

      <TransitionGroup v-else tag="ol" name="step-list" class="relative flex flex-col" data-testid="run-queue-list">
        <QueueOrderRow
          v-for="(item, index) in items"
          :key="item.id"
          :item="item"
          :position="index + 1"
          :last="index === items.length - 1"
          :live="queue.state === 'RUNNING'"
          :busy="busy"
          :can-move-earlier="waitingIndex(item) > 0"
          :can-move-later="waitingIndex(item) < waitingIds.length - 1"
          @move="moveBy(item, $event)"
          @remove="emit('remove', item)"
          @watch="emit('watch', item)"
        />
      </TransitionGroup>
    </div>

    <div class="border-t border-border px-6 py-3">
      <QueueTaskPicker :excluded="excluded" :disabled="busy" @pick="emit('add', $event)" />
    </div>
  </section>
</template>
