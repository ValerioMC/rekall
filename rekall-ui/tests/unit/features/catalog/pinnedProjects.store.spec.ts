import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { usePinnedProjectsStore } from '@/features/catalog/store/pinnedProjects.store'
import { installStorage } from '../../support/memoryStorage'
import type { ProjectId } from '@/model/branded'

const a = 'a' as ProjectId
const b = 'b' as ProjectId
const c = 'c' as ProjectId
const d = 'd' as ProjectId

beforeEach(() => {
  installStorage()
  setActivePinia(createPinia())
})

afterEach(() => delete (window as { localStorage?: unknown }).localStorage)

describe('the pinned projects', () => {
  it('starts empty', () => {
    expect(usePinnedProjectsStore().ids).toEqual([])
  })

  it('keeps projects in the order they were pinned and unpins on a second press', () => {
    const store = usePinnedProjectsStore()

    expect(store.toggle(b)).toBe('pinned')
    expect(store.toggle(a)).toBe('pinned')
    expect(store.ids).toEqual([b, a])

    expect(store.toggle(b)).toBe('unpinned')
    expect(store.ids).toEqual([a])
    expect(store.isPinned(b)).toBe(false)
  })

  it('refuses a fourth pin and leaves the three it has', () => {
    const store = usePinnedProjectsStore()
    ;[a, b, c].forEach((id) => store.toggle(id))

    expect(store.toggle(d)).toBe('full')
    expect(store.ids).toEqual([a, b, c])
  })

  it('reads back, in the same order, what an earlier session kept', () => {
    const earlier = usePinnedProjectsStore()
    earlier.toggle(b)
    earlier.toggle(a)

    setActivePinia(createPinia())

    expect(usePinnedProjectsStore().ids).toEqual([b, a])
  })
})
