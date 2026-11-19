<script setup lang="ts">
import { computed } from 'vue'
import PlanSessionGroup from '@/features/steps/components/PlanSessionGroup.vue'
import PlanStepRow from '@/features/steps/components/PlanStepRow.vue'
import { useStepDrag } from '@/features/steps/composables/useStepDrag'
import { sessionStanding, type StepSession } from '@/features/steps/lib/step-sessions'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

/**
 * The checklist as the sessions it runs in, each a group of its steps. It owns what dragging a
 * step needs and hands every decision about a step up as an event, so the page that holds the
 * plan stays the one that changes it.
 */
const props = defineProps<{
  sessions: readonly StepSession[]
  selectedId: TaskStepId | null
  currentId: TaskStepId | null
  allDone: boolean
  numberOf: (step: TaskStep) => number
}>()

const emit = defineEmits<{
  select: [step: TaskStep]
  'mark-forward': [step: TaskStep]
  'to-draft': [step: TaskStep]
  remove: [step: TaskStep]
  nudge: [step: TaskStep, by: number]
  reorder: [step: TaskStep, target: TaskStep, after: boolean]
  session: [step: TaskStep, startsSession: boolean]
}>()

const workRemains = computed(() => props.sessions.some((session) => sessionStanding(session.steps) !== 'accepted'))

const drag = useStepDrag((moving, target, after) => emit('reorder', moving, target, after))
</script>

<template>
  <div data-testid="plan-steps">
    <PlanSessionGroup
      v-for="session in sessions"
      :key="session.number"
      :session="session"
      :holds-selection="session.steps.some((step) => step.id === selectedId)"
      :work-remains="workRemains"
      @join="emit('session', session.steps[0]!, false)"
      @split="(step) => emit('session', step, true)"
    >
      <template #step="{ step, endsSession }">
        <PlanStepRow
          :step="step"
          :number="numberOf(step)"
          :selected="selectedId === step.id"
          :is-next="step.id === currentId"
          :all-done="allDone"
          :ends-session="endsSession"
          :droppable="drag.accepts(step)"
          :drop-edge="drag.edgeOf(step)"
          :dragging="drag.dragged.value?.id === step.id"
          @select="emit('select', step)"
          @mark-forward="emit('mark-forward', step)"
          @to-draft="emit('to-draft', step)"
          @remove="emit('remove', step)"
          @nudge="(by) => emit('nudge', step, by)"
          @drag-start="drag.start(step)"
          @drag-over="(after) => drag.hover(step, after)"
          @drop="drag.drop(step)"
          @drag-end="drag.end()"
        />
      </template>
    </PlanSessionGroup>
  </div>
</template>
