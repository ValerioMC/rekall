<script setup lang="ts">
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'
import StepDraftRow from './StepDraftRow.vue'

/** The staging shelf under the checklist: steps still being worded, not yet on the list. */
defineProps<{
  steps: readonly TaskStep[]
  task: Task
  selectedId: TaskStepId | null
  promoting: TaskStepId | null
}>()

defineEmits<{
  select: [step: TaskStep]
  promote: [step: TaskStep]
  move: [step: TaskStep, by: number]
  remove: [step: TaskStep]
}>()
</script>

<template>
  <section class="mt-5 first:mt-0" data-testid="drafts" aria-label="Drafts">
    <div class="mb-1 flex items-center gap-2 px-2">
      <span class="eyebrow">Drafts</span>
      <span class="font-mono text-micro tabular-nums text-text-subtle">{{ steps.length }}</span>
      <span class="h-px flex-1 bg-border" aria-hidden="true" />
      <span class="text-micro text-text-subtle">not on the plan yet</span>
    </div>

    <TransitionGroup tag="ul" name="step-list" class="relative min-w-0">
      <StepDraftRow
        v-for="(step, index) in steps"
        :key="step.id"
        :step="step"
        :task="task"
        :selected="selectedId === step.id"
        :is-first="index === 0"
        :is-last="index === steps.length - 1"
        :promoting="promoting === step.id"
        @select="$emit('select', step)"
        @promote="$emit('promote', step)"
        @move="(by) => $emit('move', step, by)"
        @remove="$emit('remove', step)"
      />
    </TransitionGroup>
  </section>
</template>
