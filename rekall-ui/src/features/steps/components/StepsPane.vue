<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import StepAddForm from '@/features/steps/components/StepAddForm.vue'
import StepChecklist from '@/features/steps/components/StepChecklist.vue'
import StepDraftShelf from '@/features/steps/components/StepDraftShelf.vue'
import StepsEmptyState from '@/features/steps/components/StepsEmptyState.vue'
import StepsHeader from '@/features/steps/components/StepsHeader.vue'
import StepFlowView from '@/features/flow/components/StepFlowView.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { provideStepEditor } from '@/features/steps/composables/useStepEditor'
import { stepIsComplete, type TaskStep } from '@/features/catalog/model/catalog'
import { STEP_DETAIL_TEMPLATE } from '@/model/templates'
import type { TaskStepId } from '@/model/branded'

const store = useConsoleStore()
const { selectedTask, selectedTaskSteps } = storeToRefs(store)
const { run } = useAsyncAction()

const editor = provideStepEditor({
  steps: () => selectedTaskSteps.value,
  save: (id, changes) => void run(() => store.saveStep(id, changes)),
  markUnsaved: () => (store.saveState = 'unsaved')
})

// Draft steps sit on a staging shelf below the checklist until they are promoted.
const checklistSteps = computed(() => selectedTaskSteps.value.filter((step) => step.state !== 'DRAFT'))
const draftSteps = computed(() => selectedTaskSteps.value.filter((step) => step.state === 'DRAFT'))
const allDone = computed(() => checklistSteps.value.every((step) => step.done))
const currentId = computed(() => checklistSteps.value.find((step) => step.state === 'OPEN')?.id ?? null)
const running = computed(() => checklistSteps.value.some((step) => step.state === 'RUNNING'))
const firstClaimed = computed(() => checklistSteps.value.find((step) => step.state === 'CLAIMED') ?? null)

const hideDone = ref(false)
const visibleSteps = computed(() =>
  hideDone.value ? checklistSteps.value.filter((step) => !step.done) : checklistSteps.value
)

// A task whose steps carry routes opens on its flow; any other on the list.
const view = ref<'list' | 'flow'>('list')
const isFlow = computed(() => checklistSteps.value.some((step) => step.links.length > 0))

const checklistEl = ref<InstanceType<typeof StepChecklist> | null>(null)
const draftsEl = ref<InstanceType<typeof StepDraftShelf> | null>(null)

/** The first step still to finish, or the first draft when every step is done. */
function openFirstUnfinished(): void {
  const next = checklistSteps.value.find((step) => !stepIsComplete(step.state)) ?? draftSteps.value[0]
  if (next) editor.open(next)
}

watch(
  () => selectedTask.value?.id ?? null,
  () => {
    editor.flush()
    hideDone.value = false
    view.value = isFlow.value ? 'flow' : 'list'
    editor.close()
    openFirstUnfinished()
  },
  { immediate: true }
)

// Something outside this pane (search, the review queue) asked for one step: open it, showing
// the done ones if that is where it is, then clear the request so it is honoured once.
watch(
  () => [store.stepToOpen, visibleSteps.value.length] as const,
  ([stepId]) => {
    if (!stepId) return
    const step = selectedTaskSteps.value.find((candidate) => candidate.id === stepId)
    if (!step) return
    if (step.done) hideDone.value = false
    editor.open(step)
    store.clearStepToOpen()
  },
  { immediate: true }
)

const reopenArmed = ref<TaskStepId | null>(null)
watch(editor.expandedId, () => (reopenArmed.value = null))

/** Ticks an open step or accepts a claimed one; the open detail moves on to the next step. */
async function markForward(step: TaskStep): Promise<void> {
  if (step.state === 'DONE') return
  const wasOpenHere = editor.expandedId.value === step.id
  editor.flush()
  await run(() => (step.state === 'CLAIMED' ? store.acceptStep(step.id) : store.toggleStep(step.id)))
  if (!wasOpenHere) return
  const next = checklistSteps.value.find((candidate) => !stepIsComplete(candidate.state))
  if (next && next.id !== step.id) editor.open(next)
  else editor.close()
}

