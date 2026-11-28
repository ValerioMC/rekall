import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import {
  DEFAULT_PROJECT_TASK_PREFERENCES,
  loadProjectTaskPreferences,
  saveProjectTaskPreferences,
  type ProjectTaskPreferences
} from '@/common/config/project-task-preferences'
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
    expect(loadProjectTaskPreferences()).toEqual(DEFAULT_PROJECT_TASK_PREFERENCES)
  })

  it('reads back what was saved', () => {
    const saved: ProjectTaskPreferences = {
      view: 'all',
      density: 'compact',
      sort: { key: 'tokens', descending: false },
      pageSize: 100,
      hiddenColumns: ['notes'],
      sidePreview: false,
      filters: { p1: { status: ['blocked'], tag: ['t1'], epic: ['none'], type: ['quick'] } },
      companyId: 'c1' as CompanyId,
      projectId: 'p1' as ProjectId
    }

    saveProjectTaskPreferences(saved)

    expect(loadProjectTaskPreferences()).toEqual(saved)
  })

  it('falls back to the default for a value it does not know', () => {
    window.localStorage.setItem('rekall.work.preferences', JSON.stringify({ view: 'archive', density: 'huge', companyId: 7 }))

    expect(loadProjectTaskPreferences()).toEqual(DEFAULT_PROJECT_TASK_PREFERENCES)
  })

  it('keeps what it can of stored choices and drops what it does not know', () => {
    window.localStorage.setItem(
      'rekall.work.preferences',
      JSON.stringify({
        sort: { key: 'needs', descending: false },
        pageSize: 33,
        hiddenColumns: ['tag', 'title', 4],
        sidePreview: 'yes',
        filters: { p1: { tag: ['t1', 7], epic: 'e1' } }
      })
    )

    expect(loadProjectTaskPreferences()).toEqual({
      ...DEFAULT_PROJECT_TASK_PREFERENCES,
      sort: { key: 'task', descending: false },
      hiddenColumns: ['tag'],
      filters: { p1: { status: [], tag: ['t1'], epic: [], type: [] } }
    })
  })

  it('falls back to the default when the stored text is not JSON', () => {
    window.localStorage.setItem('rekall.work.preferences', '{')

    expect(loadProjectTaskPreferences()).toEqual(DEFAULT_PROJECT_TASK_PREFERENCES)
  })
})
