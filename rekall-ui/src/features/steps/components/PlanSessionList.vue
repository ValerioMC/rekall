<script setup lang="ts">
import { computed } from 'vue'
import PlanSessionGroup from '@/features/steps/components/PlanSessionGroup.vue'
import PlanSessionSlot from '@/features/steps/components/PlanSessionSlot.vue'
import PlanStepRow from '@/features/steps/components/PlanStepRow.vue'
import type { StepDrag } from '@/features/steps/composables/useStepDrag'
import { sessionIsMovable } from '@/features/steps/lib/step-order'
import { placesOf } from '@/features/steps/lib/step-places'
import { groupBySession, sessionStanding, type StepSession } from '@/features/steps/lib/step-sessions'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

/**
 * The checklist as the sessions it runs in, each a group of its steps. While something is dragged,
 * a slot opens in front of every session that has not started and after the last, where it runs in
 * a session of its own. The drag state belongs to the plan; every decision about a step goes up as
 * an event, so the page that holds the plan stays the one that changes it.
 */
const props = defineProps<{
  sessions: readonly StepSession[]
  /** The whole checklist, which the menus of the open steps read their choices from. */
  steps: readonly TaskStep[]
  drag: StepDrag
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
  'run-in': [step: TaskStep, sessionNumber: number]
  alone: [step: TaskStep, before: TaskStep | null]
  'nudge-session': [session: StepSession, by: number]
  session: [step: TaskStep, startsSession: boolean]
}>()

const workRemains = computed(() => props.sessions.some((session) => sessionStanding(session.steps) !== 'accepted'))
const wholeChecklist = computed(() => groupBySession(props.steps))

function openerOf(session: StepSession): TaskStep | null {
  const opener = session.steps[0]
  return opener?.state === 'OPEN' ? opener : null
}

/** Whether the held session already sits where this slot is, so dropping there would change nothing. */
function nextToHeldSession(before: TaskStep | null): boolean {
  const held = props.drag.draggedSession.value
  if (!held) return false
  const sessions = wholeChecklist.value
  const index = sessions.findIndex((session) => session.number === held.number)
  const following = sessions[index + 1]?.steps[0] ?? null
  return before?.id === held.steps[0]?.id || before?.id === following?.id
}

function offersSlotBefore(session: StepSession): boolean {
  const opener = openerOf(session)
  return props.drag.holding.value && opener !== null && !nextToHeldSession(opener)
}

const offersSlotAtEnd = computed(() => props.drag.holding.value && !nextToHeldSession(null))

const slotLabel = computed(() => {
  const session = props.drag.draggedSession.value
  return session ? `Run these ${session.steps.length} steps here` : 'Run it alone in a new session here'
})
const slotDetail = 'The sessions after it renumber'

function placeAlone(step: TaskStep, session: StepSession, where: 'before-own-session' | 'end'): void {
  const ownSession = wholeChecklist.value.find((candidate) => candidate.number === session.number)
  emit('alone', step, where === 'end' ? null : (ownSession?.steps[0] ?? null))
}
</script>

<template>
  <div data-testid="plan-steps">
    <template v-for="session in sessions" :key="session.number">
      <PlanSessionSlot
        v-if="offersSlotBefore(session)"
        :active="drag.slotActive(openerOf(session))"
        :label="slotLabel"
        :detail="slotDetail"
        idle-label="New session here"
        @hover="drag.hoverSlot(openerOf(session))"
        @drop="drag.dropOnSlot(openerOf(session))"
      />
      <PlanSessionGroup
        :session="session"
        :holds-selection="session.steps.some((step) => step.id === selectedId)"
        :work-remains="workRemains"
        :movable="sessionIsMovable(session)"
        :folded-for-drag="drag.draggedSession.value !== null"
        @join="emit('session', session.steps[0]!, false)"
        @split="(step) => emit('session', step, true)"
        @drag-start="drag.startSession(session)"
        @drag-end="drag.end()"
        @nudge="(by) => emit('nudge-session', session, by)"
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
            :places="placesOf(steps, wholeChecklist, step)"
            @select="emit('select', step)"
            @mark-forward="emit('mark-forward', step)"
            @to-draft="emit('to-draft', step)"
            @remove="emit('remove', step)"
            @nudge="(by) => emit('nudge', step, by)"
            @run-in="(sessionNumber) => emit('run-in', step, sessionNumber)"
            @alone="(where) => placeAlone(step, session, where)"
            @drag-start="drag.start(step)"
            @drag-over="(after) => drag.hover(step, after)"
            @drop="drag.drop(step)"
            @drag-end="drag.end()"
          />
        </template>
      </PlanSessionGroup>
    </template>
    <PlanSessionSlot
      v-if="offersSlotAtEnd"
      :active="drag.slotActive(null)"
      :label="slotLabel"
      :detail="slotDetail"
      idle-label="New session at the end"
      @hover="drag.hoverSlot(null)"
      @drop="drag.dropOnSlot(null)"
    />
  </div>
</template>
