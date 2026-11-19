import type { TaskStepId } from '@/model/branded'

/**
 * What the review compares: everything the task changed, one repository, one step's commits, one
 * commit, or the uncommitted tree of one repository.
 */
export type ChangeScope =
  | { readonly kind: 'all' }
  | { readonly kind: 'repository'; readonly repositoryId: string }
  | { readonly kind: 'step'; readonly stepId: TaskStepId }
  | { readonly kind: 'commit'; readonly referenceId: string }
  | { readonly kind: 'working-tree'; readonly repositoryId: string }

export const ALL_CHANGES: ChangeScope = { kind: 'all' }

/** The id the project's own folder answers to; a component's repository answers to the component's id. */
export const PROJECT_REPOSITORY_ID = 'project'

export type DiffLayout = 'split' | 'unified'

export function sameScope(a: ChangeScope, b: ChangeScope): boolean {
  if (a.kind !== b.kind) return false
  if (a.kind === 'step' && b.kind === 'step') return a.stepId === b.stepId
  if (a.kind === 'commit' && b.kind === 'commit') return a.referenceId === b.referenceId
  if (a.kind === 'repository' && b.kind === 'repository') return a.repositoryId === b.repositoryId
  if (a.kind === 'working-tree' && b.kind === 'working-tree') return a.repositoryId === b.repositoryId
  return true
}
