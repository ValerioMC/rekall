<script setup lang="ts">
/**
 * The review line of a task with no checklist: what a session has done with it, and the answers
 * left to give. Corrections are written in the thread under the description, not here.
 */
import AppButton from '@/components/ui/AppButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { Task, TaskStepState } from '@/features/catalog/model/catalog'

const props = defineProps<{ task: Task; reviewState: Exclude<TaskStepState, 'OPEN'> }>()

const store = useConsoleStore()
const { run } = useAsyncAction()

async function acceptDescription(): Promise<void> {
  const id = props.task.id
  await run(() => store.acceptTask(id))
}

async function markTaskDone(): Promise<void> {
  const id = props.task.id
  await run(() => store.setTaskStatus(id, 'DONE'))
}
</script>

<template>
  <div
    class="shrink-0 border-b border-border bg-accent-soft/40 px-5 py-2.5"
    data-testid="description-review-bar"
  >
    <div class="flex flex-wrap items-center gap-2">
      <span class="min-w-0 flex-1 text-caption leading-snug text-text-muted">
        <template v-if="reviewState === 'RUNNING'">
          A session is on this task. It moves to review on its own once the session writes the
          wrapup, and back to open when the session ends. Accept it now if you have seen enough.
        </template>
        <template v-else-if="reviewState === 'CLAIMED'">
          A session wrote the wrapup and claimed this. Accept it, or write what needs another pass
          in the feedback below.
        </template>
        <template v-else-if="task.autoAccepted">
          Rekall accepted this quick task when the session wrote the wrapup. Write a correction in
          the feedback below to review it, or mark the whole task done.
        </template>
        <template v-else>
          You accepted this. Write a correction in the feedback below to reopen it, or mark the
          whole task done.
        </template>
      </span>
      <AppButton
        v-if="reviewState === 'CLAIMED' || reviewState === 'RUNNING'"
        variant="primary"
        size="sm"
        data-testid="description-accept"
        @click="acceptDescription"
      >
        Accept
      </AppButton>
      <AppButton
        variant="quiet"
        size="sm"
        data-testid="description-open-wrapup"
        @click="store.openWrapup()"
      >
        Open the wrapup
      </AppButton>
      <AppButton
        v-if="reviewState === 'DONE' && task.status !== 'DONE'"
        variant="primary"
        size="sm"
        data-testid="description-mark-done"
        @click="markTaskDone"
      >
        Mark task done
      </AppButton>
    </div>
  </div>
</template>
