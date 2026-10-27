<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppCard from '@/components/ui/AppCard.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import TaskTitleRow from '@/features/catalog/components/TaskTitleRow.vue'
import TaskLabelsStrip from '@/features/catalog/components/TaskLabelsStrip.vue'
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import EpicProgressPanel from '@/features/catalog/components/epic/EpicProgressPanel.vue'
import EpicTaskList from '@/features/catalog/components/epic/EpicTaskList.vue'
import ContextSizeChip from '@/features/context/components/ContextSizeChip.vue'
import NotesButton from '@/features/notes/components/NotesButton.vue'
import { useDescriptionDraft } from '@/features/catalog/composables/useDescriptionDraft'
import { useSelectedProjectComponents } from '@/features/project/composables/useSelectedProjectComponents'
import { useSelectedProjectConnections } from '@/features/connection/composables/useSelectedProjectConnections'
import { useSelectedProjectTasks } from '@/features/catalog/composables/useSelectedProjectTasks'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { epicProgress, tasksOfEpic } from '@/features/catalog/model/epic'
import { taskDraft, type RecordDraft } from '@/features/catalog/model/record-draft'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * An epic in the console: how far its tasks are, the tasks themselves, and the description every
 * one of them inherits, written here. Shown in place of the description pane when an epic is selected.
 */
const store = useConsoleStore()
const { selectedTask, tasks, steps, runningEntries } = storeToRefs(store)
const toast = useToastStore()
const { run } = useAsyncAction()

const { draft, showEditor, mode, scheduleSave, beginWriting, toggleMode } = useDescriptionDraft()

const projectComponents = useSelectedProjectComponents()
const projectConnections = useSelectedProjectConnections()
const projectTasks = useSelectedProjectTasks()

const hue = computed(() => identityHue(selectedTask.value?.projectId ?? ''))
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

defineExpose({ toggleMode })
</script>

<template>
  <section v-if="selectedTask" class="flex min-h-0 flex-1 flex-col bg-canvas" aria-label="Epic" data-testid="epic-pane">
    <div class="relative shrink-0 overflow-hidden border-b border-border">
      <div
        class="texture-grid pointer-events-none absolute inset-0 opacity-60"
        :style="{ '--texture-tint': hue.base }"
        aria-hidden="true"
      />
      <div class="pointer-events-none absolute inset-x-0 top-0 h-px" :style="{ background: hue.line }" aria-hidden="true" />

      <header class="relative flex items-start gap-3.5 px-5 py-3.5">
        <div class="min-w-0 flex-1">
          <p class="eyebrow flex items-center gap-1.5">
            <EpicGlyph :size="11" />
            Epic
          </p>
          <TaskTitleRow :task="selectedTask" copy-testid="copy-epic-anchor">
            <template #trailing>
              <ContextSizeChip :task-id="selectedTask.id" />
            </template>
          </TaskTitleRow>
          <TaskLabelsStrip :task="selectedTask" />
        </div>
        <NotesButton :task-id="selectedTask.id" />
      </header>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto">
      <div class="mx-auto flex w-full max-w-[1100px] flex-col gap-4 px-5 py-5">
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

        <AppCard :padded="false" data-testid="epic-description">
          <header class="flex items-center gap-2 border-b border-border px-5 py-2.5">
            <span class="eyebrow text-caption">Description</span>
            <span class="min-w-0 truncate text-micro text-text-subtle">inherited by every task of the epic</span>
            <AppModeToggle v-if="showEditor" v-model="mode" class="ml-auto" testid-prefix="epic-description-mode" />
          </header>
          <div v-if="!showEditor" class="flex items-center gap-3 px-5 py-5">
            <p class="min-w-0 flex-1 text-small leading-relaxed text-text-subtle">
              No description yet. Write the concept the tasks share: it reaches the context of each of them.
            </p>
            <button
              type="button"
              class="add-row focus-ring w-auto shrink-0 px-2.5 py-1"
              data-testid="epic-write-description"
              @click="beginWriting"
            >
              <span class="text-accent" aria-hidden="true">+</span> Write it
            </button>
          </div>
          <div v-else class="p-4">
            <Transition name="fade-quick" mode="out-in">
              <AppMarkdownEditor
                v-if="mode === 'write'"
                key="write"
                v-model="draft"
                height="280px"
                grow
                :path-root="selectedTask.projectRepoFolder"
                :components="projectComponents"
                :connections="projectConnections"
                :tasks="projectTasks"
                @update:model-value="scheduleSave"
              />
              <AppMarkdownEditor
                v-else
                key="read"
                :model-value="draft"
                :components="projectComponents"
                :connections="projectConnections"
                :tasks="projectTasks"
                readonly
              />
            </Transition>
          </div>
        </AppCard>
      </div>
    </div>

    <Transition name="dialog">
      <RecordDialog v-if="creating" :draft="creating" @close="creating = null" />
    </Transition>
  </section>
</template>
