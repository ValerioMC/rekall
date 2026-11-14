<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { useRouter } from 'vue-router'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import PlanAdherencePanel from '@/features/adherence/components/PlanAdherencePanel.vue'
import StepFlowView from '@/features/flow/components/StepFlowView.vue'
import GraphAttachPanel from '@/features/graph/components/GraphAttachPanel.vue'
import PlanStepRow from '@/features/steps/components/PlanStepRow.vue'
import PlanSummary from '@/features/steps/components/PlanSummary.vue'
import StepAddForm from '@/features/steps/components/StepAddForm.vue'
import StepDecisionPanel from '@/features/steps/components/StepDecisionPanel.vue'
import StepDraftShelf from '@/features/steps/components/StepDraftShelf.vue'
import StepsEmptyState from '@/features/steps/components/StepsEmptyState.vue'
import TaskHandedInPanel from '@/features/task/components/TaskHandedInPanel.vue'
import { useTaskPlan } from '@/features/steps/composables/useTaskPlan'
import { stepNumberOf } from '@/features/deviations/model/deviation'
import { handedInWhole } from '@/features/work/lib/work-rows'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

/**
 * The Plan tab: the task's steps as a list with the tube above it, and beside it the decision
 * panel of the selected step. Drafts sit under their own heading until they are promoted.
 */
const props = defineProps<{ task: Task }>()

const MAX_UP_NEXT = 3

