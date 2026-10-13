<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import DescriptionEmptyState from '@/components/console/DescriptionEmptyState.vue'
import DescriptionReviewBar from '@/components/console/DescriptionReviewBar.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useSelectedProjectComponents } from '@/composables/useSelectedProjectComponents'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import LaunchClaudeCodeButton from '@/components/claude/LaunchClaudeCodeButton.vue'
import OpenTerminalButton from '@/components/claude/OpenTerminalButton.vue'
import LogCommitButton from '@/components/console/LogCommitButton.vue'
import NotesButton from '@/components/console/NotesButton.vue'
import CommitReferenceRail from '@/components/console/CommitReferenceRail.vue'
import RevisionHistoryButton from '@/components/console/RevisionHistoryButton.vue'
import ContextSizeChip from '@/components/console/ContextSizeChip.vue'
import TaskTitleRow from '@/components/console/TaskTitleRow.vue'
import TaskLabelsStrip from '@/components/console/TaskLabelsStrip.vue'
import { TASK_DESCRIPTION_TEMPLATE, isUntouchedTemplate } from '@/model/templates'
import type { TaskId } from '@/model/branded'

const store = useConsoleStore()
const { selectedTask, selectedTaskCommitReferences } = storeToRefs(store)
const { run } = useAsyncAction()

const mode = ref<'write' | 'read'>('read')

const showEditor = ref(false)
const draft = ref('')
let saveTimer: ReturnType<typeof setTimeout> | null = null

const projectComponents = useSelectedProjectComponents()

const hue = computed(() => identityHue(selectedTask.value?.projectId ?? ''))

function flush(taskId: TaskId): void {
  if (!saveTimer) return
  clearTimeout(saveTimer)
  saveTimer = null
  void run(() => store.saveTaskDescription(taskId, draft.value))
}

function scheduleSave(): void {
  const task = selectedTask.value
  if (!task) return
  store.saveState = 'unsaved'
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => {
    saveTimer = null
    void run(() => store.saveTaskDescription(task.id, draft.value))
  }, 700)
}

watch(
  () => selectedTask.value?.id ?? null,
  (_id, previousId) => {
    if (previousId) flush(previousId)
    draft.value = selectedTask.value?.description ?? ''
    showEditor.value = draft.value.trim().length > 0
    const blank = !showEditor.value || isUntouchedTemplate(draft.value, TASK_DESCRIPTION_TEMPLATE)
    mode.value = blank ? 'write' : 'read'
  },
  { immediate: true }
)

watch(
  () => selectedTask.value?.description ?? '',
  (value) => {
    if (saveTimer || value === draft.value) return
    draft.value = value
    if (value.trim()) showEditor.value = true
  }
)

// A restore rewrote the description on the server: drop any pending save of the old buffer and
// show what was restored.
function adoptRestored(): void {
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = null
  draft.value = selectedTask.value?.description ?? ''
  showEditor.value = draft.value.trim().length > 0
  mode.value = 'read'
  store.saveState = 'saved'
}

// The task-scoped review line, shown only while the task has no checklist.
const reviewState = computed(() => {
  const task = selectedTask.value
  return task && task.reviewActive && task.stepCount === 0 ? task.reviewState : null
})

function beginWriting(): void {
  showEditor.value = true
  mode.value = 'write'
}

function toggleMode(): void {
  if (!showEditor.value) return
  mode.value = mode.value === 'write' ? 'read' : 'write'
}

defineExpose({ toggleMode })

onUnmounted(() => {
  if (selectedTask.value) flush(selectedTask.value.id)
})
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
            <RevisionHistoryButton
              :task-id="selectedTask.id"
              :task-title="selectedTask.title"
              kind="DESCRIPTION"
              @restored="adoptRestored"
            />
            <LogCommitButton
              :task-id="selectedTask.id"
              :folder="selectedTask.projectRepoFolder"
            />
            <OpenTerminalButton
              :task-id="selectedTask.id"
              :folder="selectedTask.projectRepoFolder"
            />
            <LaunchClaudeCodeButton
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
            :path-root="selectedTask?.projectRepoFolder"
            :components="projectComponents"
            @update:model-value="scheduleSave"
          />
          <AppMarkdownEditor v-else key="read" :model-value="draft" :components="projectComponents" readonly />
        </Transition>
      </div>
    </template>
  </section>
</template>
