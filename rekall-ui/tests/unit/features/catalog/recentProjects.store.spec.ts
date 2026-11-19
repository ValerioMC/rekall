import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useRecentProjectsStore } from '@/features/catalog/store/recentProjects.store'
import { installStorage } from '../../support/memoryStorage'
import type { ProjectId } from '@/model/branded'

const a = 'a' as ProjectId
const b = 'b' as ProjectId
const c = 'c' as ProjectId
const d = 'd' as ProjectId
const e = 'e' as ProjectId
const f = 'f' as ProjectId

let values: Map<string, string>

beforeEach(() => {
  values = installStorage()
  setActivePinia(createPinia())
})

afterEach(() => delete (window as { localStorage?: unknown }).localStorage)

describe('the recent projects', () => {
  it('starts empty', () => {
    expect(useRecentProjectsStore().ids).toEqual([])
  })

  it('puts the project opened last first and keeps each project once', () => {
    const store = useRecentProjectsStore()

    store.remember(a)
    store.remember(b)
    store.remember(a)

    expect(store.ids).toEqual([a, b])
  })

  it('keeps the five newest', () => {
    const store = useRecentProjectsStore()

    ;[a, b, c, d, e, f].forEach((id) => store.remember(id))

    expect(store.ids).toEqual([f, e, d, c, b])
  })

  it('reads back, in the same order, what an earlier session kept', () => {
    const earlier = useRecentProjectsStore()
    earlier.remember(a)
    earlier.remember(b)

    setActivePinia(createPinia())

    expect(useRecentProjectsStore().ids).toEqual([b, a])
    expect(values.size).toBe(1)
  })

  it('still works for the session when storage refuses to write', () => {
    installStorage({
      setItem: () => {
        throw new Error('full')
      }
    })
    setActivePinia(createPinia())
    const store = useRecentProjectsStore()

    store.remember(a)

    expect(store.ids).toEqual([a])
  })
})
