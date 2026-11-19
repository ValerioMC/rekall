<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import StepFlowView from '@/features/flow/components/StepFlowView.vue'
import GraphAttachPanel from '@/features/graph/components/GraphAttachPanel.vue'
import PlanBar from '@/features/steps/components/PlanBar.vue'
import PlanOverview from '@/features/steps/components/PlanOverview.vue'
import PlanSessionList from '@/features/steps/components/PlanSessionList.vue'
import StepAddForm from '@/features/steps/components/StepAddForm.vue'
import StepDecisionPanel from '@/features/steps/components/StepDecisionPanel.vue'
import StepDraftShelf from '@/features/steps/components/StepDraftShelf.vue'
import StepsEmptyState from '@/features/steps/components/StepsEmptyState.vue'
import TaskHandedInPanel from '@/features/task/components/TaskHandedInPanel.vue'
import TaskWidthMenu from '@/features/task/components/TaskWidthMenu.vue'
import { columnMaxWidth } from '@/features/task/model/task-width'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { useTaskPlan } from '@/features/steps/composables/useTaskPlan'
import { isTypingTarget, neighboursOf } from '@/features/steps/lib/step-navigation'
import { stepNumberOf } from '@/features/deviations/model/deviation'
import { handedInWhole } from '@/features/work/lib/work-rows'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

/**
 * The Plan tab: one row over the plan, then the task's steps as a list of fixed width with, beside
 * it, the decision panel of the selected step, which takes the rest and can take all of it. J and K step
 * through the list. Drafts sit under their own heading until they are promoted.
 */
const props = defineProps<{ task: Task }>()

