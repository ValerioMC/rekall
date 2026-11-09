<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import DescriptionEmptyState from '@/features/catalog/components/DescriptionEmptyState.vue'
import DescriptionReviewBar from '@/features/catalog/components/DescriptionReviewBar.vue'
import TaskBranchPanel from '@/features/worktree/components/TaskBranchPanel.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useSelectedProjectComponents } from '@/features/project/composables/useSelectedProjectComponents'
import { useSelectedProjectConnections } from '@/features/connection/composables/useSelectedProjectConnections'
import { useSelectedProjectTasks } from '@/features/catalog/composables/useSelectedProjectTasks'
import { useDescriptionDraft } from '@/features/catalog/composables/useDescriptionDraft'
import { identityHue } from '@/common/identity'
import LaunchClaudeCodeButton from '@/features/claude/components/LaunchClaudeCodeButton.vue'
import OpenTerminalButton from '@/features/claude/components/OpenTerminalButton.vue'
import LogCommitButton from '@/features/commits/components/LogCommitButton.vue'
import NotesButton from '@/features/notes/components/NotesButton.vue'
import CommitReferenceRail from '@/features/commits/components/CommitReferenceRail.vue'
import ContextSizeChip from '@/features/context/components/ContextSizeChip.vue'
import TaskTitleRow from '@/features/catalog/components/TaskTitleRow.vue'
import TaskLabelsStrip from '@/features/catalog/components/TaskLabelsStrip.vue'
import DescriptionViewTabs from '@/features/catalog/components/DescriptionViewTabs.vue'
import RevisionRunBar from '@/features/feedback/components/RevisionRunBar.vue'
import RevisionsPanel from '@/features/feedback/components/RevisionsPanel.vue'
import type { DescriptionView } from '@/features/catalog/model/description-view'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { Feedback } from '@/features/feedback/model/feedback'

const store = useConsoleStore()
const { selectedTask, selectedTaskId, selectedTaskCommitReferences, selectedTaskFeedback, waitingCount, runningPass } = storeToRefs(store)
const { run } = useAsyncAction()

const { draft, showEditor, mode, scheduleSave, beginWriting, toggleMode } = useDescriptionDraft()

const projectComponents = useSelectedProjectComponents()
const projectConnections = useSelectedProjectConnections()
const projectTasks = useSelectedProjectTasks()

const hue = computed(() => identityHue(selectedTask.value?.projectId ?? ''))

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
      <div class="relative shrink-0 overflow-hidden border-b border-border">
        <div
          class="texture-grid pointer-events-none absolute inset-0 opacity-60"
          :style="{ '--texture-tint': hue.base }"
          aria-hidden="true"
        />
        <div
          class="pointer-events-none absolute inset-x-0 top-0 h-px"
          :style="{ background: hue.line }"
          aria-hidden="true"
        />

        <header class="@container/description relative flex flex-wrap items-start gap-x-3.5 gap-y-2 px-5 py-3.5">
          <div class="min-w-0 grow basis-full @min-[49rem]/description:basis-0">
            <p class="eyebrow flex items-center gap-1.5">
              <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
              Description
            </p>
            <TaskTitleRow :task="selectedTask" copy-testid="copy-description-anchor">
              <template #flags>
              <span
                v-if="reviewState === 'RUNNING'"
                class="inline-flex shrink-0 items-center gap-1.5 rounded-full bg-accent-soft px-2 py-0.5 text-micro font-semibold tracking-[0.02em] text-accent"
                data-testid="description-running-flag"
              >
                <span class="relative grid size-2 place-items-center" aria-hidden="true">
                  <span class="absolute inline-flex size-2 animate-ping rounded-full bg-accent/60" />
                  <span class="relative inline-flex size-1.5 rounded-full bg-accent" />
                </span>
                running
              </span>
              <span
                v-else-if="reviewState === 'CLAIMED'"
                class="shrink-0 rounded-full border border-accent/40 px-2 py-0.5 text-micro font-semibold tracking-[0.02em] text-accent"
                data-testid="description-claimed-flag"
              >
                Awaiting review
              </span>
              <span
                v-else-if="reviewState === 'DONE'"
                class="shrink-0 rounded-full bg-accent px-2 py-0.5 text-micro font-semibold tracking-[0.02em] text-accent-ink"
                data-testid="description-accepted-flag"
              >
                Accepted
              </span>
              </template>
              <template #trailing>
                <ContextSizeChip :task-id="selectedTask.id" />
              </template>
            </TaskTitleRow>
            <TaskLabelsStrip :task="selectedTask" />
          </div>

          <div class="flex shrink-0 items-center gap-1.5 @max-[71rem]/description:[&_[data-label]]:hidden">
            <NotesButton :task-id="selectedTask.id" />
            <LogCommitButton
              :task-id="selectedTask.id"
              :folder="selectedTask.projectRepoFolder"
            />
            <OpenTerminalButton
              :task-id="selectedTask.id"
              :anchor="selectedTask.anchor"
              :folder="selectedTask.projectRepoFolder"
            />
            <LaunchClaudeCodeButton
              :task-id="selectedTask.id"
              :anchors="selectedTask.anchor"
              :folder="selectedTask.projectRepoFolder"
              missing-hint="Set this project's folder on its page to open a session from it"
            />
          </div>
        </header>
      </div>

      <div class="flex h-8 shrink-0 items-end border-b border-border px-5" data-testid="description-view-strip">
        <DescriptionViewTabs v-model="view" :count="selectedTaskFeedback.length" :attention="needsAttention" />
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
        v-if="waitingCount > 0"
        :task-id="selectedTask.id"
        :anchor="selectedTask.anchor"
        :folder="selectedTask.projectRepoFolder"
        :revisions-shown="view === 'revisions'"
        @show-revisions="view = 'revisions'"
      />

      <div v-if="view === 'revisions'" class="min-h-0 flex-1 overflow-y-auto p-4" data-testid="revisions-view">
        <RevisionsPanel :entries="selectedTaskFeedback" @discard="discardRevision" />
      </div>

      <DescriptionEmptyState v-else-if="!showEditor" @write="beginWriting" />

      <div v-else class="flex min-h-0 flex-1 flex-col">
        <div class="flex shrink-0 items-center gap-2 px-4 pt-3" data-testid="description-mode-row">
          <span class="h-px flex-1 bg-border" aria-hidden="true" />
          <AppModeToggle v-model="mode" testid-prefix="description-mode" />
        </div>
        <div class="min-h-0 flex-1 overflow-y-auto p-4">
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
      </div>

    </template>
  </section>
</template>
