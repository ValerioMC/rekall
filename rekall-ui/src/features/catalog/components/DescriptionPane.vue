<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import DescriptionEmptyState from '@/features/catalog/components/DescriptionEmptyState.vue'
import DescriptionReviewBar from '@/features/catalog/components/DescriptionReviewBar.vue'
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

const store = useConsoleStore()
const { selectedTask, selectedTaskCommitReferences } = storeToRefs(store)

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

defineExpose({ toggleMode })
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col bg-canvas" aria-label="Description">
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

        <header class="relative flex items-start gap-3.5 px-5 py-3.5">
          <div class="min-w-0 flex-1">
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

          <div class="flex shrink-0 items-center gap-1.5">
            <NotesButton :task-id="selectedTask.id" />
            <LogCommitButton
              :task-id="selectedTask.id"
              :folder="selectedTask.projectRepoFolder"
            />
            <OpenTerminalButton
              :task-id="selectedTask.id"
              :folder="selectedTask.projectRepoFolder"
            />
            <LaunchClaudeCodeButton
              :task-id="selectedTask.id"
              :anchors="selectedTask.anchor"
              :folder="selectedTask.projectRepoFolder"
              missing-hint="Set this project's folder on its page to open a session from it"
            />
          </div>

          <AppModeToggle v-if="showEditor" v-model="mode" />
        </header>
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

      <DescriptionEmptyState v-if="!showEditor" @write="beginWriting" />

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
