<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import NoteKindGlyph from '@/components/ui/NoteKindGlyph.vue'
import TokenCount from '@/components/ui/TokenCount.vue'
import AppCatalogHeader from '@/features/catalog/components/AppCatalogHeader.vue'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import EpicActivityFeed from '@/features/catalog/components/epic/EpicActivityFeed.vue'
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import EpicProgressPanel from '@/features/catalog/components/epic/EpicProgressPanel.vue'
import EpicTaskList from '@/features/catalog/components/epic/EpicTaskList.vue'
import { useEpicActivityStore } from '@/features/catalog/store/epicActivity.store'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { TASK_STATUS_LABEL, isEpic } from '@/features/catalog/model/catalog'
import { epicProgress, tasksOfEpic } from '@/features/catalog/model/epic'
import { taskDraft, type RecordDraft } from '@/features/catalog/model/record-draft'
import { asTaskId, type DocumentId, type TaskId } from '@/model/branded'

/**
 * An epic's dashboard: how far the tasks it gathers are, what moved on them lately, and the
 * description and notes every one of them inherits. Those are written in the console, on the epic.
 */
const props = defineProps<{ id: string }>()

const store = useConsoleStore()
const router = useRouter()
const activityStore = useEpicActivityStore()
const tokenUsage = useTokenUsageStore()
const { run } = useAsyncAction()

const epicId = computed(() => asTaskId(props.id))
const epic = computed(() => store.tasks.find((task) => task.id === epicId.value && isEpic(task)) ?? null)
const children = computed(() => tasksOfEpic(epicId.value, store.tasks))
const childIds = computed(() => new Set(children.value.map((task) => task.id)))
const childSteps = computed(() => store.steps.filter((step) => childIds.value.has(step.taskId)))
const claimedStepTaskIds = computed(
  () => new Set(childSteps.value.filter((step) => step.state === 'CLAIMED').map((step) => step.taskId))
)
const progress = computed(() => epicProgress(children.value, claimedStepTaskIds.value))
const runningTaskIds = computed(() => new Set(store.runningEntries.map((entry) => entry.taskId)))
const notes = computed(() => store.documents.filter((document) => document.tasks.some((ref) => ref.id === epicId.value)))
const tokens = computed(() =>
  epic.value ? tokenUsage.ofProject(epic.value.projectId, [epic.value, ...children.value]) : null
)
const projectTasks = computed(() => store.tasks.filter((task) => task.projectId === epic.value?.projectId))

/** Changes whenever a task of the epic or one of their steps moves: the cue to read the feed again. */
const movement = computed(() =>
  [
    ...children.value.map((task) => `${task.id}:${task.updatedAt}:${task.reviewState}`),
    ...childSteps.value.map((step) => `${step.id}:${step.state}`)
  ].join('|')
)

watch(
  [epicId, movement],
  () => {
    if (epic.value) void run(() => activityStore.load(epicId.value))
  },
  { immediate: true }
)

const editing = ref<RecordDraft | null>(null)

function openEdit(): void {
  if (epic.value) editing.value = taskDraft(epic.value.projectId, epic.value)
}

function openAddTask(): void {
  if (epic.value) editing.value = taskDraft(epic.value.projectId, undefined, epic.value.id)
}

/** Back in the console with the task on screen, scoped to its project like the project page does. */
function openInConsole(taskId: TaskId, pane: 'note' | 'description' = 'note'): void {
  const shown = epic.value
  if (!shown) return
  const project = store.projects.find((candidate) => candidate.id === shown.projectId)
  if (project) store.setScope(project.companyId, project.id)
  store.selectTask(taskId)
  if (pane === 'description') store.openDescription()
  void router.push('/')
}

