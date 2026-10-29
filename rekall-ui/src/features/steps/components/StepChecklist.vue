<script setup lang="ts">
import { ref } from 'vue'
import { useEnergyStream } from '@/features/tokens/composables/useEnergyStream'
import { useStepEditor } from '@/features/steps/composables/useStepEditor'
import { stepIsComplete, type Task, type TaskStep } from '@/features/catalog/model/catalog'
import type { TaskStepId } from '@/model/branded'
import type { StepSession } from '@/features/steps/lib/step-sessions'
import StepChecklistRow from './StepChecklistRow.vue'

/**
 * The checklist as a rail: one row per step, the connector beside each, and the lit conduit
 * that runs the length of the travelled path while a step is running. When the steps run in more
 * than one Claude session, a header opens each session's group.
 */
const props = defineProps<{
  steps: readonly TaskStep[]
  task: Task
  checklistLength: number
  currentId: TaskStepId | null
  allDone: boolean
  reopenArmed: TaskStepId | null
  /** The whole checklist cut into sessions, not only the rows shown. */
  sessions: readonly StepSession[]
}>()

defineEmits<{
  'mark-forward': [step: TaskStep]
  'to-draft': [step: TaskStep]
  move: [step: TaskStep, by: number]
  remove: [step: TaskStep]
  'send-back': [step: TaskStep]
  reopen: [step: TaskStep]
  session: [step: TaskStep, startsSession: boolean]
}>()

const editor = useStepEditor()
const listEl = ref<HTMLElement | null>(null)
const stream = useEnergyStream(listEl, [() => props.steps, editor.expandedId, editor.mode, editor.body])

/** The rail beside row `index`: from its node down, up to its node, or both ways. */
function railStyle(index: number): Record<string, string> {
  const isFirst = index === 0
  const isLast = index === props.steps.length - 1
  const isRunning = props.steps[index]?.state === 'RUNNING'
  if (isFirst && isLast) return { display: 'none' }
  if (isRunning && !isFirst) return isLast ? { display: 'none' } : { top: '18px', bottom: '0' }
  if (isFirst) return { top: '18px', bottom: '0' }
  if (isLast) return { top: '0', height: '18px' }
  return { top: '0', bottom: '0' }
}

/** The session a step is the first of, when there is more than one session to tell apart. */
function sessionOpenedBy(step: TaskStep): StepSession | undefined {
  if (props.sessions.length < 2) return undefined
  return props.sessions.find((session) => session.steps[0]?.id === step.id)
}

/** Whether the step may open or join a session: the first of the checklist always opens the first. */
function canChooseSession(step: TaskStep): boolean {
  return props.sessions[0]?.steps[0]?.id !== step.id
}

/** Brings one step's row to the middle of the pane. */
function reveal(id: TaskStepId, behavior: ScrollBehavior = 'smooth'): void {
  listEl.value
    ?.querySelector<HTMLElement>(`li[data-step-id="${id}"]`)
    ?.scrollIntoView({ block: 'center', behavior })
}

defineExpose({ reveal })
</script>

<template>
  <ol
    ref="listEl"
    class="relative min-w-0"
    :style="allDone ? { '--rail-tint': 'var(--color-safe)' } : undefined"
  >
    <span
      v-if="stream"
      class="energy-stream"
      :style="{ top: `${stream.top}px`, height: `${stream.height}px` }"
      data-testid="energy-stream"
      aria-hidden="true"
    />
    <template v-for="(step, index) in steps" :key="step.id">
      <li
        v-if="sessionOpenedBy(step)"
        class="flex items-center gap-2 pb-1.5 pl-9 pt-2 first:pt-0"
        data-testid="session-header"
      >
        <span class="eyebrow">Session {{ sessionOpenedBy(step)!.number }}</span>
        <span class="font-mono text-micro tabular-nums text-text-subtle">
          {{ sessionOpenedBy(step)!.steps.length }}
          {{ sessionOpenedBy(step)!.steps.length === 1 ? 'step' : 'steps' }}
        </span>
        <span class="h-px flex-1 bg-border" aria-hidden="true" />
        <span class="text-micro text-text-subtle">one Claude session</span>
      </li>
      <StepChecklistRow
        :step="step"
        :task="task"
        :rail-style="railStyle(index)"
        :rail-spent="stepIsComplete(step.state)"
        :is-next="step.id === currentId"
        :all-done="allDone"
        :is-last="step.position === checklistLength - 1"
        :reopen-armed="reopenArmed === step.id"
        :can-choose-session="canChooseSession(step)"
        @mark-forward="$emit('mark-forward', step)"
        @to-draft="$emit('to-draft', step)"
        @move="(by) => $emit('move', step, by)"
        @remove="$emit('remove', step)"
        @send-back="$emit('send-back', step)"
        @reopen="$emit('reopen', step)"
        @session="(startsSession) => $emit('session', step, startsSession)"
      />
    </template>
  </ol>
</template>
