<script setup lang="ts">
/**
 * The review line of a task with no checklist: what a session has done with it, and the answers
 * left to give. No Accept while a session is running, because nothing has been claimed to accept.
 * Revisions still waiting are run from the bar under this one, and what each pass did is read in the Revisions tab. Corrections are made by revising the description, not here.
 */
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import { groupByPass } from '@/features/feedback/lib/feedback-passes'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { Task, TaskStepState } from '@/features/catalog/model/catalog'

const props = defineProps<{ task: Task; reviewState: Exclude<TaskStepState, 'OPEN'> }>()

const store = useConsoleStore()
const { run } = useAsyncAction()
const { selectedTaskFeedback, waitingCount, runningPass } = storeToRefs(store)

const lastPass = computed(() => groupByPass(selectedTaskFeedback.value).passes.at(-1) ?? null)
const coveredEntries = computed(() => (lastPass.value?.running ? lastPass.value.entries.map((entry) => `#${entry.seq}`).join(', ') : ''))
const acceptLabel = computed(() => (waitingCount.value > 0 ? 'Accept anyway' : 'Accept'))
const acceptHint = computed(() =>
  waitingCount.value > 0 ? `${waitingCount.value} ${waitingCount.value === 1 ? 'revision has' : 'revisions have'} not been run` : 'Accept the result'
)

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
    class="min-h-0 shrink overflow-y-auto border-b border-border bg-accent-soft/40 px-5 py-2.5"
    data-testid="description-review-bar"
  >
    <div class="flex flex-wrap items-center gap-2">
      <span class="min-w-0 flex-1 text-caption leading-snug text-text-muted">
        <template v-if="reviewState === 'RUNNING'">
          <template v-if="runningPass !== null">
            Pass {{ runningPass }} is running on {{ coveredEntries }}. The result is ready to accept
            once the session writes the wrapup.
          </template>
          <template v-else>
            A session is working on this task. The result is ready to accept once it writes the
            wrapup; if the session ends first, the task goes back to open.
          </template>
        </template>
        <template v-else-if="reviewState === 'CLAIMED'">
          <template v-if="waitingCount > 0">
            A session wrote the wrapup and claimed this, and {{ waitingCount }}
            {{ waitingCount === 1 ? 'revision is' : 'revisions are' }} waiting for the next run. Run
            {{ waitingCount === 1 ? 'it' : 'them' }} before accepting.
          </template>
          <template v-else>
            A session wrote the wrapup and claimed this. Accept it, or revise the description to ask
            for another pass.
          </template>
        </template>
        <template v-else-if="task.autoAccepted">
          Rekall accepted this quick task when the session wrote the wrapup. Revise the description
          to ask for a correction, or mark the whole task done.
        </template>
        <template v-else>
          You accepted this. Revise the description to ask for a correction, or mark the whole task
          done.
        </template>
      </span>
      <AppButton
        v-if="reviewState === 'CLAIMED'"
        :variant="waitingCount > 0 ? 'quiet' : 'primary'"
        size="sm"
        :title="acceptHint"
        data-testid="description-accept"
        @click="acceptDescription"
      >
        {{ acceptLabel }}
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
