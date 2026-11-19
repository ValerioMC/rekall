import { computed, onUnmounted, ref, watch, type ComputedRef, type Ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useCheckStream } from '@/features/check/composables/useCheckStream'
import { useChecksStore } from '@/features/check/store/checks.store'
import { provideStepEditor, type StepEditor } from '@/features/steps/composables/useStepEditor'
import { openerHandover, planStepMove, type SessionChange } from '@/features/steps/lib/step-order'
import { groupBySession, visibleSessions as sessionsShown, type StepSession } from '@/features/steps/lib/step-sessions'
import { stepIsComplete, type TaskStep } from '@/features/steps/model/task-step'
import type { Task } from '@/features/catalog/model/catalog'
import type { SkillAssignment } from '@/features/skills/model/skill-assignment'
import { useConsoleStore } from '@/stores/console.store'
import { useSendBackGuardStore } from '@/features/review/store/sendBackGuard.store'
import { useToastStore } from '@/stores/toast.store'
import type { TaskStepId } from '@/model/branded'

export interface TaskPlan {
  readonly task: ComputedRef<Task | null>
  readonly editor: StepEditor
  readonly allSteps: ComputedRef<readonly TaskStep[]>
  readonly checklistSteps: ComputedRef<readonly TaskStep[]>
  readonly draftSteps: ComputedRef<readonly TaskStep[]>
  readonly visibleSteps: ComputedRef<readonly TaskStep[]>
  readonly sessions: ComputedRef<readonly StepSession[]>
  /** The sessions as the list shows them, with the done steps left out while they are hidden. */
  readonly visibleSessions: ComputedRef<readonly StepSession[]>
  readonly allDone: ComputedRef<boolean>
  readonly currentId: ComputedRef<TaskStepId | null>
  readonly running: ComputedRef<boolean>
  readonly firstClaimed: ComputedRef<TaskStep | null>
  readonly isFlow: ComputedRef<boolean>
  readonly canLayGraph: ComputedRef<boolean>
  readonly hideDone: Ref<boolean>
  readonly view: Ref<'list' | 'flow'>
  readonly reopenArmed: Ref<TaskStepId | null>
  /** The draft just added, whose detail takes the caret as its pane opens; cleared once another step is opened. */
  readonly justAddedId: Ref<TaskStepId | null>
  readonly promoting: Ref<TaskStepId | null>
  readonly deleting: Ref<TaskStep | null>
  readonly deletingBlast: ComputedRef<string>
  readonly newTitle: Ref<string>
  readonly newSkills: Ref<readonly SkillAssignment[]>
  readonly newSession: Ref<boolean>
  markForward: (step: TaskStep) => Promise<void>
  acceptAll: () => Promise<void>
  chooseSession: (step: TaskStep, startsSession: boolean) => Promise<void>
  sendBack: (step: TaskStep) => Promise<void>
  reopen: (step: TaskStep) => Promise<void>
  move: (step: TaskStep, by: number) => Promise<void>
  reorder: (step: TaskStep, target: TaskStep, after: boolean) => Promise<void>
  nudge: (step: TaskStep, by: number) => Promise<void>
  promote: (step: TaskStep) => Promise<void>
  returnToDraft: (step: TaskStep) => Promise<void>
  removeDeleting: () => Promise<void>
  add: () => Promise<TaskStep | null>
}

/**
 * Every operation on the task in view's checklist, shared by the screens that show it. It owns the
 * open step's editor, so it must be called from the component that renders the steps.
 */
