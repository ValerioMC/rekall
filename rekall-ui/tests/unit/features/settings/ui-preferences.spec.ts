import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { installStorage } from '../../support/memoryStorage'
import { DEFAULT_UI_PREFERENCES, loadUiPreferences } from '@/common/config/ui-preferences'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { nextTick } from 'vue'

const KEY = 'rekall.ui.preferences'

describe('the stored UI preferences', () => {
  beforeEach(() => {
    installStorage()
    setActivePinia(createPinia())
  })

  it('start from the defaults when nothing is stored', () => {
    expect(loadUiPreferences()).toEqual(DEFAULT_UI_PREFERENCES)
  })

  it('keep a valid choice and fall back on a value they do not know', () => {
    window.localStorage.setItem(KEY, JSON.stringify({ motion: 'reduced', scale: 33, reviewAutoAdvance: 'yes', workdayGoalHours: 40, usageWarningPercent: 70 }))

    expect(loadUiPreferences()).toEqual({
      ...DEFAULT_UI_PREFERENCES,
      motion: 'reduced',
      scale: 100,
      reviewAutoAdvance: true,
      workdayGoalHours: 16,
      usageWarningPercent: 70
    })
  })

  it('keep the width of the task page, and fall back on one they do not know', () => {
    window.localStorage.setItem(KEY, JSON.stringify({ taskWidth: 'wide' }))
    expect(loadUiPreferences().taskWidth).toBe('wide')

    window.localStorage.setItem(KEY, JSON.stringify({ taskWidth: 'huge' }))
    expect(loadUiPreferences().taskWidth).toBe('wide')
  })

  it('survive unreadable storage', () => {
    window.localStorage.setItem(KEY, '{broken')

    expect(loadUiPreferences()).toEqual(DEFAULT_UI_PREFERENCES)
  })

  it('are written when the store changes, and a goal outside its range is held to it', async () => {
    const preferences = usePreferencesStore()
    preferences.motion = 'reduced'
    preferences.taskWidth = 'narrow'
    preferences.setWorkdayGoal(0)
    preferences.setUsageWarning(120)
    await nextTick()

    expect(JSON.parse(window.localStorage.getItem(KEY)!)).toMatchObject({ motion: 'reduced', taskWidth: 'narrow', workdayGoalHours: 1, usageWarningPercent: 100 })
  })
})