function openNote(documentId: DocumentId): void {
  openInConsole(epicId.value)
  store.selectDocument(documentId)
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col overflow-y-auto bg-canvas">
    <template v-if="!store.isLoading && !epic">
      <AppCatalogHeader title="Epic" />
      <div class="mx-auto w-full max-w-[1400px] px-6 py-6">
        <AppEmptyState title="No epic here" description="It may have been deleted, turned back into a task, or the link is stale.">
          <RouterLink to="/"><AppButton variant="secondary" size="sm">Back to the console</AppButton></RouterLink>
        </AppEmptyState>
      </div>
    </template>

    <div v-else-if="!epic" class="mx-auto flex w-full max-w-[1400px] flex-col gap-4 px-6 py-6" aria-hidden="true">
      <div class="skeleton h-40 rounded-card" />
      <div class="skeleton h-64 rounded-card" />
    </div>

    <template v-else>
      <AppCatalogHeader :title="epic.title" :parent="epic.projectTitle">
        <template #leading>
          <span class="grid size-6 place-items-center rounded-key border border-epic-line bg-epic-soft">
            <EpicGlyph :size="15" />
          </span>
        </template>
        <template #title-suffix>
          <TokenCount v-if="tokens" :totals="tokens" :hue-of="epic.projectId" />
          <AppBadge :tone="epic.status === 'DONE' ? 'safe' : epic.status === 'IN_PROGRESS' ? 'accent' : 'neutral'" dot>
            {{ TASK_STATUS_LABEL[epic.status] }}
          </AppBadge>
        </template>
        <template #actions>
          <AppButton variant="secondary" size="sm" data-testid="epic-open-console" @click="openInConsole(epic.id)">
            Open in console
          </AppButton>
          <AppButton variant="secondary" size="sm" data-testid="epic-edit" @click="openEdit">Edit</AppButton>
        </template>
      </AppCatalogHeader>

      <div
        class="mx-auto grid w-full max-w-[1400px] gap-4 px-6 py-6 lg:grid-cols-[minmax(0,1fr)_minmax(300px,380px)]"
        data-testid="epic-page"
      >
        <div class="flex min-w-0 flex-col gap-4">
          <EpicProgressPanel :progress="progress" :children="children" />
          <EpicTaskList
            :children="children"
            :steps="childSteps"
            :running-task-ids="runningTaskIds"
            @open="openInConsole"
            @add="openAddTask"
          />
          <AppCard :padded="false">
            <header class="flex items-center gap-2 border-b border-border px-5 py-3">
              <span class="eyebrow text-caption">Description</span>
              <span class="text-micro text-text-subtle">inherited by every task of the epic</span>
              <button
                class="focus-ring ml-auto rounded-control px-2 py-1 text-caption text-text-muted transition-colors hover:bg-surface-hover hover:text-text"
                @click="openInConsole(epic.id, 'description')"
              >
                Edit in console
              </button>
            </header>
            <div v-if="epic.description" class="px-5 py-4">
              <AppMarkdownEditor :model-value="epic.description" :tasks="projectTasks" readonly />
            </div>
            <p v-else class="px-5 py-6 text-small leading-relaxed text-text-subtle">
              No description yet. Write the concept the tasks share in the console, on the epic: it reaches the context of each of them.
            </p>
          </AppCard>
        </div>

        <div class="flex min-w-0 flex-col gap-4">
          <EpicActivityFeed
            :activity="activityStore.activityOf(epic.id)"
            :children="children"
            :loaded="activityStore.isLoaded(epic.id)"
            @open="openInConsole"
          />
          <AppCard :padded="false" data-testid="epic-notes">
            <header class="flex items-center gap-2 border-b border-border px-5 py-3">
              <span class="eyebrow text-caption">Notes</span>
              <span class="rounded-full bg-surface px-1.5 py-px font-mono text-micro tabular-nums text-text-subtle">{{ notes.length }}</span>
            </header>
            <p v-if="!notes.length" class="px-5 py-6 text-small leading-relaxed text-text-subtle">
              No note on the epic. A note attached to it is loaded with every one of its tasks.
            </p>
            <ul v-else class="divide-y divide-border">
              <li v-for="note in notes" :key="note.id">
                <button
                  class="focus-ring flex w-full items-center gap-2.5 px-5 py-2.5 text-left transition-colors hover:bg-surface-hover"
                  @click="openNote(note.id)"
                >
                  <NoteKindGlyph :kind="note.kind" />
                  <span class="min-w-0 flex-1 truncate text-small text-text">{{ note.title }}</span>
                </button>
              </li>
            </ul>
          </AppCard>
        </div>
      </div>
    </template>

    <Transition name="dialog">
      <RecordDialog v-if="editing" :draft="editing" @close="editing = null" />
    </Transition>
  </div>
</template>
