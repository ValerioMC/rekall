import { componentNameKey, componentTitle, missingComponentTitle } from '@/common/markdown/component-reference'
import { connectionReferenceLabel, connectionTitle, missingConnectionTitle } from '@/common/markdown/connection-reference'
import type { ReferenceIcon } from '@/common/markdown/reference-completion-icon'
import { missingTaskTitle, taskReferenceLabel, taskTitle } from '@/common/markdown/task-reference'
import type { Task } from '@/features/catalog/model/catalog'
import type { ProjectConnection } from '@/features/connection/model/connection'
import type { ProjectComponent } from '@/features/project/model/project-component'

/** The project's records a `[[...]]` can name; a list is `null` when the text has no project to ask. */
export interface ReferenceTargets {
  readonly components: readonly ProjectComponent[] | null
  readonly connections: readonly ProjectConnection[] | null
  readonly tasks: readonly Task[] | null
}

export const NO_REFERENCE_TARGETS: ReferenceTargets = { components: null, connections: null, tasks: null }

export interface ReferenceChip {
  readonly text: string
  readonly title: string
  readonly icon: ReferenceIcon | null
  readonly missing: boolean
}

/**
 * What the editor draws for the text between `[[` and `]]`: the read view's chip, with the glyph a
 * suggestion carries. `null` when the text has no project for that kind, so it stays as written.
 */
export function referenceChipFor(written: string, targets: ReferenceTargets): ReferenceChip | null {
  const connection = connectionReferenceLabel(written)
  if (connection !== null) return connectionChip(connection, targets.connections)
  const task = taskReferenceLabel(written)
  if (task !== null) return taskChip(task, targets.tasks)
  return componentChip(written, targets.components)
}

function componentChip(name: string, components: readonly ProjectComponent[] | null): ReferenceChip | null {
  if (components === null) return null
  const found = components.find((component) => componentNameKey(component.name) === componentNameKey(name))
  return found
    ? { text: name, title: componentTitle(found), icon: 'component', missing: false }
    : { text: name, title: missingComponentTitle(name), icon: 'component', missing: true }
}

function connectionChip(label: string, connections: readonly ProjectConnection[] | null): ReferenceChip | null {
  if (connections === null) return null
  const found = connections.find((binding) => binding.label.toLowerCase() === label.toLowerCase())
  return found
    ? { text: label, title: connectionTitle(found), icon: found.kind, missing: false }
    : { text: label, title: missingConnectionTitle(label), icon: null, missing: true }
}

function taskChip(label: string, tasks: readonly Task[] | null): ReferenceChip | null {
  if (tasks === null) return null
  const found = tasks.find((task) => task.label.toLowerCase() === label.toLowerCase())
  return found
    ? { text: label, title: taskTitle(found), icon: 'task', missing: false }
    : { text: label, title: missingTaskTitle(label), icon: 'task', missing: true }
}
