import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { DEFAULT_WORK_PREFERENCES, loadWorkPreferences, saveWorkPreferences } from '@/common/config/work-preferences'
import type { CompanyId, ProjectId } from '@/model/branded'

describe('work preferences', () => {
  beforeEach(() => {
    const values = new Map<string, string>()
    Object.defineProperty(window, 'localStorage', {
      configurable: true,
      value: {
        getItem: (key: string) => values.get(key) ?? null,
        setItem: (key: string, value: string) => void values.set(key, value)
      }
    })
  })
  afterEach(() => delete (window as { localStorage?: unknown }).localStorage)

  it('opens on Needs me, comfortable and unscoped when nothing was kept', () => {
    expect(loadWorkPreferences()).toEqual(DEFAULT_WORK_PREFERENCES)
  })

  it('reads back what was saved', () => {
    const saved = { view: 'running', density: 'compact', companyId: 'c1' as CompanyId, projectId: 'p1' as ProjectId } as const

    saveWorkPreferences(saved)

    expect(loadWorkPreferences()).toEqual(saved)
  })

  it('falls back to the default for a value it does not know', () => {
    window.localStorage.setItem('rekall.work.preferences', JSON.stringify({ view: 'archive', density: 'huge', companyId: 7 }))

    expect(loadWorkPreferences()).toEqual(DEFAULT_WORK_PREFERENCES)
  })

  it('falls back to the default when the stored text is not JSON', () => {
    window.localStorage.setItem('rekall.work.preferences', '{')

    expect(loadWorkPreferences()).toEqual(DEFAULT_WORK_PREFERENCES)
  })
})
