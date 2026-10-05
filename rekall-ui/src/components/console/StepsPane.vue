<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import StepAddForm from '@/components/console/steps/StepAddForm.vue'
import StepChecklistRow from '@/components/console/steps/StepChecklistRow.vue'
import StepDraftRow from '@/components/console/steps/StepDraftRow.vue'
import StepsEmptyState from '@/components/console/steps/StepsEmptyState.vue'
import StepsHeader from '@/components/console/steps/StepsHeader.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useEnergyStream } from '@/composables/useEnergyStream'
import { provideStepEditor } from '@/composables/useStepEditor'
import { stepIsComplete, type TaskStep } from '@/model/catalog'
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

const listEl = ref<HTMLElement | null>(null)
const draftsEl = ref<HTMLElement | null>(null)
const stream = useEnergyStream(listEl, [visibleSteps, editor.expandedId, editor.mode, editor.body])

/** The rail beside row `index`: from its node down, up to its node, or both ways. */
function railStyle(index: number): Record<string, string> {
  const isFirst = index === 0
  const isLast = index === visibleSteps.value.length - 1
  const isRunning = visibleSteps.value[index]?.state === 'RUNNING'
  if (isFirst && isLast) return { display: 'none' }
  if (isRunning && !isFirst) return isLast ? { display: 'none' } : { top: '18px', bottom: '0' }
  if (isFirst) return { top: '18px', bottom: '0' }
  if (isLast) return { top: '0', height: '18px' }
  return { top: '0', bottom: '0' }
}

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
  listEl.value
    ?.querySelector<HTMLElement>(`li[data-step-id="${step.id}"]`)
    ?.scrollIntoView({ block: 'center', behavior: 'smooth' })
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

/** A new step starts as a draft, opened for writing its detail. */
async function add(): Promise<void> {
  const title = newTitle.value.trim()
  if (!title || !selectedTask.value) return
  newTitle.value = ''
  const created = await run(() => store.addStep(selectedTask.value!.id, title, STEP_DETAIL_TEMPLATE))
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
  await nextTick()
  draftsEl.value?.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
}

onUnmounted(editor.flush)
</script>

<template>
  <section class="flex min-h-0 w-full min-w-0 flex-1 flex-col bg-canvas" aria-label="Steps">
    <p v-if="!selectedTask" class="px-9 py-12 text-[13px] text-text-muted">
      Pick a task to see what is left on it.
    </p>

    <template v-else>
      <StepsHeader
        v-model:hide-done="hideDone"
        :task="selectedTask"
        :checklist-steps="checklistSteps"
        :draft-count="draftSteps.length"
        :current-id="currentId"
        :running="running"
        @review-first="reviewFirst"
        @show-drafts="scrollToDrafts"
      />

      <div class="min-h-0 min-w-0 flex-1 overflow-y-auto px-5 py-4">
        <StepsEmptyState
          v-if="!checklistSteps.length && !draftSteps.length"
          :anchor="selectedTask.anchor"
          @draft-first="focusAdd"
        />

        <template v-else>
          <template v-if="checklistSteps.length">
            <p v-if="!visibleSteps.length" class="py-6 text-[12.5px] text-text-subtle">
              Every step is done. Show them again to correct one.
            </p>

            <ol
              v-else
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
                v-for="(step, index) in visibleSteps"
                :key="step.id"
                :step="step"
                :task="selectedTask"
                :rail-style="railStyle(index)"
                :rail-spent="stepIsComplete(step.state)"
                :is-next="step.id === currentId"
                :all-done="allDone"
                :is-last="step.position === checklistSteps.length - 1"
                :reopen-armed="reopenArmed === step.id"
                @mark-forward="markForward(step)"
                @to-draft="returnToDraft(step)"
                @move="(by) => move(step, by)"
                @remove="deleting = step"
                @send-back="sendBack(step)"
                @reopen="reopen(step)"
              />
            </ol>
          </template>

          <section
            v-if="draftSteps.length"
            ref="draftsEl"
            class="mt-5 first:mt-0"
            data-testid="drafts"
            aria-label="Drafts"
          >
            <div class="mb-2.5 flex items-center gap-2">
              <span class="eyebrow">Drafts</span>
              <span class="font-mono text-[10.5px] tabular-nums text-text-subtle">{{ draftSteps.length }}</span>
              <span class="h-px flex-1 bg-border" aria-hidden="true" />
              <span class="text-[10.5px] text-text-subtle">still being worded, not on the list</span>
            </div>

            <TransitionGroup tag="ul" name="step-list" class="relative min-w-0 space-y-1.5">
              <StepDraftRow
                v-for="(step, index) in draftSteps"
                :key="step.id"
                :step="step"
                :task="selectedTask"
                :is-first="index === 0"
                :is-last="index === draftSteps.length - 1"
                :promoting="promoting === step.id"
                @promote="promote(step)"
                @move="(by) => move(step, by)"
                @remove="deleting = step"
              />
            </TransitionGroup>
          </section>
        </template>
      </div>

      <StepAddForm v-model="newTitle" @submit="add" />
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