async function sendBack(step: TaskStep): Promise<void> {
  editor.flush()
  await run(() => store.sendBackStep(step.id))
  editor.open(selectedTaskSteps.value.find((candidate) => candidate.id === step.id) ?? step)
}

async function reopen(step: TaskStep): Promise<void> {
  if (reopenArmed.value !== step.id) {
    reopenArmed.value = step.id
    return
  }
  reopenArmed.value = null
  editor.flush()
  await run(() => store.reopenStep(step.id))
  editor.open(step)
}

async function reviewFirst(): Promise<void> {
  const step = firstClaimed.value
  if (!step) return
  editor.open(step)
  await nextTick()
  checklistEl.value?.reveal(step.id)
}

async function move(step: TaskStep, by: number): Promise<void> {
  editor.flush()
  await run(() => store.moveStep(step.id, step.position + by))
}

const promoting = ref<TaskStepId | null>(null)

async function promote(step: TaskStep): Promise<void> {
  editor.flush()
  promoting.value = step.id
  try {
    await run(() => store.promoteStep(step.id))
  } finally {
    promoting.value = null
  }
}

async function returnToDraft(step: TaskStep): Promise<void> {
  editor.flush()
  await run(() => store.returnStepToDraft(step.id))
}

const deleting = ref<TaskStep | null>(null)
const deletingBlast = computed(() =>
  deleting.value?.bodyMarkdown?.trim()
    ? 'the step and the detail written under it · not recoverable'
    : 'one step · not recoverable'
)

async function remove(): Promise<void> {
  const step = deleting.value
  deleting.value = null
  if (!step) return
  if (editor.expandedId.value === step.id) editor.close()
  await run(() => store.removeStep(step.id))
}

const newTitle = ref('')
const newSkills = ref<readonly string[]>([])

/** A new step starts as a draft, opened for writing its detail. */
async function add(): Promise<void> {
  const title = newTitle.value.trim()
  if (!title || !selectedTask.value) return
  const skills = newSkills.value
  newTitle.value = ''
  newSkills.value = []
  const created = await run(async () => {
    const step = await store.addStep(selectedTask.value!.id, title, STEP_DETAIL_TEMPLATE)
    if (skills.length) await store.assignStepSkills(step.id, skills)
    return step
  })
  if (!created) return
  editor.flush()
  editor.open(created)
  editor.mode.value = 'write'
  await scrollToDrafts()
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

onUnmounted(editor.flush)
</script>

<template>
  <section class="flex min-h-0 w-full min-w-0 flex-1 flex-col bg-canvas" aria-label="Steps">
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
        <StepsEmptyState
          v-if="!checklistSteps.length && !draftSteps.length"
          :anchor="selectedTask.anchor"
          @draft-first="focusAdd"
        />

        <template v-else>
          <template v-if="checklistSteps.length">
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
              @mark-forward="markForward"
              @to-draft="returnToDraft"
              @move="move"
              @remove="(step) => (deleting = step)"
              @send-back="sendBack"
              @reopen="reopen"
            />
          </template>

          <StepDraftShelf
            v-if="draftSteps.length"
            ref="draftsEl"
            :steps="draftSteps"
            :task="selectedTask"
            :promoting="promoting"
            @promote="promote"
            @move="move"
            @remove="(step) => (deleting = step)"
          />
        </template>
      </div>

      <StepAddForm v-if="view === 'list'" v-model="newTitle" v-model:skills="newSkills" @submit="add" />
    </template>

    <Transition name="dialog">
      <AppConfirm
        v-if="deleting"
        title="Delete this step?"
        :body="`Removes &quot;${deleting.title}&quot; from this task's checklist.`"
        :blast="deletingBlast"
        confirm-label="Delete step"
        @cancel="deleting = null"
        @confirm="remove"
      />
    </Transition>
  </section>
</template>
