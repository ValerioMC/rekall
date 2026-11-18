<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import EpicDescription from '@/features/catalog/components/epic/EpicDescription.vue'
import EpicProgressPanel from '@/features/catalog/components/epic/EpicProgressPanel.vue'
import EpicTaskList from '@/features/catalog/components/epic/EpicTaskList.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { epicProgress, tasksOfEpic } from '@/features/catalog/model/epic'
import { taskDraft, type RecordDraft } from '@/features/catalog/model/record-draft'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * An epic on the task page: the description every task inherits as the main column, and beside
 * it how far the tasks are and the tasks themselves with their status.
 */
const store = useConsoleStore()
const { selectedTask, tasks, steps, runningEntries } = storeToRefs(store)
const toast = useToastStore()
const { run } = useAsyncAction()

const children = computed(() => (selectedTask.value ? tasksOfEpic(selectedTask.value.id, tasks.value) : []))
const childSteps = computed(() => {
  const ids = new Set(children.value.map((task) => task.id))
  return steps.value.filter((step) => ids.has(step.taskId))
})
const claimedStepTaskIds = computed(
  () => new Set(childSteps.value.filter((step) => step.state === 'CLAIMED').map((step) => step.taskId))
)
const progress = computed(() => epicProgress(children.value, claimedStepTaskIds.value))
const runningTaskIds = computed(() => new Set(runningEntries.value.map((entry) => entry.taskId)))

const creating = ref<RecordDraft | null>(null)

function openCreate(): void {
  const epic = selectedTask.value
  if (epic) creating.value = taskDraft(epic.projectId, undefined, epic.id)
}

async function attach(taskId: TaskId): Promise<void> {
  const epic = selectedTask.value
  const title = tasks.value.find((task) => task.id === taskId)?.title
  if (!epic || !title) return
  await run(() => store.setTaskEpic(taskId, epic.id), `${title} is in this epic.`)
}

/** Takes the task out without asking: the toast carries an Undo that files it back. */
async function remove(task: Task): Promise<void> {
  const epic = selectedTask.value
  if (!epic) return
  const removed = await run(async () => {
    await store.setTaskEpic(task.id, null)
    return true
  })
  if (!removed) return
  toast.notify(`${task.title} is out of this epic.`, {
    label: 'Undo',
    run: () => void run(() => store.setTaskEpic(task.id, epic.id))
  })
}
</script>

<template>
  <section v-if="selectedTask" class="flex min-h-0 min-w-0 flex-1 bg-canvas" aria-label="Epic" data-testid="epic-pane">
    <EpicDescription />
    <aside class="flex min-h-0 w-96 shrink-0 flex-col border-l border-border bg-surface" aria-label="Tasks and progress">
      <EpicProgressPanel :progress="progress" :children="children" />
      <EpicTaskList
        :epic="selectedTask"
        :children="children"
        :steps="childSteps"
        :running-task-ids="runningTaskIds"
        @open="store.selectTask"
        @attach="attach"
        @create="openCreate"
        @remove="remove"
      />
    </aside>

    <Transition name="dialog">
      <RecordDialog v-if="creating" :draft="creating" @close="creating = null" />
    </Transition>
  </section>
</template>
