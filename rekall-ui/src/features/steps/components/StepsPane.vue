<script setup lang="ts">
import { nextTick, ref } from 'vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import StepAddForm from '@/features/steps/components/StepAddForm.vue'
import StepChecklist from '@/features/steps/components/StepChecklist.vue'
import StepDraftShelf from '@/features/steps/components/StepDraftShelf.vue'
import StepsEmptyState from '@/features/steps/components/StepsEmptyState.vue'
import StepsHeader from '@/features/steps/components/StepsHeader.vue'
import StepFlowView from '@/features/flow/components/StepFlowView.vue'
import PlanAdherencePanel from '@/features/adherence/components/PlanAdherencePanel.vue'
import TaskBranchPanel from '@/features/worktree/components/TaskBranchPanel.vue'
import GraphAttachPanel from '@/features/graph/components/GraphAttachPanel.vue'
import { useTaskPlan } from '@/features/steps/composables/useTaskPlan'
import type { TaskStepId } from '@/model/branded'

const plan = useTaskPlan()
const {
  task: selectedTask,
  editor,
  allSteps: selectedTaskSteps,
  checklistSteps,
  draftSteps,
  visibleSteps,
  sessions,
  allDone,
  currentId,
  running,
  firstClaimed,
  canLayGraph,
  hideDone,
  view,
  reopenArmed,
  promoting,
  deleting,
  deletingBlast,
  newTitle,
  newSkills,
  newSession
} = plan

const checklistEl = ref<InstanceType<typeof StepChecklist> | null>(null)
const draftsEl = ref<InstanceType<typeof StepDraftShelf> | null>(null)

async function reviewFirst(): Promise<void> {
  const step = firstClaimed.value
  if (!step) return
  editor.open(step)
  await nextTick()
  checklistEl.value?.reveal(step.id)
}

async function add(): Promise<void> {
  const created = await plan.add()
  if (created) await scrollToDrafts()
}

async function focusAdd(): Promise<void> {
  await nextTick()
  document.getElementById('new-step')?.focus()
}

async function scrollToDrafts(): Promise<void> {
  view.value = 'list'
  await nextTick()
  ;(draftsEl.value?.$el as HTMLElement | undefined)?.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
}

/** A graph just laid on the task is shown as the flow it is; one without routes stays a list. */
function showAttachedGraph(): void {
  view.value = plan.isFlow.value ? 'flow' : 'list'
}

/** The flow's "Detail" opens that step on the list, where its detail is written. */
async function openFromFlow(id: TaskStepId): Promise<void> {
  const step = selectedTaskSteps.value.find((candidate) => candidate.id === id)
  if (!step) return
  view.value = 'list'
  if (step.done) hideDone.value = false
  editor.open(step)
  await nextTick()
  checklistEl.value?.reveal(id, 'auto')
}
</script>

<template>
  <section class="steps-pane flex min-h-0 w-full min-w-0 flex-1 flex-col bg-canvas" aria-label="Steps">
    <p v-if="!selectedTask" class="px-9 py-12 text-body text-text-muted">
      Pick a task to see what is left on it.
    </p>

    <template v-else>
      <StepsHeader
        v-model:hide-done="hideDone"
        v-model:view="view"
        :task="selectedTask"
        :checklist-steps="checklistSteps"
        :draft-count="draftSteps.length"
        :current-id="currentId"
        :running="running"
        @review-first="reviewFirst"
        @show-drafts="scrollToDrafts"
      />

      <StepFlowView
        v-if="view === 'flow' && checklistSteps.length"
        :task="selectedTask"
        :steps="checklistSteps"
        @open-detail="openFromFlow"
      />

      <div v-else class="min-h-0 min-w-0 flex-1 overflow-y-auto px-5 py-4">
        <template v-if="!checklistSteps.length && !draftSteps.length">
          <GraphAttachPanel :task="selectedTask" :has-steps="false" @attached="showAttachedGraph" />
          <StepsEmptyState :anchor="selectedTask.anchor" @draft-first="focusAdd" />
        </template>

        <template v-else>
          <template v-if="checklistSteps.length">
            <PlanAdherencePanel class="mb-4" :task-id="selectedTask.id" :steps="selectedTaskSteps" />
            <TaskBranchPanel class="mb-4" :task-id="selectedTask.id" :signature="`${selectedTask.stepsDone}:${selectedTask.updatedAt}`" />
            <p v-if="!visibleSteps.length" class="py-6 text-small text-text-subtle">
              Every step is done. Show them again to correct one.
            </p>

            <StepChecklist
              v-else
              ref="checklistEl"
              :steps="visibleSteps"
              :task="selectedTask"
              :checklist-length="checklistSteps.length"
              :current-id="currentId"
              :all-done="allDone"
              :reopen-armed="reopenArmed"
              :sessions="sessions"
              @session="plan.chooseSession"
              @mark-forward="plan.markForward"
              @to-draft="plan.returnToDraft"
              @move="plan.move"
              @remove="(step) => (deleting = step)"
              @send-back="plan.sendBack"
              @reopen="plan.reopen"
            />

            <GraphAttachPanel v-if="canLayGraph" class="mt-6" :task="selectedTask" has-steps @attached="showAttachedGraph" />
          </template>

          <StepDraftShelf
            v-if="draftSteps.length"
            ref="draftsEl"
            :steps="draftSteps"
            :task-steps="selectedTaskSteps"
            :task="selectedTask"
            :promoting="promoting"
            @promote="plan.promote"
            @move="plan.move"
            @remove="(step) => (deleting = step)"
            @session="plan.chooseSession"
          />
        </template>
      </div>

      <StepAddForm
        v-if="view === 'list'"
        v-model="newTitle"
        v-model:skills="newSkills"
        v-model:new-session="newSession"
        @submit="add"
      />
    </template>

    <Transition name="dialog">
      <AppConfirm
        v-if="deleting"
        title="Delete this step?"
        :body="`Removes &quot;${deleting.title}&quot; from this task's checklist.`"
        :blast="deletingBlast"
        confirm-label="Delete step"
        @cancel="deleting = null"
        @confirm="plan.removeDeleting"
      />
    </Transition>
  </section>
</template>
