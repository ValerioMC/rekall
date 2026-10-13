<script setup lang="ts">
import { ref } from 'vue'
import { useEnergyStream } from '@/composables/useEnergyStream'
import { useStepEditor } from '@/composables/useStepEditor'
import { stepIsComplete, type Task, type TaskStep } from '@/model/catalog'
import type { TaskStepId } from '@/model/branded'
import StepChecklistRow from './StepChecklistRow.vue'

/**
 * The checklist as a rail: one row per step, the connector beside each, and the lit conduit
 * that runs the length of the travelled path while a step is running.
 */
const props = defineProps<{
  steps: readonly TaskStep[]
  task: Task
  checklistLength: number
  currentId: TaskStepId | null
  allDone: boolean
  reopenArmed: TaskStepId | null
}>()

defineEmits<{
  'mark-forward': [step: TaskStep]
  'to-draft': [step: TaskStep]
  move: [step: TaskStep, by: number]
  remove: [step: TaskStep]
  'send-back': [step: TaskStep]
  reopen: [step: TaskStep]
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
    <StepChecklistRow
      v-for="(step, index) in steps"
      :key="step.id"
      :step="step"
      :task="task"
      :rail-style="railStyle(index)"
      :rail-spent="stepIsComplete(step.state)"
      :is-next="step.id === currentId"
      :all-done="allDone"
      :is-last="step.position === checklistLength - 1"
      :reopen-armed="reopenArmed === step.id"
      @mark-forward="$emit('mark-forward', step)"
      @to-draft="$emit('to-draft', step)"
      @move="(by) => $emit('move', step, by)"
      @remove="$emit('remove', step)"
      @send-back="$emit('send-back', step)"
      @reopen="$emit('reopen', step)"
    />
  </ol>
</template>
