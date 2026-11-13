<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import DescriptionReviewBar from '@/features/catalog/components/DescriptionReviewBar.vue'
import TaskBranchPanel from '@/features/worktree/components/TaskBranchPanel.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useSelectedProjectComponents } from '@/features/project/composables/useSelectedProjectComponents'
import { useSelectedProjectConnections } from '@/features/connection/composables/useSelectedProjectConnections'
import { useSelectedProjectTasks } from '@/features/catalog/composables/useSelectedProjectTasks'
import { useDescriptionDraft } from '@/features/catalog/composables/useDescriptionDraft'
import CommitReferenceRail from '@/features/commits/components/CommitReferenceRail.vue'
import ContextSizeChip from '@/features/context/components/ContextSizeChip.vue'
import TaskTokenChip from '@/features/tokens/components/TaskTokenChip.vue'
import TaskLabelsStrip from '@/features/catalog/components/TaskLabelsStrip.vue'
import DescriptionRewriteButton from '@/features/catalog/components/DescriptionRewriteButton.vue'
import DescriptionViewTabs from '@/features/catalog/components/DescriptionViewTabs.vue'
import RevisionRunBar from '@/features/feedback/components/RevisionRunBar.vue'
import RevisionsPanel from '@/features/feedback/components/RevisionsPanel.vue'
import type { DescriptionView } from '@/features/catalog/model/description-view'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { Feedback } from '@/features/feedback/model/feedback'

const store = useConsoleStore()
const { selectedTask, selectedTaskId, selectedTaskCommitReferences, selectedTaskFeedback, waitingCount, runningPass } = storeToRefs(store)
const { run } = useAsyncAction()

const { draft, mode, scheduleSave, toggleMode } = useDescriptionDraft({ openEmptyInEditor: true })

const projectComponents = useSelectedProjectComponents()
const projectConnections = useSelectedProjectConnections()
const projectTasks = useSelectedProjectTasks()

// The task-scoped review line, shown only while the task has no checklist.
const reviewState = computed(() => {
  const task = selectedTask.value
  return task && task.reviewActive && task.stepCount === 0 ? task.reviewState : null
})

const needsAttention = computed(() => waitingCount.value > 0 || runningPass.value !== null)
const view = ref<DescriptionView>('description')

watch(
  selectedTaskId,
  (id) => {
    if (id !== null) void store.loadFeedback(id).catch(() => undefined)
  },
  { immediate: true }
)

watch(selectedTaskId, () => {
  view.value = 'description'
})

function discardRevision(entry: Feedback): void {
  void run(() => store.discardRevision(entry))
}

defineExpose({ toggleMode })
</script>

<template>
  <section class="flex min-h-0 min-w-0 flex-1 flex-col bg-canvas" aria-label="Description">
    <p v-if="!selectedTask" class="px-9 py-12 text-body text-text-muted">
      Pick a task to see what it is.
    </p>

    <template v-else>
      <div class="flex h-9 shrink-0 items-end gap-3 border-b border-border px-5" data-testid="description-view-strip">
        <DescriptionViewTabs v-model="view" :count="selectedTaskFeedback.length" :attention="needsAttention" :running="runningPass !== null" />
        <span class="flex-1" />
        <div class="flex items-center gap-1.5 pb-1" data-testid="description-actions">
          <ContextSizeChip :task-id="selectedTask.id" />
          <DescriptionRewriteButton
            :key="selectedTask.id"
            :task-id="selectedTask.id"
            :anchor="selectedTask.anchor"
            :folder="selectedTask.projectRepoFolder"
            :class="{ invisible: view !== 'description' }"
            :inert="view !== 'description'"
          />
          <AppModeToggle
            v-model="mode"
            testid-prefix="description-mode"
            :class="{ invisible: view !== 'description' }"
            :inert="view !== 'description'"
          />
        </div>
      </div>

      <div class="flex min-w-0 shrink-0 flex-wrap items-center gap-x-3 gap-y-1.5 border-b border-border px-5 py-2" data-testid="description-properties">
        <TaskLabelsStrip :task="selectedTask" class="min-w-0" />
        <span class="flex-1" />
        <TaskTokenChip :task="selectedTask" />
      </div>

      <div
        v-if="selectedTaskCommitReferences.length"
        class="shrink-0 border-b border-border px-5 py-1.5"
        data-testid="description-commits"
      >
        <CommitReferenceRail :key="selectedTask.id" :references="selectedTaskCommitReferences" />
      </div>

      <DescriptionReviewBar
        v-if="reviewState === 'RUNNING' || reviewState === 'CLAIMED' || reviewState === 'DONE'"
        :task="selectedTask"
        :review-state="reviewState"
      />

      <TaskBranchPanel
        v-if="selectedTask.stepCount === 0"
        class="mx-4 my-2 shrink-0"
        :task-id="selectedTask.id"
        :signature="`${reviewState}:${selectedTask.updatedAt}`"
      />

      <RevisionRunBar
        :key="selectedTask.id"
        :task-id="selectedTask.id"
        :anchor="selectedTask.anchor"
        :folder="selectedTask.projectRepoFolder"
        :revisions-shown="view === 'revisions'"
        @show-revisions="view = 'revisions'"
      />

      <div v-if="view === 'revisions'" class="min-h-0 flex-1 overflow-y-auto p-4" data-testid="revisions-view">
        <RevisionsPanel :entries="selectedTaskFeedback" @discard="discardRevision" />
      </div>

      <div v-else class="min-h-0 flex-1 overflow-y-auto p-4">
        <Transition name="fade-quick" mode="out-in">
          <AppMarkdownEditor
            v-if="mode === 'write'"
            key="write"
            v-model="draft"
            height="100%"
            :page-fullscreen="false"
            :path-root="selectedTask?.projectRepoFolder"
            :components="projectComponents"
            :connections="projectConnections"
            :tasks="projectTasks"
            @update:model-value="scheduleSave"
          />
          <AppMarkdownEditor v-else key="read" :model-value="draft" :components="projectComponents" :connections="projectConnections" :tasks="projectTasks" readonly />
        </Transition>
      </div>

    </template>
  </section>
</template>
