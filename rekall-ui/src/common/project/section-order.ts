import type { ProjectSectionId } from '@/model/branded'

export type Direction = 'up' | 'down'

/** `ids` with `id` one place earlier or later; the same list when it is already at that end. */
export function movedOne(
  ids: readonly ProjectSectionId[],
  id: ProjectSectionId,
  direction: Direction
): ProjectSectionId[] {
  const from = ids.indexOf(id)
  const to = direction === 'up' ? from - 1 : from + 1
  if (from < 0 || to < 0 || to >= ids.length) return [...ids]
  const next = [...ids]
  next.splice(from, 1)
  next.splice(to, 0, id)
  return next
}

/** Whether `id` can move that way: it is listed and not already at that end. */
export function canMove(ids: readonly ProjectSectionId[], id: ProjectSectionId, direction: Direction): boolean {
  const from = ids.indexOf(id)
  if (from < 0) return false
  return direction === 'up' ? from > 0 : from < ids.length - 1
}
