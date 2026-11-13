import { computed, onUnmounted, ref, watch, type ComputedRef, type Ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useCheckStream } from '@/features/check/composables/useCheckStream'
import { useChecksStore } from '@/features/check/store/checks.store'
import { provideStepEditor, type StepEditor } from '@/features/steps/composables/useStepEditor'
import { groupBySession, type StepSession } from '@/features/steps/lib/step-sessions'
import { stepIsComplete, type TaskStep } from '@/features/steps/model/task-step'
import type { Task } from '@/features/catalog/model/catalog'
import { useConsoleStore } from '@/stores/console.store'
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
  readonly allDone: ComputedRef<boolean>
  readonly currentId: ComputedRef<TaskStepId | null>
  readonly running: ComputedRef<boolean>
  readonly firstClaimed: ComputedRef<TaskStep | null>
  readonly isFlow: ComputedRef<boolean>
  readonly canLayGraph: ComputedRef<boolean>
  readonly hideDone: Ref<boolean>
  readonly view: Ref<'list' | 'flow'>
  readonly reopenArmed: Ref<TaskStepId | null>
  readonly promoting: Ref<TaskStepId | null>
  readonly deleting: Ref<TaskStep | null>
  readonly deletingBlast: ComputedRef<string>
  readonly newTitle: Ref<string>
  readonly newSkills: Ref<readonly string[]>
  readonly newSession: Ref<boolean>
  markForward: (step: TaskStep) => Promise<void>
  acceptAll: () => Promise<void>
  chooseSession: (step: TaskStep, startsSession: boolean) => Promise<void>
  sendBack: (step: TaskStep) => Promise<void>
  reopen: (step: TaskStep) => Promise<void>
  move: (step: TaskStep, by: number) => Promise<void>
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

  const view = ref<'list' | 'flow'>('list')
  const isFlow = computed(() => checklistSteps.value.some((step) => step.links.length > 0))
  const canLayGraph = computed(() => !draftSteps.value.length && selectedTask.value?.graphLaidAt === null)

  function stepAwaitingDecision(excluding: TaskStepId | null): TaskStep | undefined {
    return checklistSteps.value.find((step) => step.state === 'CLAIMED' && step.id !== excluding)
  }

  function nextUnfinishedStep(excluding: TaskStepId | null): TaskStep | undefined {
    return checklistSteps.value.find((step) => !stepIsComplete(step.state) && step.id !== excluding)
  }

  /** A claim waiting for a person comes first, then the first step still to finish, then the first draft. */
  function openFirstUnfinished(): void {
    const next = stepAwaitingDecision(null) ?? nextUnfinishedStep(null) ?? draftSteps.value[0]
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

  function offerToReopenAccepted(step: TaskStep): void {
    toast.notify(`Accepted ${step.title}`, {
      label: 'Reopen',
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
    if (wasClaimed && outcome !== null) offerToReopenAccepted(step)
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

  async function removeDeleting(): Promise<void> {
    const step = deleting.value
    deleting.value = null
    if (!step) return
    if (editor.expandedId.value === step.id) editor.close()
    await run(() => store.removeStep(step.id))
  }

  const newTitle = ref('')
  const newSkills = ref<readonly string[]>([])
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
    allDone,
    currentId,
    running,
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
    newSession,
    markForward,
    acceptAll,
    chooseSession,
    sendBack,
    reopen,
    move,
    promote,
    returnToDraft,
    removeDeleting,
    add
  }
}
