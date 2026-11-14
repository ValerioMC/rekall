import type { TaskStepId } from '@/model/branded'

/** What the review compares: everything the task changed, one step's commits, one commit, or the uncommitted tree. */
export type ChangeScope =
  | { readonly kind: 'all' }
  | { readonly kind: 'step'; readonly stepId: TaskStepId }
  | { readonly kind: 'commit'; readonly referenceId: string }
  | { readonly kind: 'working-tree' }

export const ALL_CHANGES: ChangeScope = { kind: 'all' }

export const WORKING_TREE_SOURCE_ID = 'working-tree'

export type DiffLayout = 'split' | 'unified'

export function sameScope(a: ChangeScope, b: ChangeScope): boolean {
  if (a.kind !== b.kind) return false
  if (a.kind === 'step' && b.kind === 'step') return a.stepId === b.stepId
  if (a.kind === 'commit' && b.kind === 'commit') return a.referenceId === b.referenceId
  return true
}
