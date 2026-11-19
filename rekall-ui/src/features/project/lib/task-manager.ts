import type { ConnectionKind, ProjectConnection } from '@/features/connection/model/connection'

const TASK_MANAGER_KINDS: readonly ConnectionKind[] = ['GITLAB', 'GITHUB', 'JIRA']

/** A GitLab, GitHub or Jira binding can receive a project's tasks; a cluster or a Grafana instance cannot. */
export function taskManagerCandidates(bindings: readonly ProjectConnection[]): readonly ProjectConnection[] {
  return bindings.filter((binding) => TASK_MANAGER_KINDS.includes(binding.kind))
}

export function currentTaskManager(bindings: readonly ProjectConnection[]): ProjectConnection | null {
  return bindings.find((binding) => binding.taskManager) ?? null
}

/** The server refuses a task manager without the place its issues go. */
export function lacksTarget(binding: ProjectConnection): boolean {
  return !binding.target?.trim()
}
