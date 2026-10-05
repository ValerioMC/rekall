import {
  deleteCommitReference as apiDeleteCommitReference,
  setCommitReferenceInContext as apiSetCommitReferenceInContext
} from '@/api/commitReference.api'
import type { CommitReference } from '@/model/commitReference'
import type { ConsoleState } from './console-state'
import { upsertById } from './rows'

/** The commits logged against tasks, and whether each one goes into the task's context. */
export function useCommitReferences(state: ConsoleState) {
  const { commitReferences } = state

  // The log-commit button already has the row it just wrote; folding it in here keeps the list
  // current without a round trip, the same way applyStepEvent does for the SSE step stream.
  function applyCommitReference(reference: CommitReference): void {
    commitReferences.value = upsertById(commitReferences.value, reference, 'first')
  }

  async function setCommitReferenceInContext(id: string, inContext: boolean): Promise<void> {
    applyCommitReference(await apiSetCommitReferenceInContext(id, inContext))
  }

  async function deleteCommitReference(id: string): Promise<void> {
    await apiDeleteCommitReference(id)
    commitReferences.value = commitReferences.value.filter((existing) => existing.id !== id)
  }

  return { applyCommitReference, setCommitReferenceInContext, deleteCommitReference }
}