export function useTaskPlan(): TaskPlan {
  const store = useConsoleStore()
  const { selectedTask, selectedTaskSteps } = storeToRefs(store)
  const { run } = useAsyncAction()
  const checks = useChecksStore()
  const toast = useToastStore()
  const sendBackGuard = useSendBackGuardStore()

  const editor = provideStepEditor({
    steps: () => selectedTaskSteps.value,
    save: (id, changes) => void run(() => store.saveStep(id, changes)),
    markUnsaved: () => (store.saveState = 'unsaved')
  })

  const checklistSteps = computed(() => selectedTaskSteps.value.filter((step) => step.state !== 'DRAFT'))
  const draftSteps = computed(() => selectedTaskSteps.value.filter((step) => step.state === 'DRAFT'))
  const sessions = computed(() => groupBySession(checklistSteps.value))
  const allDone = computed(() => checklistSteps.value.every((step) => step.done))
  const currentId = computed(() => checklistSteps.value.find((step) => step.state === 'OPEN')?.id ?? null)
  const running = computed(() => checklistSteps.value.some((step) => step.state === 'RUNNING'))
  const firstClaimed = computed(() => checklistSteps.value.find((step) => step.state === 'CLAIMED') ?? null)

  const hideDone = ref(false)
  const visibleSteps = computed(() =>
    hideDone.value ? checklistSteps.value.filter((step) => !step.done) : checklistSteps.value
  )
  const visibleSessions = computed(() => sessionsShown(sessions.value, hideDone.value))

  const view = ref<'list' | 'flow'>('list')
  const isFlow = computed(() => checklistSteps.value.some((step) => step.links.length > 0))
  const canLayGraph = computed(() => !draftSteps.value.length && selectedTask.value?.graphLaidAt === null)

  function stepAwaitingDecision(excluding: TaskStepId | null): TaskStep | undefined {
    return checklistSteps.value.find((step) => step.state === 'CLAIMED' && step.id !== excluding)
  }

  function nextUnfinishedStep(excluding: TaskStepId | null): TaskStep | undefined {
    return checklistSteps.value.find((step) => !stepIsComplete(step.state) && step.id !== excluding)
  }

  watch(
    () => selectedTask.value?.id ?? null,
    () => {
      editor.flush()
      hideDone.value = false
      view.value = isFlow.value ? 'flow' : 'list'
      editor.close()
    },
    { immediate: true }
  )

  watch(
    () => selectedTask.value?.id ?? null,
    (taskId) => {
      if (taskId) void run(() => checks.loadResults(taskId))
    },
    { immediate: true }
  )
  useCheckStream({
    onChecks: (event) => void checks.applyProjectEvent(event.projectId),
    onResults: (event) => void checks.applyResultEvent(event.taskId)
  })

  // Something outside the plan (search, the review queue) asked for one step: open it, showing the
  // done ones if that is where it is, then clear the request so it is honoured once.
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

  const justAddedId = ref<TaskStepId | null>(null)
  watch(editor.expandedId, (openId) => {
    if (openId !== justAddedId.value) justAddedId.value = null
  })

  function offerToReopenAccepted(step: TaskStep): void {
    toast.notify(`Accepted ${step.title}`, {
      label: 'Reopen',
      run: () => void run(() => store.reopenStep(step.id))
    })
  }

  function offerToUndoTick(step: TaskStep): void {
    toast.notify(`Marked ${step.title} done`, {
      label: 'Undo',
      run: () => void run(() => store.reopenStep(step.id))
    })
  }

  /** Ticks an open step or accepts a claimed one; the open detail moves on to the next claim to review, else the next step. */
  async function markForward(step: TaskStep): Promise<void> {
    if (step.state === 'DONE') return
    const wasOpenHere = editor.expandedId.value === step.id
    const wasClaimed = step.state === 'CLAIMED'
    editor.flush()
    const outcome = await run(() => (wasClaimed ? store.acceptStep(step.id) : store.toggleStep(step.id)))
    if (outcome !== null) (wasClaimed ? offerToReopenAccepted : offerToUndoTick)(step)
    if (!wasOpenHere) return
    const next = stepAwaitingDecision(step.id) ?? nextUnfinishedStep(step.id)
    if (next) editor.open(next)
    else editor.close()
  }

  /** Accepts every claimed step at once, once the claimed steps are all that is left. */
  async function acceptAll(): Promise<void> {
    const task = selectedTask.value
    if (!task) return
    editor.flush()
    await run(() => store.acceptTask(task.id))
  }

  async function chooseSession(step: TaskStep, startsSession: boolean): Promise<void> {
    editor.flush()
    await run(() => store.beginStepSession(step.id, startsSession))
  }

  async function sendBack(step: TaskStep): Promise<void> {
    await sendBackGuard.guard(`“${step.title}”`, async () => {
      editor.flush()
      await run(() => store.sendBackStep(step.id))
      editor.open(selectedTaskSteps.value.find((candidate) => candidate.id === step.id) ?? step)
    })
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

  async function move(step: TaskStep, by: number): Promise<void> {
    editor.flush()
    await run(() => store.moveStep(step.id, step.position + by))
  }

  async function applySessionChanges(changes: readonly SessionChange[]): Promise<void> {
    for (const change of changes) await store.beginStepSession(change.step.id, change.startsSession)
  }

  /** Puts an open step before or after another, regrouping the sessions around it as the move needs. */
  async function reorder(step: TaskStep, target: TaskStep, after: boolean): Promise<void> {
    const plan = planStepMove(checklistSteps.value, step, target, after)
    if (!plan) return
    editor.flush()
    await run(async () => {
      await store.moveStep(step.id, plan.position)
      await applySessionChanges(plan.sessionChanges)
    })
  }

  /** The keyboard's move: one place up or down, never past a step that has been started. */
  async function nudge(step: TaskStep, by: number): Promise<void> {
    const neighbour = checklistSteps.value[checklistSteps.value.indexOf(step) + by]
    if (step.state !== 'OPEN' || neighbour?.state !== 'OPEN') return
    await reorder(step, neighbour, by > 0)
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

  /** Takes the step off the plan, and offers to put it back where it was, sessions included. */
  async function returnToDraft(step: TaskStep): Promise<void> {
    const { position } = step
    const handover = openerHandover(checklistSteps.value, step)
    editor.flush()
    const outcome = await run(async () => {
      await store.returnStepToDraft(step.id)
      await applySessionChanges(handover)
    })
    if (outcome === null) return
    toast.notify(`${step.title} is a draft now`, {
      label: 'Undo',
      run: () =>
        void run(async () => {
          await store.promoteStep(step.id)
          await store.moveStep(step.id, position)
          await applySessionChanges(handover.map(({ step: successor }) => ({ step: successor, startsSession: false })))
        })
    })
  }

  const deleting = ref<TaskStep | null>(null)
  const deletingBlast = computed(() =>
    deleting.value?.bodyMarkdown?.trim()
      ? 'the step and the detail written under it · not recoverable'
      : 'one step · not recoverable'
  )

  async function removeDeleting(): Promise<void> {
    const step = deleting.value
    deleting.value = null
    if (!step) return
    if (editor.expandedId.value === step.id) editor.close()
    const handover = openerHandover(checklistSteps.value, step)
    await run(async () => {
      await store.removeStep(step.id)
      await applySessionChanges(handover)
    })
  }

  const newTitle = ref('')
  const newSkills = ref<readonly SkillAssignment[]>([])
  const newSession = ref(false)

  /** A new step starts as a draft, opened for writing its detail. */
  async function add(): Promise<TaskStep | null> {
    const title = newTitle.value.trim()
    const task = selectedTask.value
    if (!title || !task) return null
    const skills = newSkills.value
    const opensSession = newSession.value
    newTitle.value = ''
    newSkills.value = []
    newSession.value = false
    const created = await run(async () => {
      const step = await store.addStep(task.id, title)
      if (skills.length) await store.assignStepSkills(step.id, skills)
      if (opensSession) await store.beginStepSession(step.id, true)
      return step
    })
    if (!created) return null
    editor.flush()
    justAddedId.value = created.id
    editor.open(created)
    return created
  }

  onUnmounted(editor.flush)

  return {
    task: selectedTask,
    editor,
    allSteps: selectedTaskSteps,
    checklistSteps,
    draftSteps,
    visibleSteps,
    sessions,
    visibleSessions,
    allDone,
    currentId,
    running,
    firstClaimed,
    isFlow,
    canLayGraph,
    hideDone,
    view,
    reopenArmed,
    justAddedId,
    promoting,
    deleting,
    deletingBlast,
    newTitle,
    newSkills,
    newSession,
    markForward,
    acceptAll,
    chooseSession,
    sendBack,
    reopen,
    move,
    reorder,
    nudge,
    promote,
    returnToDraft,
    removeDeleting,
    add
  }
}