const plan = useTaskPlan()
const store = useConsoleStore()
const router = useRouter()
const {
  editor,
  allSteps,
  checklistSteps,
  draftSteps,
  visibleSteps,
  sessions,
  allDone,
  currentId,
  firstClaimed,
  isFlow,
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

const draftsEl = ref<InstanceType<typeof StepDraftShelf> | null>(null)
const listEl = ref<HTMLElement | null>(null)

const counts = computed(() => ({
  done: checklistSteps.value.filter((step) => step.done).length,
  claimed: checklistSteps.value.filter((step) => step.state === 'CLAIMED').length,
  total: checklistSteps.value.length
}))
const runningNumber = computed(() => {
  const step = checklistSteps.value.find((candidate) => candidate.state === 'RUNNING')
  return step ? numberOf(step) : null
})
const selectedStep = computed(
  () => checklistSteps.value.find((step) => step.id === editor.expandedId.value) ?? null
)
const handedIn = computed(() => handedInWhole(props.task))
const showsPanel = computed(() => view.value === 'list' && (selectedStep.value !== null || handedIn.value))
const upNext = computed(() =>
  checklistSteps.value
    .filter((step) => step.state === 'CLAIMED' && step.id !== selectedStep.value?.id)
    .slice(0, MAX_UP_NEXT)
)
const hasNoSteps = computed(() => checklistSteps.value.length === 0 && draftSteps.value.length === 0)

function numberOf(step: TaskStep): number {
  return stepNumberOf(step, allSteps.value) ?? 0
}

function select(step: TaskStep): void {
  editor.flush()
  editor.open(step)
}

function revealRow(id: TaskStepId, behavior: ScrollBehavior): void {
  listEl.value
    ?.querySelector<HTMLElement>(`[data-step-id="${id}"]`)
    ?.scrollIntoView({ block: 'center', behavior })
}

async function reviewNext(): Promise<void> {
  const step = firstClaimed.value
  if (!step) return
  select(step)
  await nextTick()
  revealRow(step.id, 'smooth')
}

async function showDrafts(): Promise<void> {
  view.value = 'list'
  await nextTick()
  ;(draftsEl.value?.$el as HTMLElement | undefined)?.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
}

async function add(): Promise<void> {
  const created = await plan.add()
  if (created) await showDrafts()
}

async function focusAdd(): Promise<void> {
  await nextTick()
  document.getElementById('new-step')?.focus()
}

function showAttachedGraph(): void {
  view.value = isFlow.value ? 'flow' : 'list'
}

async function openFromFlow(id: TaskStepId): Promise<void> {
  const step = allSteps.value.find((candidate) => candidate.id === id)
  if (!step) return
  view.value = 'list'
  if (step.done) hideDone.value = false
  select(step)
  await nextTick()
  revealRow(id, 'auto')
}

function openDeviation(stepNumber: number): void {
  store.openDeviation(stepNumber)
  void router.push({ name: 'task', params: { id: props.task.id, tab: 'deviations' } })
}

function canChooseSession(step: TaskStep): boolean {
  return sessions.value[0]?.steps[0]?.id !== step.id
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col" data-testid="plan-tab">
    <div class="flex min-h-0 flex-1">
      <section class="flex min-h-0 min-w-0 flex-1 flex-col" aria-label="Steps">
        <div v-if="view === 'flow' && checklistSteps.length" class="flex min-h-0 flex-1 flex-col">
          <div class="px-6 pt-4">
            <PlanSummary
              v-model:hide-done="hideDone"
              v-model:view="view"
              v-bind="counts"
              :running-number="runningNumber"
              :draft-count="draftSteps.length"
              :can-show-flow="isFlow"
              @review-next="reviewNext"
              @accept-all="plan.acceptAll"
              @show-drafts="showDrafts"
            />
          </div>
          <StepFlowView :task="task" :steps="checklistSteps" @open-detail="openFromFlow" />
        </div>

        <div v-else ref="listEl" class="min-h-0 min-w-0 flex-1 overflow-y-auto px-6 pb-16 pt-4">
          <template v-if="hasNoSteps">
            <GraphAttachPanel :task="task" :has-steps="false" @attached="showAttachedGraph" />
            <p v-if="handedIn" class="py-6 text-small text-text-muted" data-testid="plan-handed-in">
              This task was handed in whole. Review it on the right.
            </p>
            <StepsEmptyState v-else :anchor="task.anchor" @draft-first="focusAdd" />
          </template>

          <template v-else>
            <template v-if="checklistSteps.length">
              <PlanSummary
                v-model:hide-done="hideDone"
                v-model:view="view"
                v-bind="counts"
                :running-number="runningNumber"
                :draft-count="draftSteps.length"
                :can-show-flow="isFlow"
                @review-next="reviewNext"
                @accept-all="plan.acceptAll"
                @show-drafts="showDrafts"
              />
              <p v-if="!visibleSteps.length" class="py-6 text-small text-text-subtle">
                Every step is done. Show them again to correct one.
              </p>
              <ol v-else class="flex flex-col gap-0.5" data-testid="plan-steps">
                <PlanStepRow
                  v-for="step in visibleSteps"
                  :key="step.id"
                  :step="step"
                  :number="numberOf(step)"
                  :selected="selectedStep?.id === step.id"
                  :is-next="step.id === currentId"
                  :all-done="allDone"
                  :is-last="step.position === checklistSteps.length - 1"
                  :can-choose-session="canChooseSession(step)"
                  @select="select(step)"
                  @mark-forward="plan.markForward(step)"
                  @to-draft="plan.returnToDraft(step)"
                  @move="(by) => plan.move(step, by)"
                  @remove="deleting = step"
                  @session="(startsSession) => plan.chooseSession(step, startsSession)"
                />
              </ol>
            </template>

            <StepDraftShelf
              v-if="draftSteps.length"
              ref="draftsEl"
              :steps="draftSteps"
              :task-steps="allSteps"
              :task="task"
              :promoting="promoting"
              @promote="plan.promote"
              @move="plan.move"
              @remove="(step) => (deleting = step)"
              @session="plan.chooseSession"
            />

            <template v-if="checklistSteps.length">
              <PlanAdherencePanel class="mt-6" :task-id="task.id" :steps="allSteps" />
              <GraphAttachPanel v-if="canLayGraph" class="mt-4" :task="task" has-steps @attached="showAttachedGraph" />
            </template>
          </template>

          <StepAddForm
            v-model="newTitle"
            v-model:skills="newSkills"
            v-model:new-session="newSession"
            class="mt-6 rounded-card border"
            @submit="add"
          />
        </div>
      </section>

      <aside
        v-if="showsPanel"
        class="min-h-0 w-[min(46%,30rem)] shrink-0 overflow-y-auto border-l border-border bg-surface px-5 py-4"
        data-testid="plan-decision"
      >
        <StepDecisionPanel
          v-if="selectedStep"
          :step="selectedStep"
          :task="task"
          :number="numberOf(selectedStep)"
          :total="checklistSteps.length"
          :reopen-armed="reopenArmed === selectedStep.id"
          :up-next="upNext"
          :number-of="numberOf"
          @accept="plan.markForward(selectedStep)"
          @send-back="plan.sendBack(selectedStep)"
          @reopen="plan.reopen(selectedStep)"
          @mark-done="plan.markForward(selectedStep)"
          @select="select"
          @open-deviation="openDeviation"
        />
        <TaskHandedInPanel v-else :task="task" />
      </aside>
    </div>

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
  </div>
</template>
