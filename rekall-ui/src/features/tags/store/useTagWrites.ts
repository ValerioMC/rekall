import {
  createTag as apiCreateTag,
  deleteTag as apiDeleteTag,
  updateTag as apiUpdateTag
} from '@/features/catalog/api/catalog.api'
import type { TagInput } from '@/features/catalog/api/catalog.api'
import type { Tag } from '@/features/catalog/model/catalog'
import type { TagId } from '@/model/branded'
import type { ConsoleState } from '../../../stores/console/console-state'
import type { useConsoleRefreshes } from '../../../stores/console/useConsoleRefreshes'
import { replaceById } from '../../../stores/console/rows'

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
