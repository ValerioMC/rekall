import type { Completion, CompletionContext, CompletionResult, CompletionSource } from '@codemirror/autocomplete'
import { insertReference, isInsideCode } from '@/common/markdown/component-completion'
import { referenceCompletionType } from '@/common/markdown/reference-completion-icon'
import { TASK_REFERENCE_PREFIX } from '@/common/markdown/task-reference'
import { TASK_STATUS_LABEL, type Task } from '@/features/catalog/model/catalog'

/** Only `[[task:` opens the list: a project holds too many tasks to offer them after a bare `[[`. */
const TASK_OPENING = /\[\[task:[^[\]\n]*$/i

/** The tasks whose label or title contains what was typed after `[[task:`, by label. */
export function taskOptions(tasks: readonly Task[], typed: string): Completion[] {
  const wanted = typed.trim().toLowerCase()
  return [...tasks]
    .filter((task) => !task.label.includes(']]'))
    .filter((task) => task.label.toLowerCase().includes(wanted) || task.title.toLowerCase().includes(wanted))
    .sort((left, right) => left.label.localeCompare(right.label))
    .map((task) => ({
      label: `${TASK_REFERENCE_PREFIX}${task.label}`,
      detail: `${task.title} · ${TASK_STATUS_LABEL[task.status]}`,
      type: referenceCompletionType('task'),
      section: { name: 'Tasks', rank: 2 },
      apply: insertReference(`${TASK_REFERENCE_PREFIX}${task.label}]]`)
    }))
}

/**
 * Typing `[[task:` offers the project's other tasks, written as `[[task:label]]`. Nothing opens
 * before the prefix, without tasks, inside code, or with no match.
 */
export function taskCompletion(tasks: () => readonly Task[] | null): CompletionSource {
  return (context: CompletionContext): CompletionResult | null => {
    const all = tasks()
    if (!all || all.length === 0) return null
    const opening = context.matchBefore(TASK_OPENING)
    if (!opening || isInsideCode(context)) return null
    const from = opening.from + 2
    const typed = context.state.sliceDoc(from + TASK_REFERENCE_PREFIX.length, context.pos)
    const options = taskOptions(all, typed)
    return options.length === 0 ? null : { from, options, filter: false }
  }
}
