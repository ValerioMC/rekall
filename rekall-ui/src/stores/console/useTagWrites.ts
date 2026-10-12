import {
  createTag as apiCreateTag,
  deleteTag as apiDeleteTag,
  updateTag as apiUpdateTag
} from '@/api/catalog.api'
import type { TagInput } from '@/api/catalog.api'
import type { Tag } from '@/model/catalog'
import type { TagId } from '@/model/branded'
import type { ConsoleState } from './console-state'
import type { useConsoleRefreshes } from './useConsoleRefreshes'
import { replaceById } from './rows'

/** Tags sorted by name. Tasks show their tag, so a renamed or deleted one re-reads them. */
export function useTagWrites(state: ConsoleState, refreshes: ReturnType<typeof useConsoleRefreshes>) {
  const { tags } = state

  async function createTag(input: TagInput): Promise<Tag> {
    const created = await apiCreateTag(input)
    tags.value = [...tags.value, created].sort((a, b) => a.name.localeCompare(b.name))
    return created
  }

  async function updateTag(id: TagId, input: TagInput): Promise<void> {
    const saved = await apiUpdateTag(id, input)
    tags.value = replaceById(tags.value, saved).sort((a, b) => a.name.localeCompare(b.name))
    await refreshes.refreshTasks()
  }

  async function deleteTag(id: TagId): Promise<void> {
    await apiDeleteTag(id)
    tags.value = tags.value.filter((tag) => tag.id !== id)
    await Promise.all([refreshes.refreshTasks(), refreshes.refreshSkillAssignments()])
  }

  return { createTag, updateTag, deleteTag }
}
