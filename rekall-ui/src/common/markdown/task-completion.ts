import type { Completion, CompletionContext, CompletionResult, CompletionSource } from '@codemirror/autocomplete'
import { insertReference, isInsideCode } from '@/common/markdown/component-completion'
import { referenceCompletionType } from '@/common/markdown/reference-completion-icon'
import { TASK_REFERENCE_PREFIX } from '@/common/markdown/task-reference'
import { TASK_STATUS_LABEL } from '@/features/catalog/model/task-status'
import { type Task } from '@/features/catalog/model/catalog'
import { newestFirst } from '@/features/catalog/model/task-code'

/** Only `[[task:` opens the list: a project holds too many tasks to offer them after a bare `[[`. */
const TASK_OPENING = /\[\[task:[^[\]\n]*$/i

/** The tasks whose code, title or description contains what was typed after `[[task:`, newest first. */
export function taskOptions(tasks: readonly Task[], typed: string): Completion[] {
  const wanted = typed.trim().toLowerCase()
  return [...tasks]
    .filter((task) => matchesTaskText(task, wanted))
    .sort(newestFirst)
    .map((task) => ({
      label: `${TASK_REFERENCE_PREFIX}${task.code}`,
      detail: `${task.title} · ${TASK_STATUS_LABEL[task.status]}`,
      info: task.description?.trim() ? task.description.trim().slice(0, 240) : undefined,
      type: referenceCompletionType('task'),
      section: { name: 'Tasks', rank: 2 },
      apply: insertReference(`${TASK_REFERENCE_PREFIX}${task.code}]]`)
    }))
}

function matchesTaskText(task: Task, wanted: string): boolean {
  return [task.code, task.title, task.description ?? ''].some((text) => text.toLowerCase().includes(wanted))
}

/**
 * Typing `[[task:` offers the project's other tasks, written as `[[task:RK-12]]`. Nothing opens
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