const plan = useTaskPlan()
const store = useConsoleStore()
const preferences = usePreferencesStore()
const router = useRouter()
const {
  editor,
  allSteps,
  checklistSteps,
  draftSteps,
  visibleSteps,
  visibleSessions,
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

const focused = ref(false)
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
const hasChecklist = computed(() => checklistSteps.value.length > 0)
const showsPanel = computed(() => view.value === 'list' && (hasChecklist.value || handedIn.value))
const listHidden = computed(() => focused.value && selectedStep.value !== null && view.value === 'list')
const neighbours = computed(() => neighboursOf(visibleSteps.value, selectedStep.value?.id ?? null))
const hasNoSteps = computed(() => checklistSteps.value.length === 0 && draftSteps.value.length === 0)

function numberOf(step: TaskStep): number {
  return stepNumberOf(step, allSteps.value) ?? 0
}

function select(step: TaskStep): void {
  editor.flush()
  editor.open(step)
}

function revealRow(id: TaskStepId, behavior: ScrollBehavior, block: ScrollLogicalPosition = 'center'): void {
  listEl.value
    ?.querySelector<HTMLElement>(`[data-step-id="${id}"]`)
    ?.scrollIntoView({ block, behavior })
}

async function stepTo(step: TaskStep): Promise<void> {
  select(step)
  await nextTick()
  revealRow(step.id, 'auto', 'nearest')
}

function showOverview(): void {
  editor.flush()
  editor.close()
}

function stepThroughWithKeys(event: KeyboardEvent): void {
  if (event.metaKey || event.ctrlKey || event.altKey || isTypingTarget(event.target)) return
  if (!showsPanel.value || deleting.value) return
  if (event.key === 'Escape' && selectedStep.value) {
    showOverview()
    return
  }
  const target = event.key === 'j' ? neighbours.value.next : event.key === 'k' ? neighbours.value.previous : null
  if (!target) return
  event.preventDefault()
  void stepTo(target)
}

watch(selectedStep, (step) => {
  if (!step) focused.value = false
})

onMounted(() => window.addEventListener('keydown', stepThroughWithKeys))
onBeforeUnmount(() => window.removeEventListener('keydown', stepThroughWithKeys))

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

async function nudge(step: TaskStep, by: number): Promise<void> {
  await plan.nudge(step, by)
  await nextTick()
  listEl.value?.querySelector<HTMLElement>(`[data-step-id="${step.id}"] [data-testid="plan-step-title"]`)?.focus()
}
</script>

<template>
  <div class="@container flex min-h-0 flex-1 flex-col" data-testid="plan-tab">
    <PlanBar
      v-if="hasChecklist"
      v-model:hide-done="hideDone"
      v-model:view="view"
      v-bind="counts"
      :task-id="task.id"
      :steps="allSteps"
      :running-number="runningNumber"
      :draft-count="draftSteps.length"
      :can-show-flow="isFlow"
      :list-keys="view === 'list'"
      @show-drafts="showDrafts"
      @review-next="reviewNext"
    />

    <div class="flex min-h-0 flex-1">
      <section
        class="flex min-h-0 min-w-0 flex-col"
        :class="[showsPanel ? 'w-105 shrink-0 border-r border-border' : 'flex-1', listHidden && 'hidden']"
        aria-label="Steps"
      >
        <StepFlowView v-if="view === 'flow' && hasChecklist" :task="task" :steps="checklistSteps" @open-detail="openFromFlow" />

        <div v-else ref="listEl" class="min-h-0 min-w-0 flex-1 overflow-y-auto pb-6 pt-1" :class="showsPanel ? 'px-3' : 'px-6 pt-4'">
          <template v-if="hasNoSteps">
            <GraphAttachPanel :task="task" :has-steps="false" @attached="showAttachedGraph" />
            <p v-if="handedIn" class="py-6 text-small text-text-muted" data-testid="plan-handed-in">
              This task was handed in whole. Review it on the right.
            </p>
            <StepsEmptyState v-else :anchor="task.anchor" @draft-first="focusAdd" />
          </template>

          <template v-else>
            <template v-if="hasChecklist">
              <p v-if="!visibleSteps.length" class="px-2 py-6 text-small text-text-subtle">
                Every step is done. Show them again to correct one.
              </p>
              <template v-else>
                <PlanSessionList
                  :sessions="visibleSessions"
                  :selected-id="selectedStep?.id ?? null"
                  :current-id="currentId"
                  :all-done="allDone"
                  :number-of="numberOf"
                  @select="select"
                  @mark-forward="plan.markForward"
                  @to-draft="plan.returnToDraft"
                  @remove="(step) => (deleting = step)"
                  @nudge="nudge"
                  @reorder="plan.reorder"
                  @session="(step, startsSession) => plan.chooseSession(step, startsSession)"
                />
                <p class="mt-3 flex flex-wrap items-center gap-x-2 gap-y-1 px-2 text-caption text-text-subtle" data-testid="plan-pass-legend">
                  <span class="flex gap-0.5" aria-hidden="true">
                    <span class="h-1.5 w-3 rounded-key bg-warn" />
                    <span class="h-1.5 w-3 rounded-key bg-safe" />
                    <span class="h-1.5 w-3 rounded-key bg-session" />
                  </span>
                  One bar per pass: yellow sent back, green accepted, indigo in hand.
                </p>
              </template>
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
          </template>
        </div>

        <StepAddForm
          v-if="view === 'list' || !hasChecklist"
          v-model="newTitle"
          v-model:skills="newSkills"
          v-model:new-session="newSession"
          @submit="add"
        />
      </section>

      <aside v-if="showsPanel" class="flex min-h-0 min-w-0 flex-1 flex-col bg-surface" data-testid="plan-decision">
        <StepDecisionPanel
          v-if="selectedStep"
          :step="selectedStep"
          :task="task"
          :number="numberOf(selectedStep)"
          :total="checklistSteps.length"
          :reopen-armed="reopenArmed === selectedStep.id"
          pinned
          :focused="listHidden"
          :neighbours="neighbours"
          :max-width="columnMaxWidth(preferences.taskWidth)"
          @accept="plan.markForward(selectedStep)"
          @send-back="plan.sendBack(selectedStep)"
          @reopen="plan.reopen(selectedStep)"
          @mark-done="plan.markForward(selectedStep)"
          @select="stepTo"
          @overview="showOverview"
          @toggle-focus="focused = !focused"
          @open-deviation="openDeviation"
        >
          <template #controls><TaskWidthMenu v-model="preferences.taskWidth" /></template>
        </StepDecisionPanel>
        <PlanOverview
          v-else-if="hasChecklist"
          :task="task"
          :steps="checklistSteps"
          :draft-count="draftSteps.length"
          :can-lay-graph="canLayGraph"
          :number-of="numberOf"
          @select="stepTo"
          @review-next="reviewNext"
          @accept-all="plan.acceptAll"
          @show-drafts="showDrafts"
          @open-deviation="openDeviation"
          @attached="showAttachedGraph"
        />
        <div v-else class="min-h-0 flex-1 page-scroll px-8 py-6">
          <TaskHandedInPanel :task="task" />
        </div>
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
