import { inject, provide, ref, type InjectionKey, type Ref } from 'vue'
import { useEditorMode } from '@/composables/useEditorMode'
import type { TaskStep } from '@/features/catalog/model/catalog'
import type { TaskStepId } from '@/model/branded'

/**
 * The one step a checklist has open, and the title and detail being typed into it. Typing is
 * saved 700 ms after it stops; any move away from the step (another step, a state change, the
 * pane closing) writes what is pending first, so nothing typed is lost to a quick click.
 */
export interface StepEditor {
  readonly expandedId: Ref<TaskStepId | null>
  readonly mode: Ref<'write' | 'read'>
  readonly title: Ref<string>
  readonly body: Ref<string>
  open: (step: TaskStep) => void
  toggle: (step: TaskStep) => void
  close: () => void
  flush: () => void
  scheduleSave: () => void
}

export interface StepEditorWiring {
  /** The steps the open one is looked up in, to skip a save that changes nothing. */
  steps: () => readonly TaskStep[]
  save: (id: TaskStepId, changes: { title: string; bodyMarkdown: string }) => void
  markUnsaved: () => void
}

const KEY: InjectionKey<StepEditor> = Symbol('step-editor')
const SAVE_DELAY_MS = 700

export function provideStepEditor(wiring: StepEditorWiring): StepEditor {
  const expandedId = ref<TaskStepId | null>(null)
  const { mode, openFor } = useEditorMode('read')
  const title = ref('')
  const body = ref('')
  let saveTimer: ReturnType<typeof setTimeout> | null = null

  function writePending(): void {
    const id = expandedId.value
    const trimmed = title.value.trim()
    if (!id || !trimmed) return
    const step = wiring.steps().find((candidate) => candidate.id === id)
    if (!step || (step.title === trimmed && (step.bodyMarkdown ?? '') === body.value)) return
    wiring.save(id, { title: trimmed, bodyMarkdown: body.value })
  }

  function flush(): void {
    if (!saveTimer) return
    clearTimeout(saveTimer)
    saveTimer = null
    writePending()
  }

  function scheduleSave(): void {
    wiring.markUnsaved()
    if (saveTimer) clearTimeout(saveTimer)
    saveTimer = setTimeout(() => {
      saveTimer = null
      writePending()
    }, SAVE_DELAY_MS)
  }

  /** A step with detail opens to read it; one without opens to write it. */
  function open(step: TaskStep): void {
    expandedId.value = step.id
    title.value = step.title
    body.value = step.bodyMarkdown ?? ''
    openFor(step.bodyMarkdown ?? '')
  }

  function toggle(step: TaskStep): void {
    flush()
    if (expandedId.value === step.id) {
      expandedId.value = null
      return
    }
    open(step)
  }

  function close(): void {
    expandedId.value = null
  }

  const editor: StepEditor = { expandedId, mode, title, body, open, toggle, close, flush, scheduleSave }
  provide(KEY, editor)
  return editor
}

export function useStepEditor(): StepEditor {
  const editor = inject(KEY)
  if (!editor) throw new Error('useStepEditor() needs a StepsPane above it')
  return editor
}
