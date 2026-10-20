<script setup lang="ts">
import type { Task, TaskStep } from '@/features/catalog/model/catalog'
import type { TaskStepId } from '@/model/branded'
import StepDraftRow from './StepDraftRow.vue'

/** The staging shelf under the checklist: steps still being worded, not yet on the list. */
defineProps<{
  steps: readonly TaskStep[]
  /** Every step of the task, drafts included: what a draft's routes are named against. */
  taskSteps: readonly TaskStep[]
  task: Task
  promoting: TaskStepId | null
}>()

defineEmits<{
  promote: [step: TaskStep]
  move: [step: TaskStep, by: number]
  remove: [step: TaskStep]
}>()
</script>

<template>
  <section class="mt-5 first:mt-0" data-testid="drafts" aria-label="Drafts">
    <div class="mb-2.5 flex items-center gap-2">
      <span class="eyebrow">Drafts</span>
      <span class="font-mono text-micro tabular-nums text-text-subtle">{{ steps.length }}</span>
      <span class="h-px flex-1 bg-border" aria-hidden="true" />
      <span class="text-micro text-text-subtle">still being worded, not on the list</span>
    </div>

    <TransitionGroup tag="ul" name="step-list" class="relative min-w-0 space-y-1.5">
      <StepDraftRow
        v-for="(step, index) in steps"
        :key="step.id"
        :step="step"
        :task="task"
        :task-steps="taskSteps"
        :is-first="index === 0"
        :is-last="index === steps.length - 1"
        :promoting="promoting === step.id"
        @promote="$emit('promote', step)"
        @move="(by) => $emit('move', step, by)"
        @remove="$emit('remove', step)"
      />
    </TransitionGroup>
  </section>
</template>
