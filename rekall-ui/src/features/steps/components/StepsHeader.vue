<script setup lang="ts">
import { computed } from 'vue'
import LaunchClaudeCodeButton from '@/features/claude/components/LaunchClaudeCodeButton.vue'
import OpenTerminalButton from '@/features/claude/components/OpenTerminalButton.vue'
import NotesButton from '@/features/notes/components/NotesButton.vue'
import PlanHereButton from '@/features/claude/components/PlanHereButton.vue'
import TaskTitleRow from '@/features/catalog/components/TaskTitleRow.vue'
import { identityHue } from '@/common/identity'
import type { Task, TaskStep } from '@/features/catalog/model/catalog'
import type { TaskStepId } from '@/model/branded'

/**
 * The top of the checklist: the task, the session buttons, and the ledger, one
 * segment per step so a long checklist still reads at a glance, with the tally, the list or flow
 * switch and the view toggles on the same line.
 */
const props = defineProps<{
  task: Task
  checklistSteps: readonly TaskStep[]
  draftCount: number
  currentId: TaskStepId | null
  running: boolean
}>()

const hideDone = defineModel<boolean>('hideDone', { required: true })
/** The checklist as a list, or as the flow its routes draw. */
const view = defineModel<'list' | 'flow'>('view', { required: true })

const VIEWS = ['list', 'flow'] as const

defineEmits<{
  'review-first': []
  'show-drafts': []
}>()

const hue = computed(() => identityHue(props.task.projectId))
const done = computed(() => props.checklistSteps.filter((step) => step.done).length)
const claimed = computed(() => props.checklistSteps.filter((step) => step.state === 'CLAIMED').length)
const allDone = computed(() => done.value === props.checklistSteps.length)
/** A flow begins at the first step, so one is proposed only onto a task with no steps or drafts. */
const acceptsAFlow = computed(() => props.checklistSteps.length === 0 && props.draftCount === 0)

function segmentClass(step: TaskStep): string {
  if (step.state === 'DONE') return allDone.value ? 'tube-gold tube-safe' : 'tube-gold'
  if (step.state === 'CLAIMED') return 'ledger-claimed'
  if (step.state === 'RUNNING') return 'ledger-running'
  return step.id === props.currentId ? 'tube-next' : 'bg-transparent'
}

</script>

<template>
  <div class="relative shrink-0 overflow-hidden border-b border-border">
    <div
      class="texture-grid pointer-events-none absolute inset-0 opacity-60"
      :style="{ '--texture-tint': hue.base }"
      aria-hidden="true"
    />
    <div class="pointer-events-none absolute inset-x-0 top-0 h-px" :style="{ background: hue.line }" aria-hidden="true" />

    <header class="relative flex items-start gap-4 px-5 py-3.5">
      <div class="min-w-0 flex-1">
        <p class="eyebrow flex items-center gap-1.5">
          <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
          Steps
        </p>
        <TaskTitleRow :task="task" copy-testid="copy-steps-anchor">
          <template #flags>
          <span
            v-if="running"
            class="inline-flex shrink-0 items-center gap-1.5 rounded-full bg-accent-soft px-2 py-0.5 text-micro font-semibold tracking-[0.02em] text-accent"
            data-testid="steps-running-flag"
          >
            <span class="relative grid size-2 place-items-center" aria-hidden="true">
              <span class="absolute inline-flex size-2 animate-ping rounded-full bg-accent/60" />
              <span class="relative inline-flex size-1.5 rounded-full bg-accent" />
            </span>
            running
          </span>
          </template>
        </TaskTitleRow>
      </div>

      <div class="flex shrink-0 items-center gap-1.5">
        <NotesButton :task-id="task.id" />
        <PlanHereButton :task-id="task.id" :anchor="task.anchor" :folder="task.projectRepoFolder" />
        <PlanHereButton
          v-if="acceptsAFlow"
          :task-id="task.id"
          :anchor="task.anchor"
          :folder="task.projectRepoFolder"
          mode="PLAN_FLOW"
        />
        <OpenTerminalButton :task-id="task.id" :step-id="currentId" :folder="task.projectRepoFolder" />
        <LaunchClaudeCodeButton
          :task-id="task.id"
          :anchors="task.anchor"
          :folder="task.projectRepoFolder"
          missing-hint="Set this project's folder on its page to open a session from it"
        />
      </div>
    </header>

    <div v-if="checklistSteps.length" class="relative flex items-center gap-3 px-5 pb-3" data-testid="steps-ledger">
      <p class="shrink-0 font-mono text-small font-semibold leading-none tabular-nums" data-testid="steps-count"
      ><span :class="allDone ? 'text-safe' : 'text-accent'">{{ done }}</span><span class="text-text-subtle">/{{ checklistSteps.length }}</span></p>
      <span class="trough flex h-[8px] min-w-0 flex-1 gap-[3px] p-[1.5px]" aria-hidden="true">
        <span
          v-for="step in checklistSteps"
          :key="step.id"
          class="h-full min-w-[6px] flex-1 rounded-full transition-colors duration-(--motion-slow)"
          :class="segmentClass(step)"
        />
      </span>
      <div class="flex shrink-0 gap-0.5 rounded-control bg-surface p-0.5" role="group" aria-label="Show the steps as">
        <button
          v-for="option in VIEWS"
          :key="option"
          type="button"
          class="focus-ring h-6 cursor-pointer rounded-control px-2.5 text-caption capitalize transition-colors duration-(--motion-quick)"
          :class="view === option ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]' : 'text-text-subtle hover:text-text'"
          :aria-pressed="view === option"
          :data-testid="`steps-view-${option}`"
          @click="view = option"
        >
          {{ option }}
        </button>
      </div>
      <div v-if="done > 0 || claimed > 0 || draftCount > 0" class="flex shrink-0 items-center gap-2.5 text-caption">
        <button
          v-if="claimed > 0"
          class="focus-ring rounded-key text-accent underline-offset-2 transition-colors hover:underline"
          data-testid="steps-claimed-count"
          @click="$emit('review-first')"
        >
          {{ claimed }} awaiting review
        </button>
        <button
          v-if="draftCount > 0"
          class="focus-ring inline-flex items-center gap-1 rounded-key text-text-subtle transition-colors hover:text-text"
          data-testid="steps-draft-count"
          @click="$emit('show-drafts')"
        >
          <span class="size-[7px] rounded-key border border-dashed border-current" aria-hidden="true" />
          {{ draftCount }} in draft
        </button>
        <span
          v-if="view === 'list' && (claimed > 0 || draftCount > 0) && (done > 0 || claimed > 0)"
          class="h-3 w-px bg-border-strong"
          aria-hidden="true"
        />
        <button
          v-if="view === 'list' && (done > 0 || claimed > 0)"
          class="focus-ring rounded-key text-text-subtle transition-colors hover:text-text"
          :aria-pressed="hideDone"
          data-testid="toggle-hide-done"
          @click="hideDone = !hideDone"
        >
          {{ hideDone ? 'Show done' : 'Hide done' }}
        </button>
      </div>
    </div>
  </div>
</template>
