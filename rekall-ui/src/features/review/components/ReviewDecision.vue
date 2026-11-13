<script setup lang="ts">
import { computed, nextTick, watch } from 'vue'
import { RouterLink } from 'vue-router'
import PlanAdherencePanel from '@/features/adherence/components/PlanAdherencePanel.vue'
import StepDecisionPanel from '@/features/steps/components/StepDecisionPanel.vue'
import TaskHandedInPanel from '@/features/task/components/TaskHandedInPanel.vue'
import { useTaskPlan } from '@/features/steps/composables/useTaskPlan'
import { stepNumberOf } from '@/features/deviations/model/deviation'
import { useConsoleStore } from '@/stores/console.store'
import type { ReviewItem } from '@/features/catalog/model/review'
import type { TaskStep } from '@/features/steps/model/task-step'

/**
 * The decision on one queue item, the same panel the Plan tab shows: a claimed step to accept or
 * send back, or a task handed in whole. The task's adherence sits under it. The item leaves the
 * queue when decided, and the page moves on to the next.
 */
const props = defineProps<{ item: ReviewItem }>()

const store = useConsoleStore()
const plan = useTaskPlan()

const task = computed(() => store.tasks.find((candidate) => candidate.id === props.item.taskId) ?? null)
const taskSteps = computed(() => store.steps.filter((step) => step.taskId === props.item.taskId).sort((a, b) => a.position - b.position))
const step = computed(() => taskSteps.value.find((candidate) => candidate.id === props.item.stepId) ?? null)
const checklistLength = computed(() => taskSteps.value.filter((candidate) => candidate.state !== 'DRAFT').length)

function numberOf(other: TaskStep): number {
  return stepNumberOf(other, taskSteps.value) ?? 0
}

// The step components read the task in view from the console store, so the item's task is put
// there, and its step opened once the checklist has settled on that task.
watch(
  () => props.item.key,
  async () => {
    if (store.selectedTaskId !== props.item.taskId) store.selectTask(props.item.taskId)
    await nextTick()
    plan.editor.flush()
    if (step.value) plan.editor.open(step.value)
  },
  { immediate: true, flush: 'post' }
)
</script>

<template>
  <div v-if="task" class="flex flex-col gap-6" data-testid="review-decision">
    <nav class="flex flex-wrap items-center gap-2 text-small text-text-muted" aria-label="Where this work is">
      <span>{{ task.companyName }} · {{ task.projectTitle }}</span>
      <span class="text-text-subtle" aria-hidden="true">/</span>
      <RouterLink
        :to="{ name: 'task', params: { id: task.id, tab: 'plan' } }"
        class="focus-ring rounded-key text-accent hover:underline"
        data-testid="review-task-link"
      >
        {{ task.title }}
      </RouterLink>
    </nav>

    <StepDecisionPanel
      v-if="step"
      :step="step"
      :task="task"
      :number="numberOf(step)"
      :total="checklistLength"
      :reopen-armed="false"
      :up-next="[]"
      :number-of="numberOf"
      @accept="plan.markForward(step)"
      @send-back="plan.sendBack(step)"
    />
    <TaskHandedInPanel v-else :task="task" />

    <PlanAdherencePanel :task-id="task.id" :steps="taskSteps" />
  </div>
</template>
