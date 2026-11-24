import { canMove, movedOne, type Direction } from '@/features/project/lib/section-order'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { ContextPlacement, SectionContext, SectionLayout } from '@/features/project/model/section-context'
import type { ProjectSectionId, SectionContextId } from '@/model/branded'

/** The arrangement as it stands: the contexts in order, each with the ids of its pages in order. */
export function layoutOf(contexts: readonly SectionContext[], sections: readonly ProjectSection[]): ContextPlacement[] {
  return contexts.map((context) => ({
    contextId: context.id,
    sectionIds: sections.filter((section) => section.contextId === context.id).map((section) => section.id)
  }))
}

/** Whether `sectionId` can go one place that way inside its context. */
export function canMovePage(layout: readonly ContextPlacement[], sectionId: ProjectSectionId, direction: Direction): boolean {
  const held = layout.find((placement) => placement.sectionIds.includes(sectionId))
  return held ? canMove(held.sectionIds, sectionId, direction) : false
}

/** `sectionId` one place earlier or later in its context; the same arrangement when it is already at that end. */
export function pageMovedWithin(
  layout: readonly ContextPlacement[],
  sectionId: ProjectSectionId,
  direction: Direction
): ContextPlacement[] {
  return layout.map((placement) =>
    placement.sectionIds.includes(sectionId)
      ? { ...placement, sectionIds: movedOne(placement.sectionIds, sectionId, direction) }
      : placement
  )
}

/**
 * `sectionId` taken out of its context and put in `contextId`, before `beforeId` or last when that
 * is `null`. Dropped on its own place or onto itself it changes nothing.
 */
export function pageMovedToContext(
  layout: readonly ContextPlacement[],
  sectionId: ProjectSectionId,
  contextId: SectionContextId,
  beforeId: ProjectSectionId | null
): ContextPlacement[] {
  if (beforeId === sectionId) return [...layout]
  const without = layout.map((placement) => ({ ...placement, sectionIds: placement.sectionIds.filter((id) => id !== sectionId) }))
  return without.map((placement) => {
    if (placement.contextId !== contextId) return placement
    const at = beforeId === null ? -1 : placement.sectionIds.indexOf(beforeId)
    const sectionIds = [...placement.sectionIds]
    sectionIds.splice(at < 0 ? sectionIds.length : at, 0, sectionId)
    return { ...placement, sectionIds }
  })
}

export function canMoveContext(layout: readonly ContextPlacement[], contextId: SectionContextId, direction: Direction): boolean {
  return canMove(
    layout.map((placement) => placement.contextId),
    contextId,
    direction
  )
}

/** `contextId` one place earlier or later among the contexts; the same arrangement when it is already at that end. */
export function contextMoved(layout: readonly ContextPlacement[], contextId: SectionContextId, direction: Direction): ContextPlacement[] {
  const order = movedOne(
    layout.map((placement) => placement.contextId),
    contextId,
    direction
  )
  return order.flatMap((id) => layout.filter((placement) => placement.contextId === id))
}

/** Whether the arrangements put every context and page in the same place. */
export function sameLayout(a: readonly ContextPlacement[], b: readonly ContextPlacement[]): boolean {
  return JSON.stringify(a) === JSON.stringify(b)
}

/** The contexts and sections as `placements` would leave them, shown before the server confirms. */
export function arranged(
  contexts: readonly SectionContext[],
  sections: readonly ProjectSection[],
  placements: readonly ContextPlacement[]
): SectionLayout {
  const orderedContexts = placements.flatMap((placement, position) => {
    const known = contexts.find((context) => context.id === placement.contextId)
    return known ? [{ ...known, position }] : []
  })
  const orderedSections = placements.flatMap((placement) =>
    placement.sectionIds.flatMap((id, position) => {
      const known = sections.find((section) => section.id === id)
      return known ? [{ ...known, contextId: placement.contextId, position }] : []
    })
  )
  return { contexts: orderedContexts, sections: orderedSections }
}
