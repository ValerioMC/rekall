import { describe, expect, it } from 'vitest'
import {
  canMoveContext,
  canMovePage,
  contextMoved,
  layoutOf,
  pageMovedToContext,
  pageMovedWithin,
  sameLayout
} from '@/features/project/lib/section-layout'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { ContextPlacement, SectionContext } from '@/features/project/model/section-context'
import { asProjectId, asProjectSectionId, asSectionContextId } from '@/model/branded'

const [a, b, c] = ['a', 'b', 'c'].map(asProjectSectionId) as [ReturnType<typeof asProjectSectionId>, ReturnType<typeof asProjectSectionId>, ReturnType<typeof asProjectSectionId>]
const ops = asSectionContextId('ops')
const team = asSectionContextId('team')
const empty = asSectionContextId('empty')

function context(id: typeof ops, position: number): SectionContext {
  return { id, projectId: asProjectId('p'), title: id, position, createdAt: 't', updatedAt: 't' }
}

function section(id: typeof a, contextId: typeof ops): ProjectSection {
  return { id, projectId: asProjectId('p'), contextId, title: id, bodyMarkdown: '', position: 0, createdAt: 't', updatedAt: 't' }
}

const layout: ContextPlacement[] = [
  { contextId: ops, sectionIds: [a, b] },
  { contextId: team, sectionIds: [c] },
  { contextId: empty, sectionIds: [] }
]

describe('section layout', () => {
  it('reads the arrangement from the contexts and sections in the order they are shown', () => {
    const read = layoutOf([context(ops, 0), context(team, 1), context(empty, 2)], [section(a, ops), section(c, team), section(b, ops)])

    expect(read).toEqual([
      { contextId: ops, sectionIds: [a, b] },
      { contextId: team, sectionIds: [c] },
      { contextId: empty, sectionIds: [] }
    ])
  })

  it('moves a page one place inside its context and stops at either end', () => {
    expect(pageMovedWithin(layout, b, 'up')[0]?.sectionIds).toEqual([b, a])
    expect(pageMovedWithin(layout, a, 'down')[0]?.sectionIds).toEqual([b, a])
    expect(pageMovedWithin(layout, a, 'up')).toEqual(layout)
    expect(pageMovedWithin(layout, c, 'down')).toEqual(layout)
    expect([canMovePage(layout, a, 'up'), canMovePage(layout, a, 'down'), canMovePage(layout, c, 'down')]).toEqual([false, true, false])
  })

  it('moves a page into another context, last or before the page it was dropped on', () => {
    expect(pageMovedToContext(layout, a, team, null)).toEqual([
      { contextId: ops, sectionIds: [b] },
      { contextId: team, sectionIds: [c, a] },
      { contextId: empty, sectionIds: [] }
    ])
    expect(pageMovedToContext(layout, a, team, c)[1]?.sectionIds).toEqual([a, c])
    expect(pageMovedToContext(layout, c, empty, null)[2]?.sectionIds).toEqual([c])
  })

  it('reorders inside one context when a page is dropped before another of it', () => {
    expect(pageMovedToContext(layout, b, ops, a)[0]?.sectionIds).toEqual([b, a])
    expect(pageMovedToContext(layout, a, ops, null)[0]?.sectionIds).toEqual([b, a])
    expect(sameLayout(pageMovedToContext(layout, a, ops, null), layout)).toBe(false)
    expect(sameLayout(pageMovedToContext(layout, a, ops, b), layout)).toBe(true)
    expect(sameLayout(pageMovedToContext(layout, a, ops, a), layout)).toBe(true)
  })

  it('moves a context one place and stops at either end', () => {
    expect(contextMoved(layout, team, 'up').map((placement) => placement.contextId)).toEqual([team, ops, empty])
    expect(contextMoved(layout, ops, 'up')).toEqual(layout)
    expect([canMoveContext(layout, ops, 'up'), canMoveContext(layout, ops, 'down'), canMoveContext(layout, empty, 'down')]).toEqual([false, true, false])
  })
})
