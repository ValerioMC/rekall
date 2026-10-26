import { TASK_STATUS_LABEL, type Task } from '@/features/catalog/model/catalog'

/** What turns a `[[...]]` reference into one naming another task of the project: `[[task:export]]`. */
export const TASK_REFERENCE_PREFIX = 'task:'
export const TASK_CHIP_CLASS = 'rk-task-ref'

/** The task label a reference names, or `null` when it names something else. */
export function taskReferenceLabel(name: string): string | null {
  const trimmed = name.trim()
  return trimmed.startsWith(TASK_REFERENCE_PREFIX) ? trimmed.slice(TASK_REFERENCE_PREFIX.length).trim() : null
}

/** The chip for an already escaped label, settled later by `paintTaskReferences`. */
export function taskChipHtml(escapedLabel: string): string {
  return `<span class="${TASK_CHIP_CLASS}" data-task="${escapedLabel}" data-state="plain">${escapedLabel}</span>`
}

/**
 * Settles every task chip under `root` against the project's tasks: found (title and status on
 * hover), missing, or plain when the text has no project (`null`).
 */
export function paintTaskReferences(root: ParentNode, tasks: readonly Task[] | null): void {
  for (const chip of root.querySelectorAll<HTMLElement>(`.${TASK_CHIP_CLASS}`)) {
    const label = chip.dataset['task'] ?? ''
    const found = tasks?.find((task) => task.label.toLowerCase() === label.toLowerCase())
    if (tasks === null) {
      chip.dataset['state'] = 'plain'
      chip.removeAttribute('title')
    } else if (found) {
      chip.dataset['state'] = 'found'
      chip.title = `${found.title}\n${TASK_STATUS_LABEL[found.status]}`
    } else {
      chip.dataset['state'] = 'missing'
      chip.title = `This project has no task labelled “${label}”.`
    }
  }
}
